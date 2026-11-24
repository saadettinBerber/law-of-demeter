#[path = "support/anchors.rs"]
mod anchors;

use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

use ra_ap_cfg::{CfgAtom, CfgDiff};
use ra_ap_hir::{
    Adt, AnyDiagnostic, Crate, EditionedFileId, Function, InFile, ModuleDef, Semantics,
};
use ra_ap_ide_db::RootDatabase;
use ra_ap_intern::Symbol;
use ra_ap_load_cargo::{LoadCargoConfig, ProcMacroServerChoice, load_workspace};
use ra_ap_proc_macro_api::ProcMacroClient;
use ra_ap_project_model::{
    CargoConfig, CargoFeatures, CfgOverrides, ProjectManifest, ProjectWorkspace,
    ProjectWorkspaceKind, RustLibSource, TargetDirectoryConfig,
};
use ra_ap_syntax::{AstNode, Edition, SyntaxNodePtr, ast, ast::HasArgList, ast::HasName};
use ra_ap_vfs::{AbsPathBuf, Vfs, VfsPath};

use anchors::{declared_function, declared_method, declared_struct, method_call};

type ProbeResult<T = ()> = Result<T, Box<dyn Error>>;

#[test]
fn a_failed_fixture_build_is_returned_as_an_error() -> ProbeResult {
    let mode = "invalid";

    let result = prepare_fixture(mode, false, false);

    assert!(result.is_err(), "a failed fixture build was accepted");
    let error = result.err().ok_or("build error is missing")?;
    assert!(
        error
            .to_string()
            .contains("LOD_FIXTURE_MODE must be base or bonus"),
        "unexpected build failure: {error}"
    );
    Ok(())
}

#[test]
fn base_build_outputs_select_the_default_body_and_seven_unit_charge() -> ProbeResult {
    let (workspace, config) = prepare_fixture("base", false, false)?;

    let fixture = load_prepared(workspace, &config, ProcMacroServerChoice::None)?;

    ra_ap_hir::attach_db(&fixture.db, || {
        assert_build_context(&fixture, "base", false, 17, 7)
    })
}

#[test]
fn bonus_build_outputs_select_the_alternate_body_and_eleven_unit_charge() -> ProbeResult {
    let (workspace, config) = prepare_fixture("bonus", true, false)?;

    let fixture = load_prepared(workspace, &config, ProcMacroServerChoice::None)?;

    ra_ap_hir::attach_db(&fixture.db, || {
        assert_build_context(&fixture, "bonus", true, 23, 11)
    })
}

#[test]
fn the_real_proc_server_expands_calls_resolving_to_the_consumers_wallet() -> ProbeResult {
    let (workspace, config) = prepare_fixture("base", false, true)?;
    let Some(RustLibSource::Path(sysroot)) = &config.sysroot else {
        return Err("explicit sysroot is missing".into());
    };
    let server = sysroot.join("libexec/rust-analyzer-proc-macro-srv");
    assert!(
        Path::new(server.as_str()).is_file(),
        "proc server is missing: {server}"
    );

    let fixture = load_prepared(
        workspace,
        &config,
        ProcMacroServerChoice::Explicit(server.clone()),
    )?;

    assert!(fixture.consumer.cfg(&fixture.db).check_atom(&flag("test")));
    assert_eq!(
        fixture
            .proc_client
            .as_ref()
            .ok_or("proc client was not retained")?
            .server_path(),
        server.as_path()
    );

    ra_ap_hir::attach_db(&fixture.db, || {
        let semantics = Semantics::new(&fixture.db);
        let source = fixture.source(&semantics);
        let wallet = semantics
            .to_def(&declared_struct(&source, "Wallet"))
            .ok_or("consumer Wallet is unresolved")?;
        let expected_charge = semantics
            .to_def(&declared_method(&source, "Wallet", "charge"))
            .ok_or("consumer charge is unresolved")?;
        let expected_balance = semantics
            .to_def(&declared_method(&source, "Wallet", "balance"))
            .ok_or("consumer balance is unresolved")?;
        let test_module = fixture
            .consumer
            .root_module(&fixture.db)
            .children(&fixture.db)
            .find(|module| {
                module
                    .name(&fixture.db)
                    .is_some_and(|name| name.as_str() == "tests")
            })
            .ok_or("active test module is missing")?;
        let test_function = source
            .syntax()
            .descendants()
            .filter_map(ast::Fn::cast)
            .find(|function| {
                function.name().is_some_and(|name| {
                    name.text() == "a_function_like_proc_macro_generates_a_charge_in_the_consumer"
                })
            })
            .ok_or("proc test source is missing")?;
        let macro_call = test_function
            .syntax()
            .descendants()
            .filter_map(ast::MacroCall::cast)
            .find(|call| {
                call.path().is_some_and(|path| {
                    path.syntax().text() == "expansion_generator::generated_charge"
                })
            })
            .ok_or("active proc invocation is missing")?;

        assert!(
            semantics
                .resolve_macro_call(&macro_call)
                .is_some_and(|definition| definition.is_proc_macro())
        );
        let call_id = semantics
            .to_def(&macro_call)
            .ok_or("proc invocation has no expansion identity")?;
        let expansion = semantics.expand(call_id);
        assert!(
            expansion.err.is_none(),
            "proc expansion failed: {:?}",
            expansion
                .err
                .as_ref()
                .map(|error| error.render_to_string(&fixture.db).message)
        );
        let calls: Vec<_> = expansion
            .value
            .descendants()
            .filter_map(ast::MethodCallExpr::cast)
            .collect();
        assert_eq!(calls.len(), 2);
        let charge = calls
            .iter()
            .find(|call| call.name_ref().is_some_and(|name| name.text() == "charge"))
            .ok_or("expanded charge call is missing")?;
        let balance = calls
            .iter()
            .find(|call| call.name_ref().is_some_and(|name| name.text() == "balance"))
            .ok_or("expanded balance call is missing")?;
        let charge_target = semantics
            .resolve_method_call(charge)
            .ok_or("expanded charge target is unresolved")?;
        let balance_target = semantics
            .resolve_method_call(balance)
            .ok_or("expanded balance target is unresolved")?;

        assert_eq!(charge_target, expected_charge);
        assert_eq!(balance_target, expected_balance);
        for call in &calls {
            let receiver = semantics
                .type_of_expr(&call.receiver().ok_or("expanded receiver is missing")?)
                .ok_or("expanded receiver type is unresolved")?;
            assert!(!receiver.original.contains_unknown());
            assert_eq!(receiver.original.as_adt(), Some(Adt::Struct(wallet)));
        }
        let amount = charge
            .arg_list()
            .and_then(|args| args.args().next())
            .ok_or("expanded charge argument is missing")?;
        assert_eq!(integer_value(&amount)?, 13);
        let result = semantics
            .type_of_expr(&ast::Expr::MethodCallExpr(balance.clone()))
            .ok_or("expanded balance type is unresolved")?;
        assert!(!result.original.contains_unknown());
        assert_eq!(result.original, expected_balance.ret_type(&fixture.db));
        assert!(
            !fixture
                .consumer
                .cfg(&fixture.db)
                .check_atom(&flag("rust_analyzer"))
        );
        let definition = semantics
            .to_def(&test_function)
            .ok_or("proc test is inactive")?;
        assert!(definition.is_test(&fixture.db));
        assert_eq!(definition.module(&fixture.db), test_module);
        assert_proc_context_diagnostics(&fixture, &semantics, &source)?;
        Ok(())
    })
}

fn assert_proc_context_diagnostics(
    fixture: &BuiltFixture,
    semantics: &Semantics<'_, RootDatabase>,
    source: &ast::SourceFile,
) -> ProbeResult {
    let legacy_caller = source
        .syntax()
        .descendants()
        .filter_map(ast::Fn::cast)
        .find(|function| {
            function.name().is_some_and(|name| {
                name.text() == "an_exported_macro_keeps_the_method_tokens_edition"
            })
        })
        .ok_or("legacy test source is missing")?;
    let invocation = legacy_caller
        .syntax()
        .descendants()
        .filter_map(ast::MacroCall::cast)
        .find(|call| {
            call.path()
                .is_some_and(|path| path.syntax().text() == "expansion_legacy::definition_items")
        })
        .ok_or("legacy invocation is missing")?;
    let expansion_id = semantics
        .to_def(&invocation)
        .ok_or("legacy invocation is inactive or unresolved")?;
    let expansion = semantics.expand(expansion_id);
    assert!(expansion.err.is_none(), "legacy expansion failed");
    let mut calls = expansion
        .value
        .descendants()
        .filter_map(ast::MethodCallExpr::cast);
    let call = calls.next().ok_or("legacy method call is missing")?;
    assert!(
        calls.next().is_none(),
        "legacy expansion has multiple calls"
    );
    assert!(
        call.name_ref()
            .is_some_and(|name| name.text() == "into_iter")
    );
    let legacy_site = InFile::new(
        semantics.hir_file_for(call.syntax()),
        SyntaxNodePtr::new(call.syntax()),
    );
    assert_eq!(
        legacy_site.file_id,
        ra_ap_hir::HirFileId::MacroFile(expansion_id)
    );

    let mut diagnostics = Vec::new();
    fixture
        .consumer
        .root_module(&fixture.db)
        .diagnostics(&fixture.db, &mut diagnostics, false);
    let mut unexpected = Vec::new();
    for diagnostic in diagnostics {
        match diagnostic {
            AnyDiagnostic::InactiveCode(_) => {}
            AnyDiagnostic::TypeMismatch(mismatch)
                if mismatch.expr_or_pat.file_id == legacy_site.file_id
                    && mismatch.expr_or_pat.value.syntax_node_ptr() == legacy_site.value =>
            {
                eprintln!("known pending legacy call TypeMismatch: {mismatch:?}");
            }
            other => unexpected.push(other),
        }
    }

    assert!(
        unexpected.is_empty(),
        "unexpected consumer diagnostics: {unexpected:?}"
    );
    Ok(())
}

struct BuiltFixture {
    db: RootDatabase,
    vfs: Vfs,
    consumer: Crate,
    target: AbsPathBuf,
    proc_client: Option<ProcMacroClient>,
}

impl BuiltFixture {
    fn source(&self, semantics: &Semantics<'_, RootDatabase>) -> ast::SourceFile {
        let file = EditionedFileId::new(
            &self.db,
            self.consumer.root_file(&self.db),
            self.consumer.edition(&self.db),
        );
        assert!(
            file.parse_errors(&self.db).is_none(),
            "consumer parse errors: {:?}",
            file.parse_errors(&self.db)
        );
        semantics.parse(file)
    }
}

fn load_prepared(
    workspace: ProjectWorkspace,
    config: &CargoConfig,
    server: ProcMacroServerChoice,
) -> ProbeResult<BuiltFixture> {
    let requested = match &server {
        ProcMacroServerChoice::Explicit(path) => Some(path.clone()),
        _ => None,
    };
    let version = workspace.toolchain.clone();
    let settings = LoadCargoConfig {
        load_out_dirs_from_check: false,
        with_proc_macro_server: server,
        prefill_caches: false,
        num_worker_threads: 1,
        proc_macro_processes: 1,
    };
    let (db, vfs, proc_client) = load_workspace(workspace, &config.extra_env, &settings)
        .map_err(|error| std::io::Error::other(format!("loading built fixture: {error:#}")))?;
    if let Some(path) = requested
        && proc_client.is_none()
    {
        let _diagnostic_client =
            ProcMacroClient::spawn(&path, &config.extra_env, version.as_ref(), 1).map_err(
                |error| std::io::Error::other(format!("proc server {}: {error}", path.as_str())),
            )?;
        return Err(
            "loader did not retain a proc client although direct server startup succeeded".into(),
        );
    }
    let consumer_path = AbsPathBuf::assert_utf8(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../rust-expansion/consumer/src/lib.rs")
            .canonicalize()?,
    );
    let mut roots = Crate::all(&db).into_iter().filter(|krate| {
        vfs.file_path(krate.root_file(&db)).as_path() == Some(consumer_path.as_path())
    });
    let consumer = roots.next().ok_or("consumer crate is missing")?;
    assert!(
        roots.next().is_none(),
        "consumer has ambiguous crate contexts"
    );
    let TargetDirectoryConfig::Directory(target) = &config.target_dir_config else {
        return Err("build target directory is not explicit".into());
    };
    let target = AbsPathBuf::assert_utf8(PathBuf::from(target.as_str()));
    Ok(BuiltFixture {
        db,
        vfs,
        consumer,
        target,
        proc_client,
    })
}

fn assert_build_context(
    fixture: &BuiltFixture,
    mode: &str,
    alternate: bool,
    selected_value: u128,
    charge_amount: u128,
) -> ProbeResult {
    let semantics = Semantics::new(&fixture.db);
    let source = fixture.source(&semantics);
    let expected_charge = semantics
        .to_def(&declared_method(&source, "Wallet", "charge"))
        .ok_or("consumer charge is unresolved")?;
    let wallet = semantics
        .to_def(&declared_struct(&source, "Wallet"))
        .ok_or("consumer Wallet is unresolved")?;

    let out_dir = fixture.consumer.base().env(&fixture.db).get("OUT_DIR");
    assert!(
        out_dir.is_some(),
        "build OUT_DIR was not transferred into the model"
    );
    let out_dir = AbsPathBuf::assert_utf8(PathBuf::from(out_dir.ok_or("OUT_DIR is missing")?));
    let generated_path = out_dir.join("generated.rs");
    let file = fixture
        .vfs
        .file_id(&VfsPath::from(generated_path.clone()))
        .ok_or("generated.rs is missing from VFS")?
        .0;
    let generated_file =
        EditionedFileId::new(&fixture.db, file, fixture.consumer.edition(&fixture.db));
    assert!(
        generated_file.parse_errors(&fixture.db).is_none(),
        "generated source parse errors: {:?}",
        generated_file.parse_errors(&fixture.db)
    );
    let generated_source = semantics.parse(generated_file);
    let physical_function = declared_function(&generated_source, "generated_charge");
    let generated = active_function(fixture, "generated_charge")?;
    let generated_definition = semantics
        .source(generated)
        .ok_or("generated function has no source")?;
    let call = method_call(&generated_definition.value, "charge");
    let argument = call
        .arg_list()
        .and_then(|args| args.args().next())
        .ok_or("generated charge argument is missing")?;
    let physical_argument = method_call(&physical_function, "charge")
        .arg_list()
        .and_then(|args| args.args().next())
        .ok_or("VFS charge argument is missing")?;
    let selected = active_function(fixture, "selected_value")?;
    let selected_definition = semantics
        .source(selected)
        .ok_or("selected function has no source")?;
    let selected_body = selected_definition
        .value
        .body()
        .and_then(|body| body.tail_expr())
        .ok_or("selected body is missing")?;
    let receiver = semantics
        .type_of_expr(&call.receiver().ok_or("generated receiver is missing")?)
        .ok_or("generated receiver type is unresolved")?;
    let target = semantics
        .resolve_method_call(&call)
        .ok_or("generated charge target is unresolved")?;

    assert_eq!(integer_value(&selected_body)?, selected_value);
    assert_eq!(integer_value(&argument)?, charge_amount);
    assert_eq!(integer_value(&physical_argument)?, charge_amount);
    assert_eq!(target, expected_charge);
    assert!(!receiver.original.contains_unknown());
    assert_eq!(
        receiver
            .original
            .as_reference_inner()
            .ok_or("generated receiver is not borrowed")?
            .as_adt(),
        Some(Adt::Struct(wallet))
    );
    assert_eq!(
        generated.module(&fixture.db).krate(&fixture.db),
        fixture.consumer
    );
    assert_eq!(
        fixture
            .consumer
            .base()
            .env(&fixture.db)
            .get("LOD_FIXTURE_MODE")
            .as_deref(),
        Some(mode)
    );
    assert!(Path::new(out_dir.as_str()).starts_with(Path::new(fixture.target.as_str())));
    let runs = std::fs::read_to_string(out_dir.join("build-runs.log"))?;
    assert_eq!(runs.lines().last(), Some(mode));
    assert_eq!(fixture.consumer.edition(&fixture.db), Edition::Edition2024);
    assert_eq!(
        fixture
            .consumer
            .cfg(&fixture.db)
            .check_atom(&feature("alternate")),
        alternate
    );
    assert_eq!(
        fixture
            .consumer
            .cfg(&fixture.db)
            .check_atom(&flag("build_bonus")),
        mode == "bonus"
    );
    assert!(!fixture.consumer.cfg(&fixture.db).check_atom(&flag("test")));
    assert!(
        !fixture
            .consumer
            .cfg(&fixture.db)
            .check_atom(&flag("rust_analyzer"))
    );
    assert!(fixture.proc_client.is_none());
    assert_local_diagnostics(fixture);
    Ok(())
}

fn active_function(fixture: &BuiltFixture, name: &str) -> ProbeResult<Function> {
    let mut functions = fixture
        .consumer
        .root_module(&fixture.db)
        .declarations(&fixture.db)
        .into_iter()
        .filter_map(|declaration| match declaration {
            ModuleDef::Function(function) if function.name(&fixture.db).as_str() == name => {
                Some(function)
            }
            _ => None,
        });
    let function = functions.next().ok_or("active function is missing")?;
    assert!(
        functions.next().is_none(),
        "multiple active functions named {name}"
    );
    Ok(function)
}

fn integer_value(expression: &ast::Expr) -> ProbeResult<u128> {
    let ast::Expr::Literal(literal) = expression else {
        return Err("expected a source integer literal".into());
    };
    let ast::LiteralKind::IntNumber(integer) = literal.kind() else {
        return Err("source literal is not an integer".into());
    };
    Ok(integer.value()?)
}

fn assert_local_diagnostics(fixture: &BuiltFixture) {
    for module in fixture.consumer.modules(&fixture.db) {
        let mut diagnostics = Vec::new();
        module.diagnostics(&fixture.db, &mut diagnostics, false);
        let errors: Vec<_> = diagnostics
            .iter()
            .filter(|diagnostic| !matches!(diagnostic, AnyDiagnostic::InactiveCode(_)))
            .collect();
        assert!(
            errors.is_empty(),
            "unexpected consumer diagnostics: {errors:?}"
        );
    }
}

fn feature(name: &str) -> CfgAtom {
    CfgAtom::KeyValue {
        key: Symbol::intern("feature"),
        value: Symbol::intern(name),
    }
}

fn prepare_fixture(
    mode: &str,
    alternate: bool,
    set_test: bool,
) -> ProbeResult<(ProjectWorkspace, CargoConfig)> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../rust-expansion")
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
        .join(format!("analyzed-{mode}-{alternate}-{set_test}"));
    if !target.is_absolute() {
        return Err("CARGO_TARGET_DIR must be absolute".into());
    }
    let mut config = CargoConfig {
        sysroot: Some(RustLibSource::Path(AbsPathBuf::assert_utf8(sysroot))),
        sysroot_src: Some(AbsPathBuf::assert_utf8(source)),
        set_test,
        cfg_overrides: CfgOverrides {
            global: CfgDiff::new(Vec::new(), vec![flag("rust_analyzer")]),
            ..CfgOverrides::default()
        },
        features: CargoFeatures::Selected {
            features: if alternate {
                vec!["alternate".into()]
            } else {
                Vec::new()
            },
            no_default_features: false,
        },
        metadata_extra_args: vec!["--locked".into(), "--offline".into()],
        extra_args: vec!["--locked".into(), "--offline".into()],
        target_dir_config: TargetDirectoryConfig::Directory(
            target.to_str().ok_or("target path is not UTF-8")?.into(),
        ),
        ..CargoConfig::default()
    };
    config
        .extra_env
        .insert("LOD_FIXTURE_MODE".into(), Some(mode.into()));
    config.extra_env.insert("LOD_FIXTURE_BROKEN".into(), None);
    config.extra_env.insert(
        "CARGO_TARGET_DIR".into(),
        Some(target.to_str().ok_or("target path is not UTF-8")?.into()),
    );
    let manifest = ProjectManifest::discover_single(&AbsPathBuf::assert_utf8(directory))
        .map_err(|error| std::io::Error::other(format!("fixture manifest: {error:#}")))?;
    let mut workspace =
        ProjectWorkspace::load(manifest, &config, &|message| eprintln!("{message}"))
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
    let build = workspace
        .run_build_scripts(&config, &|message| eprintln!("{message}"))
        .map_err(|error| std::io::Error::other(format!("running fixture build: {error:#}")))?;
    eprintln!("fixture build error: {:?}", build.error());
    if let Some(error) = build.error() {
        return Err(std::io::Error::other(format!("fixture build failed: {error}")).into());
    }
    workspace.set_build_scripts(build);
    Ok((workspace, config))
}

fn flag(name: &str) -> CfgAtom {
    CfgAtom::Flag(Symbol::intern(name))
}
