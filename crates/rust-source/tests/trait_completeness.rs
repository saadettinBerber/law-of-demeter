use std::path::{Path, PathBuf};
use std::process::Command;

use law_of_demeter_core::application::{
    AnalysisIssue, AnalysisIssueKind, CheckProject, ReadError, Selection, Target, TypeSource,
};
use law_of_demeter_rust_source::{LoadOptions, RustProject};

#[test]
fn an_active_trait_default_body_is_incomplete_without_fabricating_a_method_or_call() {
    let project = load(&[]);
    let wallet = wallet_name(&project);
    let owner = wallet
        .strip_suffix("::Wallet")
        .expect("Wallet has a named container")
        .to_owned()
        + "::Checkout";

    let report = CheckProject::new(&project, &project).execute();

    assert_eq!(
        report.issues(),
        [AnalysisIssue::new(
            AnalysisIssueKind::Unsupported,
            ReadError::at_line(
                &fixture().join("lib.rs"),
                11,
                "trait default method bodies are outside the supported scope"
            ),
        )
        .in_body(Target::method(&owner, "pay"))],
        "{report:?}"
    );
    assert_eq!(report.counts().methods, 1, "{report:?}");
    assert!(report.findings().is_empty());
    assert!(report.source_errors().is_empty());
    assert!(report.type_errors().is_empty());
    assert_eq!(project.project_types(), [wallet]);
}

#[test]
fn required_signatures_and_cfg_excluded_default_bodies_are_not_executable() {
    let project = load(&["signatures-only"]);

    let report = CheckProject::new(&project, &project).execute();

    assert!(report.issues().is_empty(), "{report:?}");
    assert_eq!(report.counts().methods, 1);
    assert!(report.findings().is_empty());
    assert!(report.source_errors().is_empty());
    assert!(report.type_errors().is_empty());
}

#[test]
fn selecting_an_unrelated_method_excludes_the_trait_default_body_issue() {
    let project = load(&[]);
    let selected = Target::method(&wallet_name(&project), "debit");

    let report = CheckProject::new(&project, &project)
        .with_selection(Selection::of(vec![selected]))
        .execute();

    assert!(report.issues().is_empty(), "{report:?}");
    assert!(report.unmatched_targets().is_empty());
    assert_eq!(report.counts().methods, 1);
    assert!(report.findings().is_empty());
}

#[test]
fn a_macro_generated_trait_default_body_keeps_an_explicit_issue() {
    let project = load(&["signatures-only", "generated-default"]);
    let wallet = wallet_name(&project);
    let owner = wallet
        .strip_suffix("::Wallet")
        .expect("Wallet has a named container")
        .to_owned()
        + "::GeneratedCheckout";

    let report = CheckProject::new(&project, &project).execute();

    assert_eq!(report.issues().len(), 1, "{report:?}");
    assert_eq!(report.issues()[0].kind(), AnalysisIssueKind::Unsupported);
    assert_eq!(
        report.issues()[0].owner(),
        Some(&Target::method(&owner, "execute"))
    );
    assert_eq!(report.issues()[0].path(), fixture().join("lib.rs"));
    assert!(report.findings().is_empty());
    assert_eq!(report.counts().methods, 1);
    assert!(report.source_errors().is_empty());
    assert!(report.type_errors().is_empty());
}

fn load(features: &[&str]) -> RustProject {
    let options = features
        .iter()
        .map(|feature| (*feature).to_owned())
        .collect();
    let mut compiler = Command::new("cargo");
    compiler
        .args(["check", "--offline", "--locked", "--lib", "--manifest-path"])
        .arg(fixture().join("Cargo.toml"));
    if !features.is_empty() {
        compiler.arg("--features").arg(features.join(","));
    }
    let compiled = compiler
        .env("CARGO_TARGET_DIR", output().join("compiler"))
        .output()
        .expect("Cargo checks the trait fixture");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    RustProject::load(
        &fixture().join("Cargo.toml"),
        LoadOptions::new("trait-defaults", &output().join(features.join("-")))
            .with_features(options, false),
    )
    .expect("compiler-valid trait source loads")
}

fn wallet_name(project: &RustProject) -> String {
    project
        .project_types()
        .into_iter()
        .find(|name| name.ends_with("::Wallet"))
        .expect("Wallet is declared")
}

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/trait-defaults")
        .canonicalize()
        .expect("trait fixture exists")
}

fn output() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/rust-friends-journey/analyzed/trait-defaults")
}
