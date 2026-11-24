use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

use ra_ap_cfg::{CfgAtom, CfgDiff};
use ra_ap_hir::{Adt, AnyDiagnostic, Crate, EditionedFileId, Mutability, Semantics};
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
        assert!(errors.is_none(), "scenario source parse errors: {errors:?}");
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
}

fn load_fixture() -> ProbeResult<LoadedFixture> {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rust-scenarios")
        .canonicalize()?;
    let source_path = AbsPathBuf::assert_utf8(fixture_dir.join("core.rs"));
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
        load_workspace_at(&fixture_dir, &cargo_config, &load_config, &|message| {
            eprintln!("{message}")
        })
        .map_err(|error| std::io::Error::other(format!("loading scenario fixture: {error:#}")))?;
    assert!(proc_client.is_none());
    let mut crates = Crate::all(&db).into_iter().filter(|krate| {
        vfs.file_path(krate.root_file(&db)).as_path() == Some(source_path.as_path())
    });
    let krate = crates.next().ok_or("scenario crate was not loaded")?;
    assert!(
        crates.next().is_none(),
        "multiple scenario crate contexts were loaded"
    );
    Ok(LoadedFixture {
        db,
        vfs,
        krate,
        source_path,
    })
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
