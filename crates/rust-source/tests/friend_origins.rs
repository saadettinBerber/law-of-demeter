use std::path::{Path, PathBuf};
use std::process::Command;

use law_of_demeter_core::application::{CheckProject, Report, Selection, Target, TypeSource};
use law_of_demeter_core::domain::method::{Reason, Verdict};
use law_of_demeter_rust_source::{LoadOptions, RustProject};

#[test]
fn forwarding_to_the_order_is_allowed_while_reaching_its_customer_and_wallet_is_reported() {
    let project = load_fixture();

    let report = check(&project, &["direct", "through_chain"]);

    assert_eq!(
        verdicts(&report),
        [
            (
                "direct".into(),
                Verdict::allowed("charge", 46, Reason::parameter("order"))
            ),
            (
                "through_chain".into(),
                Verdict::allowed("customer", 50, Reason::parameter("order"))
            ),
            (
                "through_chain".into(),
                Verdict::violation("wallet", 50, Reason::return_of("customer"))
            ),
            (
                "through_chain".into(),
                Verdict::violation("withdraw", 50, Reason::return_of("wallet"))
            ),
        ]
    );
    assert_complete(&report);
    assert!(
        report
            .findings()
            .iter()
            .all(|finding| finding.path() == fixture().join("lib.rs")
                && finding.class_name() == project_type(&project, "PaymentService"))
    );
}

#[test]
fn self_held_fields_and_parameters_are_friends_but_another_instances_field_is_not() {
    let project = load_fixture();

    let report = check(
        &project,
        &["on_self", "on_held_field", "on_parameter", "on_peer_field"],
    );

    assert_eq!(
        verdicts(&report),
        [
            (
                "on_self".into(),
                Verdict::allowed("debit", 56, Reason::This)
            ),
            (
                "on_held_field".into(),
                Verdict::allowed("withdraw", 64, Reason::field("wallet"))
            ),
            (
                "on_parameter".into(),
                Verdict::allowed("withdraw", 68, Reason::parameter("wallet"))
            ),
            (
                "on_peer_field".into(),
                Verdict::violation("withdraw", 72, Reason::field_of("wallet"))
            ),
        ]
    );
    assert_complete(&report);
}

fn load_fixture() -> RustProject {
    let compiled = Command::new("cargo")
        .args(["check", "--offline", "--locked", "--manifest-path"])
        .arg(fixture().join("Cargo.toml"))
        .arg("--lib")
        .env("CARGO_TARGET_DIR", output().join("compiler"))
        .output()
        .expect("Cargo checks the friend fixture");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    RustProject::load(
        &fixture().join("Cargo.toml"),
        LoadOptions::new("friend-origins", &output().join("analysis")),
    )
    .expect("compiler-valid friend source loads")
}

fn check(project: &RustProject, methods: &[&str]) -> Report {
    let service = project_type(project, "PaymentService");
    let selection = Selection::of(
        methods
            .iter()
            .map(|method| Target::method(&service, method))
            .collect(),
    );
    CheckProject::new(project, project)
        .with_selection(selection)
        .execute()
}

fn verdicts(report: &Report) -> Vec<(String, Verdict)> {
    report
        .findings()
        .iter()
        .map(|finding| (finding.method_name().to_owned(), finding.verdict().clone()))
        .collect()
}

fn assert_complete(report: &Report) {
    assert!(report.issues().is_empty(), "{report:?}");
    assert!(report.source_errors().is_empty(), "{report:?}");
    assert!(report.type_errors().is_empty(), "{report:?}");
    assert!(report.unmatched_targets().is_empty(), "{report:?}");
    assert_eq!(report.counts().unresolved, 0, "{report:?}");
}

fn project_type(project: &RustProject, name: &str) -> String {
    project
        .project_types()
        .into_iter()
        .find(|candidate| candidate.ends_with(&format!("::{name}")))
        .expect("project type is declared")
}

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/friends")
        .canonicalize()
        .expect("friend fixture exists")
}

fn output() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/rust-friends-journey/analyzed/friends")
}
