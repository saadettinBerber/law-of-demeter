use std::path::{Path, PathBuf};
use std::process::Command;

use law_of_demeter_core::application::{
    AnalysisIssue, AnalysisIssueKind, CheckProject, ReadError, Selection, Target, TypeSource,
};
use law_of_demeter_core::domain::method::{Reason, Verdict};
use law_of_demeter_rust_source::{LoadOptions, RustProject};

#[test]
fn a_selected_trait_default_is_reported_instead_of_using_an_inherent_fluent_signature() {
    let project = load_compiled_fixture();
    let owner = Target::method(&processor(&project), "through_trait");

    let report = CheckProject::new(&project, &project)
        .with_selection(Selection::of(vec![owner.clone()]))
        .execute();

    assert_eq!(
        report.issues(),
        [AnalysisIssue::new(
            AnalysisIssueKind::Unsupported,
            ReadError::at_line(
                &fixture().join("lib.rs"),
                41,
                "trait method targets are outside the first source slice",
            ),
        )
        .in_body(owner)],
        "{report:?}"
    );
    assert!(report.findings().is_empty(), "{:?}", report.findings());
    assert!(
        report.source_errors().is_empty(),
        "{:?}",
        report.source_errors()
    );
    assert!(
        report.type_errors().is_empty(),
        "{:?}",
        report.type_errors()
    );
}

#[test]
fn a_deref_projection_is_reported_instead_of_treating_the_wrapper_as_the_receiver() {
    let project = load_compiled_fixture();
    let owner = Target::method(&processor(&project), "through_deref");

    let report = CheckProject::new(&project, &project)
        .with_selection(Selection::of(vec![owner.clone()]))
        .execute();

    assert_eq!(
        report.issues(),
        [AnalysisIssue::new(
            AnalysisIssueKind::Unsupported,
            ReadError::at_line(
                &fixture().join("lib.rs"),
                45,
                "receiver projection is outside the first source slice",
            ),
        )
        .in_body(owner)],
        "{report:?}"
    );
    assert!(report.findings().is_empty(), "{:?}", report.findings());
    assert!(
        report.source_errors().is_empty(),
        "{:?}",
        report.source_errors()
    );
    assert!(
        report.type_errors().is_empty(),
        "{:?}",
        report.type_errors()
    );
}

#[test]
fn references_and_autoref_keep_the_original_object_for_an_inherent_target() {
    let project = load_compiled_fixture();
    let processor = processor(&project);
    let selection = Selection::of(vec![
        Target::method(&processor, "through_references"),
        Target::method(&processor, "through_autoref"),
        Target::method(&processor, "through_inherent"),
    ]);

    let report = CheckProject::new(&project, &project)
        .with_selection(selection)
        .execute();

    let findings: Vec<_> = report
        .findings()
        .iter()
        .map(|finding| (finding.method_name(), finding.verdict().clone()))
        .collect();
    assert_eq!(
        findings,
        [
            (
                "through_references",
                Verdict::allowed("apply", 49, Reason::parameter("account"))
            ),
            (
                "through_autoref",
                Verdict::allowed("apply", 53, Reason::parameter("account"))
            ),
            (
                "through_inherent",
                Verdict::allowed("account", 57, Reason::parameter("parcel"))
            ),
        ]
    );
    assert!(report.issues().is_empty(), "{:?}", report.issues());
    assert!(report.unmatched_targets().is_empty());
    assert!(report.source_errors().is_empty());
    assert!(report.type_errors().is_empty());
}

fn load_compiled_fixture() -> RustProject {
    let compiled = Command::new("cargo")
        .args(["check", "--offline", "--locked", "--manifest-path"])
        .arg(fixture().join("Cargo.toml"))
        .arg("--lib")
        .env("CARGO_TARGET_DIR", output().join("compiler"))
        .output()
        .expect("Cargo checks the dispatch fixture");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    RustProject::load(
        &fixture().join("Cargo.toml"),
        LoadOptions::new("dispatch-boundary", &output().join("analysis")),
    )
    .expect("compiler-valid Cargo source loads")
}

fn processor(project: &RustProject) -> String {
    project
        .project_types()
        .into_iter()
        .find(|name| name.ends_with("::Processor"))
        .expect("Processor is a project declaration")
}

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/dispatch")
        .canonicalize()
        .expect("dispatch fixture exists")
}

fn output() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/rust-source-journey/analyzed/dispatch")
}
