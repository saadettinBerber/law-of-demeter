use super::support::{
    ProbeResult, crate_for_root, declared_function, declared_method, declared_struct, flag,
    load_fixture_at, method_call,
};
use std::path::{Path, PathBuf};
use std::process::Command;

use ra_ap_cfg::{CfgAtom, CfgDiff};
use ra_ap_hir::{Adt, AnyDiagnostic, Crate, EditionedFileId, InFile, ModuleDef, Semantics};
use ra_ap_ide_db::RootDatabase;
use ra_ap_intern::Symbol;
use ra_ap_load_cargo::{LoadCargoConfig, ProcMacroServerChoice, load_workspace};
use ra_ap_project_model::{
    CargoConfig, CargoFeatures, CfgOverrides, ProjectManifest, ProjectWorkspace,
    ProjectWorkspaceKind, RustLibSource, TargetDirectoryConfig,
};
use ra_ap_syntax::{AstNode, SyntaxNodePtr, ast};
use ra_ap_vfs::{AbsPathBuf, Vfs};

#[test]
fn a_false_cfg_boolean_keeps_the_unresolved_call_active() -> ProbeResult {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("experiments/unresolved-cfg/fixture")
        .canonicalize()?;
    let model = load_fixture_at(
        &directory,
        "../../../../rust-expansion/consumer/negative/cfg_boolean.rs",
        false,
    )?;

    ra_ap_hir::attach_db(&model.db, || {
        let semantics = Semantics::new(&model.db);
        let source = model.source(&semantics);
        let caller = declared_function(&source, "main");
        let definition = semantics.to_def(&caller).ok_or("main is inactive")?;
        let call = free_call(&caller)?;
        let callee = call.expr().ok_or("callee is missing")?;
        let cfg_call = caller
            .syntax()
            .descendants()
            .find_map(ast::MacroCall::cast)
            .ok_or("cfg invocation is missing")?;
        let cfg_id = semantics
            .to_def(&cfg_call)
            .ok_or("cfg invocation is unresolved")?;
        let expanded = semantics.expand(cfg_id);
        assert!(expanded.err.is_none(), "cfg expansion failed");
        eprintln!("cfg expanded syntax: {:?}", expanded.value);
        let cfg_expression = ast::CfgPredExpr::cast(expanded.value)
            .ok_or("typed cfg predicate expression is missing")?;
        let cfg_type = semantics
            .type_of_expr(&ast::Expr::CfgPredExpr(cfg_expression))
            .ok_or("cfg predicate type is missing")?;
        let mut diagnostics = Vec::new();
        model
            .krate
            .root_module(&model.db)
            .diagnostics(&model.db, &mut diagnostics, false);
        let result = semantics
            .type_of_expr(&ast::Expr::CallExpr(call.clone()))
            .ok_or("active call type is unavailable")?;

        assert!(!model.krate.cfg(&model.db).check_atom(&CfgAtom::KeyValue {
            key: Symbol::intern("feature"),
            value: Symbol::intern("alternate")
        }));
        model.assert_location(&semantics, callee.syntax(), 3)?;
        assert_eq!(definition.module(&model.db).krate(&model.db), model.krate);
        assert!(!cfg_type.original.contains_unknown());
        assert!(cfg_type.original.is_bool());
        assert!(semantics.resolve_expr_as_callable(&callee).is_none());
        assert!(result.original.contains_unknown());
        assert_eq!(diagnostics.len(), 1);
        assert_ident_diagnostic(&semantics, &diagnostics[0], &callee);
        eprintln!("cfg boolean active call type: {result:?}");
        Ok(())
    })
}

#[test]
fn a_disabled_cfg_item_stays_in_raw_source_without_an_active_body() -> ProbeResult {
    let model = load_model("lib.rs", &[])?;

    ra_ap_hir::attach_db(&model.db, || {
        let semantics = Semantics::new(&model.db);
        let source = model.source(&semantics);
        let conditional = declared_function(&source, "conditional_call");
        let call = free_call(&conditional)?;
        let diagnostics = model.diagnostics();
        let declarations = model.krate.root_module(&model.db).declarations(&model.db);
        let valid = declared_function(&source, "known_call");
        let valid_call = method_call(&valid, "debit");
        let expected = semantics
            .to_def(&declared_method(&source, "Wallet", "debit"))
            .ok_or("Wallet::debit is unresolved")?;

        assert!(!model.feature_enabled());
        assert!(semantics.to_def(&conditional).is_none());
        assert!(!declarations.iter().any(|item| matches!(item,
            ModuleDef::Function(function) if function.name(&model.db).as_str() == "conditional_call")));
        assert_eq!(semantics.resolve_method_call(&valid_call), Some(expected));
        assert_eq!(diagnostics.len(), 1);
        let AnyDiagnostic::InactiveCode(inactive) = &diagnostics[0] else {
            panic!("expected inactive item, found {:?}", diagnostics[0]);
        };
        assert_eq!(inactive.node, site(&semantics, conditional.syntax()));
        assert!(semantics.type_of_expr(&ast::Expr::CallExpr(call)).is_none());
        Ok(())
    })
}

#[test]
fn an_enabled_item_has_unresolved_targets_with_a_known_receiver_and_result() -> ProbeResult {
    let model = load_model("lib.rs", &["alternate"])?;

    ra_ap_hir::attach_db(&model.db, || {
        let semantics = Semantics::new(&model.db);
        let source = model.source(&semantics);
        let conditional = declared_function(&source, "conditional_call");
        let definition = semantics
            .to_def(&conditional)
            .ok_or("enabled item is inactive")?;
        let call = free_call(&conditional)?;
        let callee = call.expr().ok_or("missing function callee is absent")?;
        let missing_method = method_call(&conditional, "missing_debit");
        let receiver = semantics
            .type_of_expr(&missing_method.receiver().ok_or("receiver is missing")?)
            .ok_or("known receiver type is missing")?;
        let result = semantics
            .type_of_expr(&ast::Expr::MethodCallExpr(missing_method.clone()))
            .ok_or("contextual result type is missing")?;
        let wallet = semantics
            .to_def(&declared_struct(&source, "Wallet"))
            .ok_or("Wallet is unresolved")?;
        let other_target = semantics
            .to_def(&declared_method(&source, "Reserve", "missing_debit"))
            .ok_or("same-named control method is unresolved")?;
        let valid_call = method_call(&declared_function(&source, "known_call"), "debit");
        let expected_valid = semantics
            .to_def(&declared_method(&source, "Wallet", "debit"))
            .ok_or("valid control method is unresolved")?;
        let valid_type = semantics
            .type_of_expr(&ast::Expr::MethodCallExpr(valid_call.clone()))
            .ok_or("valid control type is missing")?;
        let diagnostics = model.diagnostics();
        let free_result = semantics
            .type_of_expr(&ast::Expr::CallExpr(call.clone()))
            .ok_or("active free-call type is unavailable")?;

        assert!(model.feature_enabled());
        assert_eq!(definition.module(&model.db).krate(&model.db), model.krate);
        assert!(semantics.resolve_expr_as_callable(&callee).is_none());
        assert!(free_result.original.contains_unknown());
        assert_eq!(semantics.resolve_method_call(&missing_method), None);
        assert_ne!(other_target, expected_valid);
        assert!(!receiver.original.contains_unknown());
        assert_eq!(
            receiver
                .original
                .as_reference_inner()
                .ok_or("receiver is not borrowed")?
                .as_adt(),
            Some(Adt::Struct(wallet))
        );
        assert!(!result.original.contains_unknown());
        assert_eq!(result.original, expected_valid.ret_type(&model.db));
        assert_eq!(
            semantics.resolve_method_call(&valid_call),
            Some(expected_valid)
        );
        assert!(!valid_type.original.contains_unknown());
        assert_eq!(valid_type.original, expected_valid.ret_type(&model.db));
        assert_eq!(diagnostics.len(), 2);
        let mut identifiers = 0;
        let mut methods = 0;
        for diagnostic in &diagnostics {
            match diagnostic {
                AnyDiagnostic::UnresolvedIdent(_) => {
                    assert_ident_diagnostic(&semantics, diagnostic, &callee);
                    identifiers += 1;
                }
                AnyDiagnostic::UnresolvedMethodCall(unresolved) => {
                    let actual = InFile::new(
                        unresolved.expr.file_id,
                        unresolved.expr.value.syntax_node_ptr(),
                    );
                    assert_eq!(actual, site(&semantics, missing_method.syntax()));
                    eprintln!("unresolved method diagnostic site: {actual:?}");
                    assert!(!unresolved.receiver.contains_unknown());
                    assert_eq!(unresolved.receiver, receiver.original);
                    methods += 1;
                }
                other => panic!("unexpected active diagnostic: {other:?}"),
            }
        }
        assert_eq!((identifiers, methods), (1, 1));
        eprintln!(
            "active missing function type: {:?}; missing method result: {:?}",
            semantics.type_of_expr(&ast::Expr::CallExpr(call)),
            result.original
        );
        Ok(())
    })
}

struct Model {
    db: RootDatabase,
    vfs: Vfs,
    krate: Crate,
    file: EditionedFileId,
    source_path: AbsPathBuf,
}

impl Model {
    fn source(&self, semantics: &Semantics<'_, RootDatabase>) -> ast::SourceFile {
        assert!(
            self.file.parse_errors(&self.db).is_none(),
            "source parse failed"
        );
        assert_eq!(
            self.vfs.file_path(self.file.file_id(&self.db)).as_path(),
            Some(self.source_path.as_path())
        );
        assert!(!self.krate.cfg(&self.db).check_atom(&flag("test")));
        assert!(!self.krate.cfg(&self.db).check_atom(&flag("rust_analyzer")));
        semantics.parse(self.file)
    }

    fn feature_enabled(&self) -> bool {
        self.krate.cfg(&self.db).check_atom(&CfgAtom::KeyValue {
            key: Symbol::intern("feature"),
            value: Symbol::intern("alternate"),
        })
    }

    fn diagnostics(&self) -> Vec<AnyDiagnostic<'_>> {
        let mut diagnostics = Vec::new();
        for module in self.krate.modules(&self.db) {
            module.diagnostics(&self.db, &mut diagnostics, false);
        }
        diagnostics
    }
}

fn load_model(source: &str, features: &[&str]) -> ProbeResult<Model> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("experiments/unresolved-cfg/fixture")
        .canonicalize()?;
    let source_path = AbsPathBuf::assert_utf8(directory.join(source).canonicalize()?);
    let output = Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()?;
    if !output.status.success() {
        return Err("rustc sysroot query failed".into());
    }
    let sysroot = PathBuf::from(String::from_utf8(output.stdout)?.trim());
    let source = sysroot.join("lib/rustlib/src/rust/library");
    if !source.is_dir() {
        return Err("matching rust-src is missing".into());
    }
    let target = PathBuf::from(std::env::var("CARGO_TARGET_DIR")?);
    if !target.is_absolute() {
        return Err("probe target must be absolute".into());
    }
    let config = CargoConfig {
        sysroot: Some(RustLibSource::Path(AbsPathBuf::assert_utf8(sysroot))),
        sysroot_src: Some(AbsPathBuf::assert_utf8(source)),
        set_test: false,
        cfg_overrides: CfgOverrides {
            global: CfgDiff::new(Vec::new(), vec![flag("rust_analyzer")]),
            ..CfgOverrides::default()
        },
        features: CargoFeatures::Selected {
            features: features.iter().map(|name| (*name).to_owned()).collect(),
            no_default_features: false,
        },
        metadata_extra_args: vec!["--locked".into(), "--offline".into()],
        target_dir_config: TargetDirectoryConfig::Directory(
            target
                .join("unresolved-model")
                .to_str()
                .ok_or("target is not UTF-8")?
                .into(),
        ),
        ..CargoConfig::default()
    };
    let manifest = ProjectManifest::discover_single(&AbsPathBuf::assert_utf8(directory))
        .map_err(|error| std::io::Error::other(format!("fixture manifest: {error:#}")))?;
    let workspace = ProjectWorkspace::load(manifest, &config, &|message| eprintln!("{message}"))
        .map_err(|error| std::io::Error::other(format!("fixture metadata: {error:#}")))?;
    if let ProjectWorkspaceKind::Cargo {
        error: Some(error), ..
    } = &workspace.kind
    {
        return Err(std::io::Error::other(format!("partial fixture metadata: {error:#}")).into());
    }
    if let Some(error) = workspace.sysroot.error() {
        return Err(std::io::Error::other(format!("fixture sysroot: {error}")).into());
    }
    let settings = LoadCargoConfig {
        load_out_dirs_from_check: false,
        with_proc_macro_server: ProcMacroServerChoice::None,
        prefill_caches: false,
        num_worker_threads: 1,
        proc_macro_processes: 1,
    };
    let (db, vfs, proc_client) = load_workspace(workspace, &config.extra_env, &settings)
        .map_err(|error| std::io::Error::other(format!("fixture load: {error:#}")))?;
    assert!(proc_client.is_none());
    let krate = crate_for_root(&db, &vfs, &source_path)?;
    let file = EditionedFileId::new(&db, krate.root_file(&db), krate.edition(&db));
    Ok(Model {
        db,
        vfs,
        krate,
        file,
        source_path,
    })
}

fn free_call(caller: &ast::Fn) -> ProbeResult<ast::CallExpr> {
    let mut calls = caller
        .syntax()
        .descendants()
        .filter_map(ast::CallExpr::cast);
    let call = calls.next().ok_or("source call is missing")?;
    assert!(calls.next().is_none(), "source free call is ambiguous");
    Ok(call)
}

fn assert_ident_diagnostic(
    semantics: &Semantics<'_, RootDatabase>,
    diagnostic: &AnyDiagnostic<'_>,
    callee: &ast::Expr,
) {
    let AnyDiagnostic::UnresolvedIdent(unresolved) = diagnostic else {
        panic!("expected unresolved identifier, found {diagnostic:?}");
    };
    let actual = InFile::new(
        unresolved.node.file_id,
        unresolved.node.value.0.syntax_node_ptr(),
    );
    assert_eq!(actual, site(semantics, callee.syntax()));
    eprintln!("unresolved identifier diagnostic site: {actual:?}");
    assert!(unresolved.node.value.1.is_none());
}

fn site(
    semantics: &Semantics<'_, RootDatabase>,
    node: &ra_ap_syntax::SyntaxNode,
) -> InFile<SyntaxNodePtr> {
    InFile::new(semantics.hir_file_for(node), SyntaxNodePtr::new(node))
}
