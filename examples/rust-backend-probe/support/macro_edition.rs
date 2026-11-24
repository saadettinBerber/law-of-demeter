use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

use ra_ap_cfg::{CfgAtom, CfgDiff};
use ra_ap_hir::{Crate, EditionedFileId, Macro, Semantics};
use ra_ap_ide_db::RootDatabase;
use ra_ap_intern::Symbol;
use ra_ap_load_cargo::{LoadCargoConfig, ProcMacroServerChoice, load_workspace};
use ra_ap_project_model::{
    CargoConfig, CfgOverrides, ProjectManifest, ProjectWorkspace, ProjectWorkspaceKind,
    RustLibSource, TargetDirectoryConfig,
};
use ra_ap_syntax::{AstNode, Edition, ast, ast::HasName};
use ra_ap_vfs::AbsPathBuf;

type ProbeResult<T = ()> = Result<T, Box<dyn Error>>;

pub(super) fn the_model_preserves_editions_and_direct_or_forwarded_token_dispatch() -> ProbeResult {
    let model = load_model(false)?;

    ra_ap_hir::attach_db(&model.db, || {
        let semantics = Semantics::new(&model.db);
        let source = model.source(&semantics, model.consumer);
        let provider = model.source(&semantics, model.provider);
        let caller = function(&source, "callsite_count");
        let expected_macro = macro_definition(&semantics, &provider, "callsite_items")?;
        let reference = initializer(&caller, "reference")?;
        let reference = ast::MethodCallExpr::cast(reference.syntax().clone())
            .ok_or("reference is not a method call")?;

        let call = expanded_call(
            &model,
            &semantics,
            &caller,
            expected_macro,
            Edition::Edition2024,
        )?;
        let actual = semantics
            .type_of_expr(&ast::Expr::MethodCallExpr(call.clone()))
            .ok_or("macro result type is unresolved")?;
        let expected = semantics
            .type_of_expr(&ast::Expr::MethodCallExpr(reference.clone()))
            .ok_or("reference result type is unresolved")?;
        let macro_result = semantics
            .type_of_expr(&initializer(&caller, "iterator")?)
            .ok_or("original macro expression type is unresolved")?;
        assert!(!macro_result.original.contains_unknown());
        assert_eq!(actual.original, macro_result.original);
        let reference_receiver = reference
            .receiver()
            .ok_or("reference receiver is missing")?;
        eprintln!(
            "reference receiver: {:?}; adjustments: {:?}",
            semantics.type_of_expr(&reference_receiver),
            semantics.expr_adjustments(&reference_receiver)
        );
        let target = semantics
            .resolve_method_call(&call)
            .ok_or("macro target is unresolved")?;
        let expected_target = semantics
            .resolve_method_call(&reference)
            .ok_or("reference target is unresolved")?;

        assert_context(&model, false);
        assert!(!actual.original.contains_unknown());
        assert!(!expected.original.contains_unknown());
        assert_eq!(actual.original, expected.original);
        assert_eq!(target, expected_target);
        assert_provider_dispatch(&semantics, &provider)?;
        assert!(
            !model
                .consumer
                .root_module(&model.db)
                .children(&model.db)
                .any(|module| {
                    module
                        .name(&model.db)
                        .is_some_and(|name| name.as_str() == "tests")
                })
        );
        assert!(
            source
                .syntax()
                .descendants()
                .filter_map(ast::Module::cast)
                .any(|module| { module.name().is_some_and(|name| name.text() == "tests") })
        );
        let mut diagnostics = Vec::new();
        model
            .consumer
            .root_module(&model.db)
            .diagnostics(&model.db, &mut diagnostics, false);
        eprintln!("normal model diagnostics: {diagnostics:?}");
        let unexpected: Vec<_> = diagnostics
            .iter()
            .filter(|diagnostic| !matches!(diagnostic, ra_ap_hir::AnyDiagnostic::InactiveCode(_)))
            .collect();
        assert!(
            unexpected.is_empty(),
            "unexpected normal diagnostics: {unexpected:?}"
        );
        Ok(())
    })
}

pub(super) fn a_definition_token_uses_its_legacy_edition_for_array_method_lookup() -> ProbeResult {
    let model = load_model(true)?;

    ra_ap_hir::attach_db(&model.db, || {
        let semantics = Semantics::new(&model.db);
        let source = model.source(&semantics, model.consumer);
        let provider = model.source(&semantics, model.provider);
        let caller = function(&source, "definition_count");
        let expected_macro = macro_definition(&semantics, &provider, "definition_items")?;
        let reference = initializer(&caller, "reference")?;
        let reference = ast::MethodCallExpr::cast(reference.syntax().clone())
            .ok_or("reference is not a method call")?;
        let compiler_test = function(&source, "a_definition_token_keeps_reference_identity");
        let test_definition = semantics
            .to_def(&compiler_test)
            .ok_or("compiler test is inactive")?;

        let call = expanded_call(
            &model,
            &semantics,
            &caller,
            expected_macro,
            Edition::Edition2018,
        )?;
        let actual = semantics
            .type_of_expr(&ast::Expr::MethodCallExpr(call.clone()))
            .ok_or("macro result type is unresolved")?;
        let expected = semantics
            .type_of_expr(&ast::Expr::MethodCallExpr(reference.clone()))
            .ok_or("reference result type is unresolved")?;
        let macro_result = semantics
            .type_of_expr(&initializer(&caller, "iterator")?)
            .ok_or("original macro expression type is unresolved")?;
        assert!(!macro_result.original.contains_unknown());
        assert_eq!(actual.original, macro_result.original);
        let reference_receiver = reference
            .receiver()
            .ok_or("reference receiver is missing")?;
        eprintln!(
            "reference receiver: {:?}; adjustments: {:?}",
            semantics.type_of_expr(&reference_receiver),
            semantics.expr_adjustments(&reference_receiver)
        );
        let target = semantics
            .resolve_method_call(&call)
            .ok_or("macro target is unresolved")?;
        let expected_target = semantics
            .resolve_method_call(&reference)
            .ok_or("reference target is unresolved")?;
        let mut diagnostics = Vec::new();
        model
            .consumer
            .root_module(&model.db)
            .diagnostics(&model.db, &mut diagnostics, false);
        eprintln!("test model diagnostics: {diagnostics:?}");
        eprintln!(
            "legacy result: {:?}; borrowed reference result: {:?}",
            actual.original, expected.original
        );
        eprintln!("legacy target: {target:?}; borrowed reference target: {expected_target:?}");

        assert_context(&model, true);
        assert!(test_definition.is_test(&model.db));
        assert!(!actual.original.contains_unknown());
        assert!(!expected.original.contains_unknown());
        let returns_borrowed_iterator = actual.original == expected.original;
        let selects_borrowed_implementation = target == expected_target;
        let has_no_type_errors = diagnostics
            .iter()
            .all(|diagnostic| matches!(diagnostic, ra_ap_hir::AnyDiagnostic::InactiveCode(_)));
        assert!(
            returns_borrowed_iterator && selects_borrowed_implementation && has_no_type_errors,
            "legacy token: borrowed result={returns_borrowed_iterator}, borrowed target={selects_borrowed_implementation}, no type errors={has_no_type_errors}"
        );
        Ok(())
    })
}

fn assert_provider_dispatch(
    semantics: &Semantics<'_, RootDatabase>,
    provider: &ast::SourceFile,
) -> ProbeResult {
    let caller = function(provider, "direct_count");
    let reference = initializer(&caller, "reference")?;
    let direct = initializer(&caller, "iterator")?;
    let direct_call = ast::MethodCallExpr::cast(direct.syntax().clone())
        .ok_or("provider direct call is missing")?;
    let reference_call = ast::MethodCallExpr::cast(reference.syntax().clone())
        .ok_or("provider reference call is missing")?;

    let actual = semantics
        .type_of_expr(&direct)
        .ok_or("provider direct type is unresolved")?;
    let expected = semantics
        .type_of_expr(&reference)
        .ok_or("provider reference type is unresolved")?;
    let target = semantics
        .resolve_method_call(&direct_call)
        .ok_or("provider direct target is unresolved")?;
    let expected_target = semantics
        .resolve_method_call(&reference_call)
        .ok_or("provider reference target is unresolved")?;
    let receiver = direct_call
        .receiver()
        .ok_or("provider direct receiver is missing")?;
    let receiver_type = semantics
        .type_of_expr(&receiver)
        .ok_or("provider receiver type is unresolved")?;
    eprintln!(
        "provider direct result: {:?}; receiver: {receiver_type:?}; adjustments: {:?}",
        actual.original,
        semantics.expr_adjustments(&receiver)
    );

    assert!(!actual.original.contains_unknown());
    assert!(!expected.original.contains_unknown());
    assert_eq!(actual.original, expected.original);
    assert_eq!(target, expected_target);
    assert!(
        receiver_type
            .adjusted
            .as_ref()
            .and_then(|ty| ty.as_reference_inner())
            .is_some()
    );
    Ok(())
}

struct Model {
    db: RootDatabase,
    provider: Crate,
    consumer: Crate,
}

impl Model {
    fn source(&self, semantics: &Semantics<'_, RootDatabase>, krate: Crate) -> ast::SourceFile {
        let file =
            EditionedFileId::new(&self.db, krate.root_file(&self.db), krate.edition(&self.db));
        assert!(
            file.parse_errors(&self.db).is_none(),
            "source parse errors: {:?}",
            file.parse_errors(&self.db)
        );
        semantics.parse(file)
    }
}

fn expanded_call<'db>(
    model: &'db Model,
    semantics: &Semantics<'db, RootDatabase>,
    caller: &ast::Fn,
    expected_macro: Macro,
    token_edition: Edition,
) -> ProbeResult<ast::MethodCallExpr> {
    let value = initializer(caller, "iterator")?;
    let invocation = value
        .syntax()
        .descendants()
        .find_map(ast::MacroCall::cast)
        .ok_or("macro invocation is missing")?;
    let definition = semantics
        .resolve_macro_call(&invocation)
        .ok_or("macro definition is unresolved")?;
    assert_eq!(definition, expected_macro);
    assert_eq!(
        definition.module(&model.db).krate(&model.db),
        model.provider
    );
    let id = semantics
        .to_def(&invocation)
        .ok_or("macro call identity is unresolved")?;
    let expansion = semantics.expand(id);
    assert!(expansion.err.is_none(), "macro expansion failed");
    let mut calls = expansion
        .value
        .descendants()
        .filter_map(ast::MethodCallExpr::cast);
    let call = calls.next().ok_or("expanded method is missing")?;
    assert!(
        calls.next().is_none(),
        "expansion has more than one method call"
    );
    assert_eq!(
        semantics.hir_file_for(call.syntax()),
        ra_ap_hir::HirFileId::MacroFile(id)
    );
    let name = call.name_ref().ok_or("expanded method name is missing")?;
    let span = id
        .expansion_span_map(&model.db)
        .span_at(name.syntax().text_range().start());
    eprintln!(
        "method token: context edition {:?}, anchor edition {:?}",
        span.ctx.edition(&model.db),
        span.anchor.file_id.edition()
    );
    assert_eq!(span.ctx.edition(&model.db), token_edition);
    assert_eq!(span.anchor.file_id.edition(), token_edition);
    let receiver = call.receiver().ok_or("expanded receiver is missing")?;
    let receiver_type = semantics
        .type_of_expr(&receiver)
        .ok_or("receiver type is unresolved")?;
    assert!(!receiver_type.original.contains_unknown());
    eprintln!(
        "receiver: {receiver_type:?}; adjustments: {:?}",
        semantics.expr_adjustments(&receiver)
    );
    let callable = semantics
        .resolve_method_call_as_callable(&call)
        .ok_or("expanded callable is unresolved")?;
    eprintln!(
        "bound self: {:?}; callable result: {:?}",
        callable.receiver_param(&model.db),
        callable.return_type()
    );
    Ok(call)
}

fn function(source: &ast::SourceFile, name: &str) -> ast::Fn {
    source
        .syntax()
        .descendants()
        .filter_map(ast::Fn::cast)
        .find(|function| function.name().is_some_and(|item| item.text() == name))
        .expect("fixture function is missing")
}

fn initializer(caller: &ast::Fn, name: &str) -> ProbeResult<ast::Expr> {
    let binding = caller
        .syntax()
        .descendants()
        .filter_map(ast::LetStmt::cast)
        .find(|binding| {
            binding
                .pat()
                .is_some_and(|pattern| pattern.syntax().text() == name)
        })
        .ok_or("fixture binding is missing")?;
    assert!(
        binding.ty().is_none(),
        "macro/control binding must be unannotated"
    );
    binding
        .initializer()
        .ok_or_else(|| "fixture initializer is missing".into())
}

fn macro_definition(
    semantics: &Semantics<'_, RootDatabase>,
    source: &ast::SourceFile,
    name: &str,
) -> ProbeResult<Macro> {
    let definition = source
        .syntax()
        .descendants()
        .filter_map(ast::MacroRules::cast)
        .find(|item| item.name().is_some_and(|item| item.text() == name))
        .ok_or("provider macro declaration is missing")?;
    semantics
        .to_macro_def(&ast::Macro::MacroRules(definition))
        .ok_or_else(|| "provider macro definition is unresolved".into())
}

fn assert_context(model: &Model, test: bool) {
    assert_ne!(model.provider, model.consumer);
    assert_eq!(model.provider.edition(&model.db), Edition::Edition2018);
    assert_eq!(model.consumer.edition(&model.db), Edition::Edition2024);
    for krate in [model.provider, model.consumer] {
        assert_eq!(krate.cfg(&model.db).check_atom(&flag("test")), test);
        assert!(!krate.cfg(&model.db).check_atom(&flag("rust_analyzer")));
    }
}

fn load_model(test: bool) -> ProbeResult<Model> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/macro-edition")
        .canonicalize()?;
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
    let target = PathBuf::from(std::env::var("CARGO_TARGET_DIR")?)
        .join(format!("analyzed-macro-edition-{test}"));
    if !target.is_absolute() {
        return Err("CARGO_TARGET_DIR must be absolute".into());
    }
    let config = CargoConfig {
        sysroot: Some(RustLibSource::Path(AbsPathBuf::assert_utf8(sysroot))),
        sysroot_src: Some(AbsPathBuf::assert_utf8(source)),
        set_test: test,
        cfg_overrides: CfgOverrides {
            global: CfgDiff::new(Vec::new(), vec![flag("rust_analyzer")]),
            ..CfgOverrides::default()
        },
        metadata_extra_args: vec!["--locked".into(), "--offline".into()],
        target_dir_config: TargetDirectoryConfig::Directory(
            target.to_str().ok_or("target is not UTF-8")?.into(),
        ),
        ..CargoConfig::default()
    };
    let manifest = ProjectManifest::discover_single(&AbsPathBuf::assert_utf8(directory.clone()))?;
    let workspace = ProjectWorkspace::load(manifest, &config, &|message| eprintln!("{message}"))?;
    if let ProjectWorkspaceKind::Cargo {
        error: Some(error), ..
    } = &workspace.kind
    {
        return Err(std::io::Error::other(format!("partial metadata: {error:#}")).into());
    }
    if let Some(error) = workspace.sysroot.error() {
        return Err(std::io::Error::other(format!("sysroot: {error}")).into());
    }
    let settings = LoadCargoConfig {
        load_out_dirs_from_check: false,
        with_proc_macro_server: ProcMacroServerChoice::None,
        prefill_caches: false,
        num_worker_threads: 1,
        proc_macro_processes: 1,
    };
    let (db, vfs, client) = load_workspace(workspace, &config.extra_env, &settings)?;
    assert!(client.is_none());
    let find_crate = |path: &Path| -> ProbeResult<Crate> {
        let path = AbsPathBuf::assert_utf8(path.canonicalize()?);
        let mut matches = Crate::all(&db)
            .into_iter()
            .filter(|krate| vfs.file_path(krate.root_file(&db)).as_path() == Some(path.as_path()));
        let krate = matches.next().ok_or("fixture crate is missing")?;
        assert!(
            matches.next().is_none(),
            "fixture crate context is ambiguous"
        );
        Ok(krate)
    };
    let provider = find_crate(&directory.join("provider/src/lib.rs"))?;
    let consumer = find_crate(&directory.join("consumer/src/lib.rs"))?;
    Ok(Model {
        db,
        provider,
        consumer,
    })
}

fn flag(name: &str) -> CfgAtom {
    CfgAtom::Flag(Symbol::intern(name))
}

pub(super) fn check_call_orders() -> ProbeResult {
    order_queries::check_call_orders()
}

#[path = "../experiments/edition-patch/order_queries.rs"]
mod order_queries;
