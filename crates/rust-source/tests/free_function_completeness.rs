use std::path::{Path, PathBuf};
use std::process::Command;

use law_of_demeter_core::application::{
    AnalysisIssue, AnalysisIssueKind, CheckProject, ReadError, Selection, Target, TypeSource,
};
use law_of_demeter_rust_source::{LoadOptions, RustProject};

#[test]
fn a_generated_free_function_without_a_physical_range_keeps_a_global_unsupported_issue() {
    let project = load(&[]);
    let wallet = wallet_name(&project);

    let report = CheckProject::new(&project, &project).execute();
    let selected = CheckProject::new(&project, &project)
        .with_selection(Selection::of(vec![Target::method(&wallet, "debit")]))
        .execute();

    let expected = [AnalysisIssue::new(
        AnalysisIssueKind::Unsupported,
        ReadError::new(
            &fixture().join("lib.rs"),
            "free function `pay` has no physical source location and is unsupported",
        ),
    )];
    assert_eq!(report.issues(), expected, "{report:?}");
    assert_eq!(selected.issues(), expected, "{selected:?}");
    assert_eq!(report.counts().methods, 1);
    assert_eq!(selected.counts().methods, 1);
    assert!(report.findings().is_empty());
    assert!(report.source_errors().is_empty());
    assert!(report.type_errors().is_empty());
    assert!(selected.unmatched_targets().is_empty());
    assert_eq!(project.project_types(), [wallet]);
}

#[test]
fn the_handwritten_counterpart_keeps_its_existing_global_unsupported_location() {
    let project = load(&["plain"]);

    let report = CheckProject::new(&project, &project).execute();

    assert_eq!(
        report.issues(),
        [AnalysisIssue::new(
            AnalysisIssueKind::Unsupported,
            ReadError::at_line(
                &fixture().join("lib.rs"),
                19,
                "free functions are outside the first source slice",
            ),
        )],
        "{report:?}"
    );
    assert_eq!(report.counts().methods, 1);
    assert!(report.findings().is_empty());
    assert!(report.source_errors().is_empty());
    assert!(report.type_errors().is_empty());
}

#[test]
fn cfg_excluded_generated_and_handwritten_functions_do_not_create_executable_issues() {
    let project = load(&["disabled"]);

    let report = CheckProject::new(&project, &project).execute();

    assert!(report.issues().is_empty(), "{report:?}");
    assert_eq!(report.counts().methods, 1);
    assert!(report.findings().is_empty());
    assert!(report.source_errors().is_empty());
    assert!(report.type_errors().is_empty());
}

fn load(features: &[&str]) -> RustProject {
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
        .expect("Cargo checks the free function fixture");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    RustProject::load(
        &fixture().join("Cargo.toml"),
        LoadOptions::new("free-function-coverage", &output().join(features.join("-")))
            .with_features(
                features
                    .iter()
                    .map(|feature| (*feature).to_owned())
                    .collect(),
                false,
            ),
    )
    .expect("compiler-valid free function source loads")
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
        .join("tests/fixtures/free-function-coverage")
        .canonicalize()
        .expect("free function fixture exists")
}

fn output() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/rust-friends-journey/analyzed/free-function-coverage")
}
