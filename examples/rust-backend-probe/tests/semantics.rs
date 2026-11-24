use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

use ra_ap_cfg::{CfgAtom, CfgDiff};
use ra_ap_hir::{
    Adt, AnyDiagnostic, Crate, EditionedFileId, ModuleDef, Mutability, PathResolution, Semantics,
};
use ra_ap_ide_db::RootDatabase;
use ra_ap_intern::Symbol;
use ra_ap_load_cargo::{LoadCargoConfig, ProcMacroServerChoice, load_workspace_at};
use ra_ap_project_model::{CargoConfig, CfgOverrides, RustLibSource, TargetDirectoryConfig};
use ra_ap_syntax::{AstNode, Edition, ast, ast::HasName};
use ra_ap_vfs::{AbsPathBuf, Vfs};

type ProbeResult<T = ()> = Result<T, Box<dyn Error>>;

#[test]
fn the_parameter_call_resolves_to_the_wallet_declaration() -> ProbeResult {
    let fixture = load_fixture()?;
    let semantics = Semantics::new(&fixture.db);
    let source = fixture.source(&semantics);
    let wallet = semantics
        .to_def(&declared_struct(&source, "Wallet"))
        .ok_or("Wallet has no definition")?;
    let wallet_debit = semantics
        .to_def(&declared_method(&source, "Wallet", "debit"))
        .ok_or("Wallet::debit has no definition")?;
    let caller = declared_method(&source, "PaymentService", "on_parameter");

    let call = method_call(&caller, "debit");
    let receiver = semantics
        .type_of_expr(&call.receiver().ok_or("S2 has no receiver")?)
        .ok_or("S2 receiver has no type")?;
    let target = semantics
        .resolve_method_call(&call)
        .ok_or("S2 target is unresolved")?;

    assert!(!receiver.original.contains_unknown());
    let (referent, mutability) = receiver
        .original
        .as_reference()
        .ok_or("S2 receiver is not a reference")?;
    assert_eq!(mutability, Mutability::Shared);
    assert_eq!(referent.as_adt(), Some(Adt::Struct(wallet)));
    assert_eq!(target, wallet_debit);
    fixture.assert_location(&semantics, call.syntax(), 13)?;
    Ok(())
}

#[test]
fn the_returned_chain_resolves_the_order_method_and_wallet_result() -> ProbeResult {
    let fixture = load_fixture()?;
    let semantics = Semantics::new(&fixture.db);
    let source = fixture.source(&semantics);
    let wallet = semantics
        .to_def(&declared_struct(&source, "Wallet"))
        .ok_or("Wallet has no definition")?;
    let wallet_debit = semantics
        .to_def(&declared_method(&source, "Wallet", "debit"))
        .ok_or("Wallet::debit has no definition")?;
    let order_wallet = semantics
        .to_def(&declared_method(&source, "Order", "wallet"))
        .ok_or("Order::wallet has no definition")?;
    let caller = declared_method(&source, "PaymentService", "on_returned_chain");

    let projection = method_call(&caller, "wallet");
    let debit = method_call(&caller, "debit");
    let result = semantics
        .type_of_expr(&ast::Expr::MethodCallExpr(projection.clone()))
        .ok_or("S5 wallet result has no type")?;
    let projection_target = semantics
        .resolve_method_call(&projection)
        .ok_or("S5 wallet target is unresolved")?;
    let debit_target = semantics
        .resolve_method_call(&debit)
        .ok_or("S5 debit target is unresolved")?;

    assert_eq!(projection_target, order_wallet);
    assert_eq!(debit_target, wallet_debit);
    assert_eq!(
        debit.receiver(),
        Some(ast::Expr::MethodCallExpr(projection.clone()))
    );
    assert!(!result.original.contains_unknown());
    let (referent, mutability) = result
        .original
        .as_reference()
        .ok_or("S5 wallet result is not a reference")?;
    assert_eq!(mutability, Mutability::Shared);
    assert_eq!(referent.as_adt(), Some(Adt::Struct(wallet)));
    fixture.assert_location(&semantics, projection.syntax(), 29)?;
    fixture.assert_location(&semantics, debit.syntax(), 29)?;
    Ok(())
}

#[test]
fn the_normal_library_excludes_test_items_without_removing_their_source() -> ProbeResult {
    let fixture = load_fixture()?;
    let semantics = Semantics::new(&fixture.db);
    let source = fixture.source(&semantics);
    let root = fixture.krate.root_module(&fixture.db);

    let cfg = fixture.krate.cfg(&fixture.db);
    let children: Vec<_> = root.children(&fixture.db).collect();
    let mut diagnostics = Vec::new();
    ra_ap_hir::attach_db(&fixture.db, || {
        root.diagnostics(&fixture.db, &mut diagnostics, false);
    });

    assert_eq!(fixture.krate.edition(&fixture.db), Edition::Edition2024);
    assert!(!cfg.check_atom(&flag("test")));
    assert!(!cfg.check_atom(&flag("rust_analyzer")));
    assert!(
        source
            .syntax()
            .descendants()
            .filter_map(ast::Module::cast)
            .any(|module| module.name().is_some_and(|name| name.text() == "tests"))
    );
    assert!(
        children.is_empty(),
        "unexpected active child modules: {children:?}"
    );
    assert_eq!(
        diagnostics.len(),
        1,
        "unexpected diagnostics: {diagnostics:?}"
    );
    assert!(
        matches!(&diagnostics[0], AnyDiagnostic::InactiveCode(_)),
        "unexpected diagnostic: {:?}",
        diagnostics[0]
    );
    Ok(())
}

#[test]
fn a_reexported_wallet_keeps_its_provider_identity_across_a_dependency_alias() -> ProbeResult {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/identity")
        .canonicalize()?;
    let fixture = load_fixture_at(&directory, "consumer/src/lib.rs")?;
    let semantics = Semantics::new(&fixture.db);
    let consumer = fixture.source(&semantics);
    let provider = crate_for_root(
        &fixture.db,
        &fixture.vfs,
        &AbsPathBuf::assert_utf8(directory.join("provider/src/lib.rs")),
    )?;
    let provider_source = child_module_source(&semantics, provider, "exposed")?;
    let consumer_source = child_module_source(&semantics, fixture.krate, "exposed")?;
    let provider_wallet = semantics
        .to_def(&declared_struct(&provider_source, "Wallet"))
        .ok_or("provider Wallet is unresolved")?;
    let consumer_wallet = semantics
        .to_def(&declared_struct(&consumer_source, "Wallet"))
        .ok_or("consumer Wallet is unresolved")?;
    let provider_debit = semantics
        .to_def(&declared_method(&provider_source, "Wallet", "debit"))
        .ok_or("provider debit is unresolved")?;
    let consumer_debit = semantics
        .to_def(&declared_method(&consumer_source, "Wallet", "debit"))
        .ok_or("consumer debit is unresolved")?;

    let imported_call = method_call(&declared_function(&consumer, "debit_imported"), "debit");
    let local_call = method_call(&declared_function(&consumer, "debit_local"), "debit");
    let imported_type = semantics
        .type_of_expr(
            &imported_call
                .receiver()
                .ok_or("imported receiver is missing")?,
        )
        .ok_or("imported receiver type is unresolved")?;
    let local_type = semantics
        .type_of_expr(&local_call.receiver().ok_or("local receiver is missing")?)
        .ok_or("local receiver type is unresolved")?;
    let imported_target = semantics
        .resolve_method_call(&imported_call)
        .ok_or("imported target is unresolved")?;
    let local_target = semantics
        .resolve_method_call(&local_call)
        .ok_or("local target is unresolved")?;

    assert!(!imported_type.original.contains_unknown());
    assert!(!local_type.original.contains_unknown());
    let (imported_referent, imported_mutability) = imported_type
        .original
        .as_reference()
        .ok_or("imported receiver is not a reference")?;
    let (local_referent, local_mutability) = local_type
        .original
        .as_reference()
        .ok_or("local receiver is not a reference")?;
    assert_eq!(imported_mutability, Mutability::Mut);
    assert_eq!(local_mutability, Mutability::Mut);
    assert_eq!(
        imported_referent.as_adt(),
        Some(Adt::Struct(provider_wallet))
    );
    assert_eq!(local_referent.as_adt(), Some(Adt::Struct(consumer_wallet)));
    assert_ne!(provider_wallet, consumer_wallet);
    assert_eq!(imported_target, provider_debit);
    assert_eq!(local_target, consumer_debit);
    assert_ne!(imported_target, local_target);
    assert_ne!(provider, fixture.krate);
    assert_eq!(
        provider_wallet.module(&fixture.db).krate(&fixture.db),
        provider
    );
    assert_eq!(
        imported_target.module(&fixture.db).krate(&fixture.db),
        provider
    );
    assert_eq!(
        consumer_wallet.module(&fixture.db).krate(&fixture.db),
        fixture.krate
    );
    assert_eq!(
        local_target.module(&fixture.db).krate(&fixture.db),
        fixture.krate
    );
    let dependency = fixture
        .krate
        .dependencies(&fixture.db)
        .into_iter()
        .find(|dependency| dependency.name.as_str() == "ledger")
        .ok_or("ledger dependency edge is missing")?;
    assert_eq!(dependency.krate, provider);
    let imported_definition = semantics
        .source(imported_target)
        .ok_or("imported target source is missing")?;
    let local_definition = semantics
        .source(local_target)
        .ok_or("local target source is missing")?;
    let imported_path = fixture.source_path_of(&semantics, imported_definition.value.syntax())?;
    let local_path = fixture.source_path_of(&semantics, local_definition.value.syntax())?;
    assert_eq!(
        imported_path,
        AbsPathBuf::assert_utf8(directory.join("provider/src/exposed.rs"))
    );
    assert_eq!(
        local_path,
        AbsPathBuf::assert_utf8(directory.join("consumer/src/exposed.rs"))
    );
    assert_eq!(imported_path.file_name(), local_path.file_name());
    assert_clean_library(&fixture, provider);
    assert_clean_library(&fixture, fixture.krate);
    Ok(())
}

#[test]
fn a_returned_unnameable_wallet_keeps_its_provider_identity() -> ProbeResult {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/identity")
        .canonicalize()?;
    let fixture = load_fixture_at(&directory, "consumer/src/lib.rs")?;
    let semantics = Semantics::new(&fixture.db);
    let consumer = fixture.source(&semantics);
    let provider = crate_for_root(
        &fixture.db,
        &fixture.vfs,
        &AbsPathBuf::assert_utf8(directory.join("provider/src/lib.rs")),
    )?;
    let provider_root = semantics.parse(EditionedFileId::new(
        &fixture.db,
        provider.root_file(&fixture.db),
        provider.edition(&fixture.db),
    ));
    let hidden_source = child_module_source(&semantics, provider, "hidden")?;
    let exposed_source = child_module_source(&semantics, provider, "exposed")?;
    let local_source = child_module_source(&semantics, fixture.krate, "exposed")?;
    let hidden_wallet = semantics
        .to_def(&declared_struct(&hidden_source, "Wallet"))
        .ok_or("hidden Wallet is unresolved")?;
    let exposed_wallet = semantics
        .to_def(&declared_struct(&exposed_source, "Wallet"))
        .ok_or("exposed Wallet is unresolved")?;
    let local_wallet = semantics
        .to_def(&declared_struct(&local_source, "Wallet"))
        .ok_or("local Wallet is unresolved")?;
    let hidden_debit = semantics
        .to_def(&declared_method(&hidden_source, "Wallet", "debit"))
        .ok_or("hidden debit is unresolved")?;
    let local_debit = semantics
        .to_def(&declared_method(&local_source, "Wallet", "debit"))
        .ok_or("local debit is unresolved")?;
    let provider_function = semantics
        .to_def(&declared_function(&provider_root, "hidden_wallet"))
        .ok_or("hidden_wallet function is unresolved")?;
    let caller = declared_function(&consumer, "debit_returned");

    let creation = caller
        .body()
        .ok_or("returned caller body is missing")?
        .syntax()
        .descendants()
        .find_map(ast::CallExpr::cast)
        .ok_or("returned value call is missing")?;
    let result = semantics
        .type_of_expr(&ast::Expr::CallExpr(creation.clone()))
        .ok_or("returned value type is unresolved")?;
    let ast::Expr::PathExpr(path) = creation
        .expr()
        .ok_or("returned function expression is missing")?
    else {
        return Err("returned function expression is not a path".into());
    };
    let creation_target =
        semantics.resolve_path(&path.path().ok_or("returned function path is missing")?);
    let debit = method_call(&caller, "debit");
    let receiver = semantics
        .type_of_expr(&debit.receiver().ok_or("returned receiver is missing")?)
        .ok_or("returned receiver type is unresolved")?;
    let target = semantics
        .resolve_method_call(&debit)
        .ok_or("returned debit target is unresolved")?;

    assert_eq!(
        creation_target,
        Some(PathResolution::Def(ModuleDef::Function(provider_function)))
    );
    assert!(!result.original.contains_unknown());
    assert!(!receiver.original.contains_unknown());
    assert_eq!(result.original.as_adt(), Some(Adt::Struct(hidden_wallet)));
    assert_eq!(receiver.original.as_adt(), Some(Adt::Struct(hidden_wallet)));
    assert_ne!(hidden_wallet, exposed_wallet);
    assert_ne!(hidden_wallet, local_wallet);
    assert_eq!(target, hidden_debit);
    assert_ne!(target, local_debit);
    assert_eq!(
        hidden_wallet.module(&fixture.db).krate(&fixture.db),
        provider
    );
    assert_eq!(target.module(&fixture.db).krate(&fixture.db), provider);
    assert_ne!(provider, fixture.krate);
    let actual_wallet = match receiver.original.as_adt() {
        Some(Adt::Struct(wallet)) => wallet,
        _ => return Err("returned receiver has no struct definition".into()),
    };
    let wallet_definition = semantics
        .source(actual_wallet)
        .ok_or("returned Wallet source is missing")?;
    let target_definition = semantics
        .source(target)
        .ok_or("returned debit source is missing")?;
    let expected_path = AbsPathBuf::assert_utf8(directory.join("provider/src/hidden.rs"));
    assert_eq!(
        fixture.source_path_of(&semantics, wallet_definition.value.syntax())?,
        expected_path
    );
    assert_eq!(
        fixture.source_path_of(&semantics, target_definition.value.syntax())?,
        expected_path
    );
    assert_clean_library(&fixture, provider);
    assert_clean_library(&fixture, fixture.krate);
    Ok(())
}

struct LoadedFixture {
    db: RootDatabase,
    vfs: Vfs,
    krate: Crate,
    source_path: AbsPathBuf,
}

impl LoadedFixture {
    fn source(&self, semantics: &Semantics<'_, RootDatabase>) -> ast::SourceFile {
        let file = EditionedFileId::new(
            &self.db,
            self.krate.root_file(&self.db),
            self.krate.edition(&self.db),
        );
        let errors = file.parse_errors(&self.db);
        assert!(errors.is_none(), "fixture source parse errors: {errors:?}");
        semantics.parse(file)
    }

    fn assert_location(
        &self,
        semantics: &Semantics<'_, RootDatabase>,
        node: &ra_ap_syntax::SyntaxNode,
        line: usize,
    ) -> ProbeResult {
        let range = semantics
            .original_range_opt(node)
            .ok_or("call has no original source range")?;
        let source = std::fs::read_to_string(&self.source_path)?;
        let offset = u32::from(range.range.start()) as usize;

        assert_eq!(
            self.vfs
                .file_path(range.file_id.file_id(&self.db))
                .as_path(),
            Some(self.source_path.as_path())
        );
        assert_eq!(
            source[..offset]
                .bytes()
                .filter(|byte| *byte == b'\n')
                .count()
                + 1,
            line
        );
        Ok(())
    }

    fn source_path_of(
        &self,
        semantics: &Semantics<'_, RootDatabase>,
        node: &ra_ap_syntax::SyntaxNode,
    ) -> ProbeResult<AbsPathBuf> {
        let range = semantics
            .original_range_opt(node)
            .ok_or("definition source range is missing")?;
        self.vfs
            .file_path(range.file_id.file_id(&self.db))
            .as_path()
            .map(|path| path.to_path_buf())
            .ok_or_else(|| "definition has no physical source path".into())
    }
}

fn load_fixture() -> ProbeResult<LoadedFixture> {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rust-scenarios")
        .canonicalize()?;
    load_fixture_at(&fixture_dir, "core.rs")
}

fn load_fixture_at(fixture_dir: &Path, source_file: &str) -> ProbeResult<LoadedFixture> {
    let source_path = AbsPathBuf::assert_utf8(fixture_dir.join(source_file));
    let sysroot = rustc_sysroot()?;
    let source = sysroot.join("lib/rustlib/src/rust/library");
    if !source.is_dir() {
        return Err(format!("matching rust-src is missing: {}", source.display()).into());
    }
    let target = PathBuf::from(std::env::var("CARGO_TARGET_DIR")?);
    if !target.is_absolute() {
        return Err("CARGO_TARGET_DIR must be an absolute probe output path".into());
    }
    let cargo_config = CargoConfig {
        sysroot: Some(RustLibSource::Path(AbsPathBuf::assert_utf8(sysroot))),
        sysroot_src: Some(AbsPathBuf::assert_utf8(source)),
        set_test: false,
        cfg_overrides: CfgOverrides {
            global: CfgDiff::new(Vec::new(), vec![flag("rust_analyzer")]),
            ..CfgOverrides::default()
        },
        metadata_extra_args: vec!["--locked".into(), "--offline".into()],
        target_dir_config: TargetDirectoryConfig::Directory(
            target
                .join("analyzed")
                .to_str()
                .ok_or("target path is not UTF-8")?
                .into(),
        ),
        ..CargoConfig::default()
    };
    let load_config = LoadCargoConfig {
        load_out_dirs_from_check: false,
        with_proc_macro_server: ProcMacroServerChoice::None,
        prefill_caches: false,
        num_worker_threads: 1,
        proc_macro_processes: 1,
    };
    let (db, vfs, proc_client) =
        load_workspace_at(fixture_dir, &cargo_config, &load_config, &|message| {
            eprintln!("{message}")
        })
        .map_err(|error| {
            std::io::Error::other(format!(
                "loading fixture {}: {error:#}",
                fixture_dir.display()
            ))
        })?;
    assert!(proc_client.is_none());
    let krate = crate_for_root(&db, &vfs, &source_path)?;
    Ok(LoadedFixture {
        db,
        vfs,
        krate,
        source_path,
    })
}

fn crate_for_root(db: &RootDatabase, vfs: &Vfs, source_path: &AbsPathBuf) -> ProbeResult<Crate> {
    let mut crates = Crate::all(db).into_iter().filter(|krate| {
        vfs.file_path(krate.root_file(db)).as_path() == Some(source_path.as_path())
    });
    let krate = crates.next().ok_or("fixture crate was not loaded")?;
    assert!(
        crates.next().is_none(),
        "multiple fixture crate contexts were loaded"
    );
    Ok(krate)
}

fn child_module_source(
    semantics: &Semantics<'_, RootDatabase>,
    krate: Crate,
    name: &str,
) -> ProbeResult<ast::SourceFile> {
    let module = krate
        .root_module(semantics.db)
        .children(semantics.db)
        .find(|module| {
            module
                .name(semantics.db)
                .is_some_and(|module_name| module_name.as_str() == name)
        })
        .ok_or("fixture child module is missing")?;
    ast::SourceFile::cast(semantics.module_definition_node(module).value)
        .ok_or_else(|| "fixture child module is not a source file".into())
}

fn assert_clean_library(fixture: &LoadedFixture, krate: Crate) {
    assert_eq!(krate.edition(&fixture.db), Edition::Edition2024);
    assert!(!krate.cfg(&fixture.db).check_atom(&flag("test")));
    assert!(!krate.cfg(&fixture.db).check_atom(&flag("rust_analyzer")));
    ra_ap_hir::attach_db(&fixture.db, || {
        for module in krate.modules(&fixture.db) {
            let mut diagnostics = Vec::new();
            module.diagnostics(&fixture.db, &mut diagnostics, false);
            assert!(
                diagnostics.is_empty(),
                "unexpected fixture diagnostics: {diagnostics:?}"
            );
        }
    });
}

fn declared_function(source: &ast::SourceFile, name: &str) -> ast::Fn {
    source
        .syntax()
        .children()
        .filter_map(ast::Fn::cast)
        .find(|function| {
            function
                .name()
                .is_some_and(|function_name| function_name.text() == name)
        })
        .expect("fixture function declaration is missing")
}

fn rustc_sysroot() -> ProbeResult<PathBuf> {
    let output = Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "rustc sysroot query failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(PathBuf::from(String::from_utf8(output.stdout)?.trim()))
}

fn flag(name: &str) -> CfgAtom {
    CfgAtom::Flag(Symbol::intern(name))
}

fn declared_struct(source: &ast::SourceFile, name: &str) -> ast::Struct {
    source
        .syntax()
        .children()
        .filter_map(ast::Struct::cast)
        .find(|item| {
            item.name()
                .is_some_and(|item_name| item_name.text() == name)
        })
        .expect("fixture struct declaration is missing")
}

fn declared_method(source: &ast::SourceFile, owner: &str, name: &str) -> ast::Fn {
    source
        .syntax()
        .children()
        .filter_map(ast::Impl::cast)
        .find(|implementation| {
            implementation
                .self_ty()
                .is_some_and(|ty| ty.syntax().text() == owner)
        })
        .expect("fixture impl declaration is missing")
        .syntax()
        .descendants()
        .filter_map(ast::Fn::cast)
        .find(|method| {
            method
                .name()
                .is_some_and(|method_name| method_name.text() == name)
        })
        .expect("fixture method declaration is missing")
}

fn method_call(caller: &ast::Fn, name: &str) -> ast::MethodCallExpr {
    caller
        .body()
        .expect("fixture caller has no body")
        .syntax()
        .descendants()
        .filter_map(ast::MethodCallExpr::cast)
        .find(|call| {
            call.name_ref()
                .is_some_and(|method_name| method_name.text() == name)
        })
        .expect("fixture method call is missing")
}
