#![cfg(unix)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SOURCE: &str = "pub struct PaymentService;
pub struct Wallet;
pub struct Order { wallet: Wallet }
impl Wallet { pub fn debit(&self, amount: i32) {} }
impl Order { pub fn wallet(&self) -> &Wallet { &self.wallet } }
impl PaymentService {
    pub fn on_parameter(&self, wallet: &Wallet) { wallet.debit(7); }
    pub fn on_returned_chain(&self, order: &Order) { order.wallet().debit(7); }
}
";
const CLEAN: &str = "pub struct Wallet;\nimpl Wallet { pub fn debit(&self) {} }\n";

#[test]
fn raw_identifier_type_selections_resolve_readable_and_suffix_names() {
    let project = RustFixture::new(include_str!("fixtures/rust-raw-identifiers.rs"));
    project.compile();

    let output = project.analyze(&[
        "--only=shop/shop::r#type",
        "--only=shop::r#type",
        "--only=r#type",
        "--show-allowed",
    ]);

    assert_code(&output, 0);
    let report = text(&output.stdout);
    assert!(report.contains("Methods: 2\n  Calls: 1"), "{report}");
    assert!(
        report.contains(
            "lib.rs:9  shop/shop::r#type#match  touch()  Allowed: other is an argument of f"
        ),
        "{report}"
    );
}

#[test]
fn raw_identifier_method_spelling_selects_the_declared_method() {
    let project = RustFixture::new(include_str!("fixtures/rust-raw-identifiers.rs"));
    project.compile();

    let output = project.analyze(&["--only=shop/shop::r#type#r#match", "--show-allowed"]);

    assert_code(&output, 0);
    let report = text(&output.stdout);
    assert!(report.contains("Methods: 1\n  Calls: 1"), "{report}");
    assert!(
        report.contains(
            "lib.rs:9  shop/shop::r#type#match  touch()  Allowed: other is an argument of f"
        ),
        "{report}"
    );
}

#[test]
fn displayed_method_spelling_selects_raw_identifier_declarations() {
    let project = RustFixture::new(include_str!("fixtures/rust-raw-identifiers.rs"));
    project.compile();

    let output = project.analyze(&["--only=shop/shop::r#type#match", "--show-allowed"]);

    assert_code(&output, 0);
    let report = text(&output.stdout);
    assert!(report.contains("Methods: 1\n  Calls: 1"), "{report}");
}

#[test]
fn a_trait_dispatch_is_incomplete_instead_of_using_a_same_named_inherent_signature() {
    let project = RustFixture::new(include_str!("fixtures/rust-trait-dispatch.rs"));
    project.compile();

    let output = project.analyze(&["--only=Processor#through_trait"]);

    assert_code(&output, 3);
    let report = text(&output.stdout);
    assert!(report.contains("lib.rs:29  shop/shop::Processor#through_trait  Unsupported: trait method targets are outside the first source slice"), "{report}");
    assert!(report.contains("Methods: 0\n  Calls: 0"), "{report}");
    assert!(!report.contains("No violations found."), "{report}");
    assert!(!text(&output.stderr).contains("nothing matches --only"));
}

#[test]
fn a_deref_dispatch_is_incomplete_instead_of_using_the_wrapper_as_receiver() {
    let project = RustFixture::new(include_str!("fixtures/rust-deref-dispatch.rs"));
    project.compile();

    let output = project.analyze(&["--only=Processor#through_deref"]);

    assert_code(&output, 3);
    let report = text(&output.stdout);
    assert!(report.contains("lib.rs:23  shop/shop::Processor#through_deref  Unsupported: receiver projection is outside the first source slice"), "{report}");
    assert!(report.contains("Methods: 0\n  Calls: 0"), "{report}");
    assert!(!report.contains("No violations found."), "{report}");
    assert!(!text(&output.stderr).contains("nothing matches --only"));
}

#[test]
fn a_valid_empty_library_is_complete_with_zero_methods_and_calls() {
    let project = RustFixture::new("");

    let output = project.analyze(&[]);

    assert_code(&output, 0);
    let report = text(&output.stdout);
    assert!(
        report.contains("Methods: 0\n  Calls: 0 (allowed 0, violations 0, unresolved 0)"),
        "{report}"
    );
    assert!(report.contains("Analysis issues: 0"), "{report}");
}

#[test]
fn the_default_analysis_output_belongs_to_the_cargo_workspace() {
    let project = RustFixture::new(CLEAN);

    let output = Command::new(env!("CARGO_BIN_EXE_law-of-demeter"))
        .current_dir(&project.path)
        .args(["--rust=Cargo.toml", "--package=shop"])
        .output()
        .expect("CLI process");

    assert_code(&output, 0);
    let expected = project
        .path
        .canonicalize()
        .expect("workspace path")
        .join("target/law-of-demeter");
    assert!(expected.is_dir());
    assert!(text(&output.stderr).contains(&format!("Analysis output: {}", expected.display())));
}

#[test]
fn a_nonexistent_selection_in_a_complete_project_returns_two() {
    let project = RustFixture::new(CLEAN);

    let output = project.analyze(&["--only=Wallet#absent"]);

    assert_code(&output, 2);
    assert!(text(&output.stderr).contains("nothing matches --only: Wallet#absent"));
}

#[test]
fn only_unmatched_selections_are_named_in_the_no_match_error() {
    let project = RustFixture::new(CLEAN);

    let output = project.analyze(&["--only=Wallet#debit", "--only=Missing"]);

    assert_code(&output, 2);
    let context = text(&output.stderr);
    assert!(
        context.contains("Error: nothing matches --only: Missing\n"),
        "{context}"
    );
    assert!(
        !context.contains("nothing matches --only: Wallet#debit"),
        "{context}"
    );
}

#[test]
fn an_unsupported_selected_body_is_incomplete_even_when_it_has_no_method_report() {
    let project = RustFixture::new(SOURCE);

    let output = project.analyze(&["--only=Order#wallet"]);

    assert_code(&output, 3);
    let report = text(&output.stdout);
    assert!(
        report.contains("shop/shop::Order#wallet  Unsupported:"),
        "{report}"
    );
    assert!(report.contains("Methods: 0"), "{report}");
    assert!(!report.contains("No violations found."), "{report}");
}

#[test]
fn a_missing_method_target_is_incomplete_with_its_owner_and_location() {
    let project = RustFixture::new(
        "pub struct Wallet;\nimpl Wallet { pub fn pay(&self) { self.absent(); } }\n",
    );

    let output = project.analyze(&["--only=Wallet#pay"]);

    assert_code(&output, 3);
    assert!(
        text(&output.stdout).contains("lib.rs:2  shop/shop::Wallet#pay  TargetResolution:"),
        "{}",
        text(&output.stdout)
    );
}

#[test]
fn an_unreadable_module_prevents_a_false_no_match_verdict() {
    let project = RustFixture::new("mod missing;\n");

    let output = project.analyze(&["--only=Missing#pay"]);

    assert_code(&output, 3);
    let report = text(&output.stdout);
    assert!(
        report.contains("Read: module source cannot be read"),
        "{report}"
    );
    assert!(!text(&output.stderr).contains("nothing matches --only"));
}

#[test]
fn uncertain_selection_keeps_only_selected_findings_and_counts() {
    let project = RustFixture::new(SOURCE);

    let output = project.analyze(&[
        "--only=PaymentService#on_parameter",
        "--only=Missing#pay",
        "--show-allowed",
    ]);

    assert_code(&output, 3);
    let report = text(&output.stdout);
    assert!(
        report.contains("Methods: 1\n  Calls: 1 (allowed 1, violations 0, unresolved 0)"),
        "{report}"
    );
    assert!(report.contains("Unverified selection issues"), "{report}");
    assert!(
        !report.contains("No violations found.") && !report.contains("debit()  Violation:"),
        "{report}"
    );
    let context = text(&output.stderr);
    assert!(
        context.contains(
            "Selection could not be verified because project analysis is incomplete: Missing#pay\n"
        ),
        "{context}"
    );
}

#[test]
fn invalid_syntax_reports_parse_issues_and_returns_three() {
    let project = RustFixture::new("mod broken;\n");
    fs::write(project.path.join("broken.rs"), "pub struct Wallet {\n").expect("broken source");

    let output = project.analyze(&[]);

    assert_code(&output, 3);
    assert!(
        text(&output.stdout).contains("Parse:"),
        "{}",
        text(&output.stdout)
    );
}

#[test]
fn unresolved_declared_types_return_three_outside_the_selection() {
    let project = RustFixture::new(
        "pub struct Wallet { missing: Absent }\npub struct Service;\nimpl Service { pub fn pay(&self) {} }\n",
    );

    let output = project.analyze(&["--only=Service#pay"]);

    assert_code(&output, 3);
    assert!(
        text(&output.stdout).contains("Type reading errors (1)"),
        "{}",
        text(&output.stdout)
    );
}

#[test]
fn a_known_violation_is_preserved_when_an_active_free_function_is_unsupported() {
    let project = RustFixture::new(&format!("{SOURCE}pub fn unsupported() {{}}\n"));

    let output = project.analyze(&["--only=PaymentService#on_returned_chain"]);

    assert_code(&output, 3);
    let report = text(&output.stdout);
    assert!(report.contains("Violations (1):"), "{report}");
    assert!(report.contains("Unsupported:"), "{report}");
}

#[test]
fn inactive_cfg_errors_are_excluded_from_a_complete_library() {
    let project = RustFixture::new(&format!(
        "{CLEAN}#[cfg(any())]\nmod missing;\n#[cfg(any())]\npub fn invalid() {{ absent(); }}\n"
    ));

    let output = project.analyze(&[]);

    assert_code(&output, 0);
    assert!(text(&output.stdout).contains("Analysis issues: 0"));
}

#[test]
fn cfg_macro_does_not_remove_the_inactive_runtime_branch_from_analysis() {
    let project = RustFixture::new(
        "pub struct Wallet;\nimpl Wallet { pub fn pay(&self) { if cfg!(any()) { self.absent(); } } }\n",
    );

    let output = project.analyze(&["--only=Wallet#pay"]);

    assert_code(&output, 3);
    assert!(
        text(&output.stdout).contains("TargetResolution:"),
        "{}",
        text(&output.stdout)
    );
}

#[test]
fn cargo_features_and_no_default_features_reach_the_active_source_configuration() {
    let project = RustFixture::new(&format!(
        "{CLEAN}#[cfg(feature = \"unsupported\")]\npub fn unsupported() {{}}\n#[cfg(feature = \"payment\")]\nimpl Wallet {{ pub fn pay(&self) {{ self.debit(); }} }}\n"
    ));
    project.extend_manifest(
        "[features]\ndefault = [\"unsupported\"]\nunsupported = []\npayment = []\n",
    );

    let output = project.analyze(&[
        "--features=payment",
        "--no-default-features",
        "--only=Wallet#pay",
        "--show-allowed",
    ]);

    assert_code(&output, 0);
    assert!(
        text(&output.stdout).contains("debit()  Allowed: f's own object"),
        "{}",
        text(&output.stdout)
    );
}

#[test]
fn inline_rust_paths_keep_features_and_selection_active() {
    let project = RustFixture::new(&format!(
        "{CLEAN}#[cfg(feature = \"payment\")]\nimpl Wallet {{ pub fn pay(&self) {{ self.debit(); }} }}\n"
    ));
    project.extend_manifest("[features]\npayment = []\n");
    let library = project.path.join("-library");
    fs::create_dir(&library).expect("nested library");
    for file in ["Cargo.toml", "Cargo.lock", "lib.rs"] {
        fs::rename(project.path.join(file), library.join(file)).expect("move library source");
    }

    let output = project.analyze(&[
        "--rust=-library/Cargo.toml",
        "--target-dir=-analysis",
        "--features=payment",
        "--only=Wallet#pay",
        "--show-allowed",
    ]);

    assert_code(&output, 0);
    assert!(text(&output.stdout).contains("debit()  Allowed: f's own object"));
    let context = text(&output.stderr);
    assert!(
        context.contains(&format!(
            "Analysis output: {}",
            project
                .path
                .canonicalize()
                .expect("project path")
                .join("-analysis")
                .display()
        )),
        "{context}"
    );
}

#[test]
fn an_ambiguous_type_suffix_requires_an_explicit_catalog_choice() {
    let project = RustFixture::new(
        "pub mod first { pub struct Wallet; }\npub mod second { pub struct Wallet; }\n",
    );

    let output = project.analyze(&["--only=Wallet"]);

    assert_code(&output, 2);
    let error = text(&output.stderr);
    assert!(error.contains("ambiguous --only: Wallet"), "{error}");
    assert!(
        error.contains("shop/shop::first::Wallet") && error.contains("shop/shop::second::Wallet"),
        "{error}"
    );
}

#[test]
fn a_full_cargo_identity_containing_a_hash_preserves_the_last_method_separator() {
    let project = RustFixture::new(CLEAN);
    let metadata = Command::new("cargo")
        .current_dir(&project.path)
        .args([
            "metadata",
            "--no-deps",
            "--format-version=1",
            "--offline",
            "--locked",
        ])
        .output()
        .expect("metadata");
    assert!(metadata.status.success());
    let metadata: serde_json::Value =
        serde_json::from_slice(&metadata.stdout).expect("metadata JSON");
    let id = metadata["packages"][0]["id"].as_str().expect("package ID");
    let target = format!("--only={id}::shop::Wallet#debit");

    let output = project.analyze(&[&target]);

    assert!(id.contains('#'), "{id}");
    assert_code(&output, 0);
    assert!(text(&output.stdout).contains("Methods: 1"));
}

#[test]
fn an_invalid_target_platform_fails_before_analysis_with_the_cargo_reason() {
    let project = RustFixture::new(CLEAN);

    let output = project.analyze(&["--target=missing-platform"]);

    assert_code(&output, 2);
    let error = text(&output.stderr);
    assert!(error.contains("missing-platform"), "{error}");
    assert!(!error.contains("JDK"), "{error}");
}

#[test]
fn a_build_script_failure_explains_why_analysis_could_not_start() {
    let project = RustFixture::new(CLEAN);
    fs::write(
        project.path.join("build.rs"),
        "fn main() { panic!(\"build preparation failed\"); }\n",
    )
    .expect("build source");

    let output = project.analyze(&[]);

    assert_code(&output, 2);
    let error = text(&output.stderr);
    assert!(
        error.contains("Cargo build failed") && error.contains("build preparation failed"),
        "{error}"
    );
}

#[test]
fn an_absent_manifest_explains_the_startup_failure() {
    let project = RustFixture::new(CLEAN);
    fs::remove_file(project.path.join("Cargo.toml")).expect("remove fixture manifest");

    let output = project.analyze(&[]);

    assert_code(&output, 2);
    assert!(text(&output.stderr).contains("Cargo metadata for"));
}

#[test]
fn package_selection_keeps_other_workspace_types_in_the_catalog() {
    let project = RustFixture::new(CLEAN);
    fs::write(
        project.path.join("Cargo.toml"),
        "[workspace]\nresolver = \"3\"\nmembers = [\"orders\", \"other\"]\n",
    )
    .expect("workspace manifest");
    for package in ["orders", "other"] {
        let directory = project.path.join(package);
        fs::create_dir(&directory).expect("member directory");
        fs::write(directory.join("Cargo.toml"), format!("[package]\nname = \"{package}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n[lib]\npath = \"lib.rs\"\n")).expect("member manifest");
        fs::write(directory.join("lib.rs"), CLEAN).expect("member source");
    }
    fs::write(project.path.join("Cargo.lock"), "version = 4\n[[package]]\nname = \"orders\"\nversion = \"0.1.0\"\n[[package]]\nname = \"other\"\nversion = \"0.1.0\"\n").expect("workspace lock");

    let output = project.analyze(&["--package=orders", "--only=Wallet"]);

    assert_code(&output, 2);
    let error = text(&output.stderr);
    assert!(
        error.contains("orders/orders::Wallet") && error.contains("other/other::Wallet"),
        "{error}"
    );
}

#[test]
fn a_rust_parameter_call_succeeds_without_a_jdk_or_javac() {
    let project = RustFixture::new(SOURCE);

    let output = project.analyze(&[
        "--only=shop/shop::PaymentService#on_parameter",
        "--show-allowed",
    ]);

    assert_code(&output, 0);
    let report = text(&output.stdout);
    assert!(report.contains("lib.rs:7  shop/shop::PaymentService#on_parameter  debit()  Allowed: wallet is an argument of f"), "{report}");
    assert!(
        report.contains("Methods: 1\n  Calls: 1 (allowed 1, violations 0, unresolved 0)"),
        "{report}"
    );
    assert!(!text(&output.stderr).contains("JDK:"));
}

#[test]
fn a_returned_wallet_reports_the_exact_violation_and_returns_one() {
    let project = RustFixture::new(SOURCE);

    let output = project.analyze(&[
        "--only",
        "PaymentService#on_returned_chain",
        "--show-allowed",
    ]);

    assert_code(&output, 1);
    let report = text(&output.stdout);
    assert!(report.contains("Violations (1):"), "{report}");
    assert!(report.contains("lib.rs:8  shop/shop::PaymentService#on_returned_chain  debit()  Violation: the object returned by wallet() is not a friend of f"), "{report}");
    assert!(
        report.contains("Calls: 2 (allowed 1, violations 1, unresolved 0)"),
        "{report}"
    );
}

#[test]
#[ignore = "requires Docker engine and a built image; run with --ignored"]
fn rust_container_keeps_a_parameter_call_complete_on_a_read_only_source_mount() {
    let project = RustFixture::new(SOURCE);

    let output = project.in_container(&["--only=PaymentService#on_parameter", "--show-allowed"]);

    assert_code(&output, 0);
    assert!(text(&output.stdout).contains("debit()  Allowed:"));
    assert!(!project.path.join("target").exists());
}

#[test]
#[ignore = "requires Docker engine and a built image; run with --ignored"]
fn rust_container_reports_a_returned_wallet_with_exit_one() {
    let project = RustFixture::new(SOURCE);

    let output = project.in_container(&["--only=PaymentService#on_returned_chain"]);

    assert_code(&output, 1);
    assert!(
        text(&output.stdout)
            .contains("debit()  Violation: the object returned by wallet() is not a friend of f")
    );
}

#[test]
#[ignore = "requires Docker engine and a built image; run with --ignored"]
fn rust_container_keeps_an_unsupported_body_incomplete() {
    let project = RustFixture::new(SOURCE);

    let output = project.in_container(&["--only=Order#wallet"]);

    assert_code(&output, 3);
    assert!(text(&output.stdout).contains("Unsupported:"));
}

#[test]
#[ignore = "requires Docker engine and a built image; run with --ignored"]
fn rust_container_keeps_a_trait_dispatch_incomplete() {
    let project = RustFixture::new(include_str!("fixtures/rust-trait-dispatch.rs"));
    project.compile_in_container();

    let output = project.in_container(&["--only=Processor#through_trait"]);

    assert_code(&output, 3);
    let report = text(&output.stdout);
    assert!(report.contains("lib.rs:29  shop/shop::Processor#through_trait  Unsupported: trait method targets are outside the first source slice"), "{report}");
    assert!(report.contains("Methods: 0\n  Calls: 0"), "{report}");
}

#[test]
#[ignore = "requires Docker engine and a built image; run with --ignored"]
fn rust_container_keeps_a_deref_dispatch_incomplete() {
    let project = RustFixture::new(include_str!("fixtures/rust-deref-dispatch.rs"));
    project.compile_in_container();

    let output = project.in_container(&["--only=Processor#through_deref"]);

    assert_code(&output, 3);
    let report = text(&output.stdout);
    assert!(report.contains("lib.rs:23  shop/shop::Processor#through_deref  Unsupported: receiver projection is outside the first source slice"), "{report}");
    assert!(report.contains("Methods: 0\n  Calls: 0"), "{report}");
}

#[test]
#[ignore = "requires Docker engine and a built image; run with --ignored"]
fn rust_container_selects_raw_identifier_types_and_methods() {
    let project = RustFixture::new(include_str!("fixtures/rust-raw-identifiers.rs"));
    project.compile_in_container();

    let type_only = project.in_container(&["--only=shop/shop::r#type", "--show-allowed"]);
    let raw_method = project.in_container(&["--only=shop/shop::r#type#r#match", "--show-allowed"]);
    let displayed_method =
        project.in_container(&["--only=shop/shop::r#type#match", "--show-allowed"]);

    assert_eq!(
        (
            type_only.status.code(),
            raw_method.status.code(),
            displayed_method.status.code()
        ),
        (Some(0), Some(0), Some(0)),
        "{}\n{}\n{}",
        text(&type_only.stderr),
        text(&raw_method.stderr),
        text(&displayed_method.stderr)
    );
    assert!(text(&type_only.stdout).contains("Methods: 2\n  Calls: 1"));
    for output in [raw_method, displayed_method] {
        let report = text(&output.stdout);
        assert!(report.contains("Methods: 1\n  Calls: 1"), "{report}");
        assert!(
            report.contains(
                "lib.rs:9  shop/shop::r#type#match  touch()  Allowed: other is an argument of f"
            ),
            "{report}"
        );
    }
}

struct RustFixture {
    _temporary: tempfile::TempDir,
    path: PathBuf,
    output: PathBuf,
}

impl RustFixture {
    fn compile(&self) {
        let output = Command::new("cargo")
            .current_dir(&self.path)
            .args(["check", "--offline", "--locked", "--lib"])
            .env("CARGO_TARGET_DIR", self.output.join("compiler"))
            .output()
            .expect("Cargo checks the dispatch fixture");

        assert!(output.status.success(), "{}", text(&output.stderr));
    }

    fn compile_in_container(&self) {
        let output = self
            .container_command()
            .args([
                "--env",
                "CARGO_TARGET_DIR=/analysis/compiler",
                "--entrypoint",
                "cargo",
            ])
            .arg(container_image())
            .args(["check", "--offline", "--locked", "--lib"])
            .output()
            .expect("container Cargo checks the dispatch fixture");

        assert!(output.status.success(), "{}", text(&output.stderr));
    }

    fn in_container(&self, arguments: &[&str]) -> Output {
        self.container_command()
            .arg(container_image())
            .args([
                "--rust=Cargo.toml",
                "--package=shop",
                "--target-dir=/analysis",
            ])
            .args(arguments)
            .output()
            .expect("Docker engine and built image")
    }

    fn container_command(&self) -> Command {
        let cache = self._temporary.path().join("cargo cache");
        fs::create_dir_all(&cache).expect("container cache");
        fs::create_dir_all(&self.output).expect("container build output");
        let host_id = |flag| {
            let result = Command::new("id").arg(flag).output().expect("host ID");
            assert!(result.status.success());
            text(&result.stdout).trim().to_owned()
        };
        let mut command = Command::new("docker");
        command
            .args(["run", "--rm", "--pull=never", "--user"])
            .arg(format!("{}:{}", host_id("-u"), host_id("-g")))
            .arg("--mount")
            .arg(format!(
                "type=bind,src={},dst=/workspace,readonly",
                self.path.canonicalize().expect("source path").display()
            ))
            .arg("--mount")
            .arg(format!(
                "type=bind,src={},dst=/analysis",
                self.output.canonicalize().expect("output path").display()
            ))
            .arg("--mount")
            .arg(format!(
                "type=bind,src={},dst=/cargo-cache",
                cache.canonicalize().expect("cache path").display()
            ))
            .args([
                "--env",
                "CARGO_HOME=/cargo-cache",
                "--env",
                "JAVA_HOME=/missing-jdk",
            ]);
        command
    }
    fn extend_manifest(&self, extra: &str) {
        let manifest = self.path.join("Cargo.toml");
        let original = fs::read_to_string(&manifest).expect("manifest");
        fs::write(manifest, format!("{original}{extra}")).expect("manifest update");
    }
    fn new(source: &str) -> Self {
        let root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/rust-cli-journey/fixtures");
        fs::create_dir_all(&root).expect("fixture root is writable");
        let temporary = tempfile::tempdir_in(root).expect("fixture directory");
        let path = temporary.path().join("project with spaces");
        fs::create_dir_all(&path).expect("source directory");
        fs::write(path.join("Cargo.toml"), "[package]\nname = \"shop\"\nversion = \"0.1.0\"\nedition = \"2024\"\n[lib]\npath = \"lib.rs\"\n[workspace]\n").expect("manifest");
        fs::write(
            path.join("Cargo.lock"),
            "version = 4\n[[package]]\nname = \"shop\"\nversion = \"0.1.0\"\n",
        )
        .expect("lock");
        fs::write(path.join("lib.rs"), source).expect("source");
        let output = temporary.path().join("analysis with spaces");
        Self {
            path,
            output,
            _temporary: temporary,
        }
    }

    fn analyze(&self, arguments: &[&str]) -> Output {
        let tools = self._temporary.path().join("tools without java");
        fs::create_dir_all(&tools).expect("tools directory");
        for tool in [
            "cargo", "rustc", "rustup", "cc", "clang", "ld", "ar", "xcrun",
        ] {
            let found = Command::new("which")
                .arg(tool)
                .output()
                .expect("tool lookup");
            if found.status.success() {
                let source = PathBuf::from(text(&found.stdout).trim());
                let target = tools.join(tool);
                if !target.exists() {
                    #[cfg(unix)]
                    std::os::unix::fs::symlink(source, target).expect("tool link");
                }
            }
        }
        Command::new(env!("CARGO_BIN_EXE_law-of-demeter"))
            .current_dir(&self.path)
            .env("JAVA_HOME", "/missing-jdk")
            .env("PATH", &tools)
            .arg("--rust=Cargo.toml")
            .arg("--package=shop")
            .arg("--target-dir")
            .arg(&self.output)
            .args(arguments)
            .output()
            .expect("CLI process runs")
    }
}

fn container_image() -> std::ffi::OsString {
    std::env::var_os("CONTAINER_IMAGE").unwrap_or_else(|| "law-of-demeter:local".into())
}

fn assert_code(output: &Output, expected: i32) {
    assert_eq!(
        output.status.code(),
        Some(expected),
        "{}\n{}",
        text(&output.stdout),
        text(&output.stderr)
    );
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
