use std::path::{Path, PathBuf};

use law_of_demeter_core::application::{
    AnalysisIssueKind, CheckProject, Selection, Target, TypeSource,
};
use law_of_demeter_core::domain::method::{Reason, Verdict};
use law_of_demeter_rust_source::{LoadOptions, RustProject};

#[test]
fn a_returned_wallet_is_reported_while_a_parameter_wallet_is_allowed() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/rust-scenarios")
        .canonicalize()
        .expect("fixture is available");
    let project = RustProject::load(
        &directory.join("Cargo.toml"),
        LoadOptions::new("rust-scenario-core", &output("scenarios")),
    )
    .expect("Cargo source loads");
    let service = project_type(&project, "PaymentService");
    let selection = Selection::of(vec![
        Target::method(&service, "on_parameter"),
        Target::method(&service, "on_returned_chain"),
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
                "on_parameter",
                Verdict::allowed("debit", 13, Reason::parameter("wallet"))
            ),
            (
                "on_returned_chain",
                Verdict::allowed("wallet", 29, Reason::parameter("order"))
            ),
            (
                "on_returned_chain",
                Verdict::violation("debit", 29, Reason::return_of("wallet"))
            ),
        ]
    );
    assert!(
        report
            .findings()
            .iter()
            .all(|finding| finding.path() == directory.join("core.rs")
                && finding.class_name() == service)
    );
    assert_eq!(report.counts().methods, 2);
    assert_eq!(report.counts().unresolved, 0);
    assert!(report.issues().is_empty(), "{:?}", report.issues());
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
    assert!(report.unmatched_targets().is_empty());
}

#[test]
fn unrelated_names_and_modules_preserve_the_parameter_and_returned_object_distinction() {
    let project = control(&[]);
    let processor = project_type(&project, "Processor");
    let selection = Selection::of(vec![
        Target::method(&processor, "direct"),
        Target::method(&processor, "chained"),
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
                "direct",
                Verdict::allowed("apply", 9, Reason::parameter("account"))
            ),
            (
                "chained",
                Verdict::allowed("account", 13, Reason::parameter("parcel"))
            ),
            (
                "chained",
                Verdict::violation("apply", 13, Reason::return_of("account"))
            ),
        ]
    );
    assert!(report.issues().is_empty(), "{:?}", report.issues());
    assert!(report.source_errors().is_empty());
    assert!(report.type_errors().is_empty());
    assert!(report.unmatched_targets().is_empty());
}

#[test]
fn a_known_receiver_with_a_missing_method_produces_an_issue_and_no_allowed_call() {
    let project = control(&["missing"]);
    let processor = project_type(&project, "Processor");
    let selection = Selection::of(vec![Target::method(&processor, "missing")]);

    let report = CheckProject::new(&project, &project)
        .with_selection(selection)
        .execute();

    assert!(report.findings().is_empty(), "{:?}", report.findings());
    assert!(
        report
            .issues()
            .iter()
            .any(|issue| issue.kind() == AnalysisIssueKind::TargetResolution
                && issue.line() == Some(18)),
        "{:?}",
        report.issues()
    );
}

#[test]
fn an_invalid_argument_is_reported_as_incomplete_compilation() {
    let project = control(&["wrong-type"]);
    let processor = project_type(&project, "Processor");
    let selection = Selection::of(vec![Target::method(&processor, "bad_type")]);

    let report = CheckProject::new(&project, &project)
        .with_selection(selection)
        .execute();

    assert!(
        report
            .issues()
            .iter()
            .any(|issue| issue.kind() == AnalysisIssueKind::Compilation),
        "{:?}",
        report.issues()
    );
}

#[test]
fn a_project_type_with_an_unknown_field_stays_in_the_catalog_and_reports_a_type_error() {
    let project = control(&["unknown-type"]);
    let processor = project_type(&project, "Processor");
    let broken = project_type(&project, "Broken");

    let report = CheckProject::new(&project, &project)
        .with_selection(Selection::of(vec![Target::method(&processor, "direct")]))
        .execute();

    assert!(broken.ends_with("::Broken"));
    assert_eq!(report.type_errors().len(), 1, "{:?}", report.type_errors());
    assert_eq!(report.type_errors()[0].line(), Some(27));
    assert_eq!(
        report.type_errors()[0]
            .path()
            .file_name()
            .and_then(|name| name.to_str()),
        Some("lib.rs")
    );
}

#[test]
fn an_unreadable_module_reports_a_source_error_even_outside_the_selection() {
    let project = control(&["missing-file"]);
    let processor = project_type(&project, "Processor");

    let report = CheckProject::new(&project, &project)
        .with_selection(Selection::of(vec![Target::method(&processor, "direct")]))
        .execute();

    assert_eq!(
        report.source_errors().len(),
        1,
        "{:?}",
        report.source_errors()
    );
    assert!(
        report
            .issues()
            .iter()
            .any(|issue| issue.kind() == AnalysisIssueKind::Read),
        "{:?}",
        report.issues()
    );
}

#[test]
fn an_unknown_method_selection_is_returned_unchanged() {
    let project = control(&[]);
    let processor = project_type(&project, "Processor");
    let target = Target::method(&processor, "absent");

    let report = CheckProject::new(&project, &project)
        .with_selection(Selection::of(vec![target.clone()]))
        .execute();

    assert_eq!(report.unmatched_targets(), [target]);
    assert!(report.findings().is_empty());
    assert!(report.issues().is_empty());
}

#[test]
fn an_unsupported_active_body_has_an_issue_instead_of_a_clean_empty_method() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/rust-scenarios");
    let project = RustProject::load(
        &root.join("Cargo.toml"),
        LoadOptions::new("rust-scenario-core", &output("unsupported")),
    )
    .expect("Cargo source loads");
    let service = project_type(&project, "PaymentService");

    let report = CheckProject::new(&project, &project)
        .with_selection(Selection::of(vec![Target::method(&service, "on_created")]))
        .execute();

    assert_eq!(report.counts().methods, 0);
    assert!(report.findings().is_empty());
    assert_eq!(report.issues().len(), 1, "{:?}", report.issues());
    assert_eq!(report.issues()[0].kind(), AnalysisIssueKind::Unsupported);
    assert_eq!(report.issues()[0].line(), Some(21));
}

#[test]
fn an_absent_manifest_returns_a_read_failure() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/absent/Cargo.toml");

    let result = RustProject::load(&path, LoadOptions::new("absent", &output("absent")));

    let error = result.err().expect("loading an absent manifest must fail");
    assert_eq!(error.path(), path);
    assert!(!error.message().is_empty());
}

#[test]
fn invalid_module_syntax_produces_a_parse_issue_and_source_error() {
    let temporary = tempfile::tempdir().expect("temporary fixture directory");
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/control");
    for file in ["Cargo.toml", "Cargo.lock", "lib.rs"] {
        std::fs::copy(fixture.join(file), temporary.path().join(file))
            .expect("fixture copy succeeds");
    }
    std::fs::write(temporary.path().join("parcel.rs"), "pub struct Account {\n")
        .expect("invalid source is writable");
    let project = RustProject::load(
        &temporary.path().join("Cargo.toml"),
        LoadOptions::new("source-control", &output("parse")),
    )
    .expect("Cargo metadata still loads");

    let report = CheckProject::new(&project, &project).execute();

    assert!(
        report
            .issues()
            .iter()
            .any(|issue| issue.kind() == AnalysisIssueKind::Parse
                && issue.path().file_name().and_then(|name| name.to_str()) == Some("parcel.rs")),
        "{:?}",
        report.issues()
    );
    assert!(!report.source_errors().is_empty());
}

#[test]
fn an_active_free_function_is_reported_without_a_synthetic_self_type() {
    let project = control(&["free-function"]);

    let report = CheckProject::new(&project, &project).execute();

    assert!(
        report
            .issues()
            .iter()
            .any(|issue| issue.kind() == AnalysisIssueKind::Unsupported
                && issue.line() == Some(35)
                && issue.owner().is_none()),
        "{:?}",
        report.issues()
    );
    assert!(
        !report
            .findings()
            .iter()
            .any(|finding| finding.method_name() == "free_call")
    );
}

#[test]
fn an_unsupported_aggregate_remains_a_project_type_with_an_explicit_issue() {
    let project = control(&["aggregate"]);

    let report = CheckProject::new(&project, &project).execute();

    assert!(
        project
            .project_types()
            .iter()
            .any(|name| name.ends_with("::Choice")),
        "{:?}",
        project.project_types()
    );
    assert!(
        report
            .issues()
            .iter()
            .any(|issue| issue.kind() == AnalysisIssueKind::Unsupported
                && issue
                    .owner()
                    .is_some_and(|owner| owner.class_name().ends_with("::Choice"))),
        "{:?}",
        report.issues()
    );
}

fn project_type(project: &RustProject, name: &str) -> String {
    project
        .project_types()
        .into_iter()
        .find(|candidate| candidate.ends_with(&format!("::{name}")))
        .expect("declared project type exists")
}

fn output(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/rust-source-journey/analyzed")
        .join(name)
}

fn control(features: &[&str]) -> RustProject {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/control");
    RustProject::load(
        &root.join("Cargo.toml"),
        LoadOptions::new(
            "source-control",
            &output(&format!("control-{}", features.join("-"))),
        )
        .with_features(
            features
                .iter()
                .map(|feature| (*feature).to_owned())
                .collect(),
            false,
        ),
    )
    .expect("controlled Cargo source loads")
}
