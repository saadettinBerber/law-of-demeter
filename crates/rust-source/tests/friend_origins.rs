use std::path::{Path, PathBuf};
use std::process::Command;

use law_of_demeter_core::application::{
    AnalysisIssueKind, CheckProject, Report, Selection, Target, TypeSource,
};
use law_of_demeter_core::domain::method::{Reason, Verdict};
use law_of_demeter_core::domain::type_shape::TypeRef;
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

#[test]
fn named_and_tuple_struct_locals_are_created_friends_and_keep_initializer_calls() {
    let project = load_fixture();
    let wallet = TypeRef::class(&project_type(&project, "Wallet"));
    let pocket = TypeRef::class(&project_type(&project, "Pocket"));

    let report = check(&project, &["create_named", "create_tuple"]);

    assert_eq!(
        verdicts(&report),
        [
            (
                "create_named".into(),
                Verdict::allowed("amount", 105, Reason::parameter("seed"))
            ),
            (
                "create_named".into(),
                Verdict::allowed(
                    "withdraw",
                    107,
                    Reason::local("wallet", Reason::created(wallet))
                )
            ),
            (
                "create_tuple".into(),
                Verdict::allowed("amount", 111, Reason::parameter("seed"))
            ),
            (
                "create_tuple".into(),
                Verdict::allowed(
                    "withdraw",
                    112,
                    Reason::local("pocket", Reason::created(pocket))
                )
            ),
        ]
    );
    assert_complete(&report);
}

#[test]
fn a_created_outer_holder_does_not_make_its_existing_wallet_a_created_friend() {
    let project = load_fixture();

    let report = check(&project, &["new_holder"]);

    assert_eq!(
        verdicts(&report),
        [(
            "new_holder".into(),
            Verdict::violation("withdraw", 117, Reason::field_of("wallet"))
        )]
    );
    assert_complete(&report);
}

#[test]
fn an_ordinary_associated_function_is_not_treated_as_a_constructor() {
    let project = load_fixture();

    let report = check(&project, &["from_function"]);

    assert!(
        report
            .issues()
            .iter()
            .any(|issue| issue.kind() == AnalysisIssueKind::Unsupported
                && issue.path() == fixture().join("lib.rs")
                && issue.line() == Some(121)
                && issue.owner()
                    == Some(&Target::method(
                        &project_type(&project, "PaymentService"),
                        "from_function"
                    ))),
        "{report:?}"
    );
    assert!(report.findings().is_empty(), "{report:?}");
}

#[test]
fn locals_and_normal_borrows_keep_the_chain_receivers_origin() {
    let project = load_fixture();

    let report = check(
        &project,
        &[
            "through_local",
            "through_reborrow",
            "parameter_reborrow",
            "shared_borrow",
        ],
    );

    assert_eq!(
        verdicts(&report),
        [
            (
                "through_local".into(),
                Verdict::allowed("customer", 168, Reason::parameter("order"))
            ),
            (
                "through_local".into(),
                Verdict::violation(
                    "wallet",
                    169,
                    Reason::local("customer", Reason::return_of("customer"))
                )
            ),
            (
                "through_local".into(),
                Verdict::violation(
                    "withdraw",
                    170,
                    Reason::local("wallet", Reason::return_of("wallet"))
                )
            ),
            (
                "through_reborrow".into(),
                Verdict::allowed("customer", 174, Reason::parameter("order"))
            ),
            (
                "through_reborrow".into(),
                Verdict::violation(
                    "wallet",
                    176,
                    Reason::local(
                        "temporary",
                        Reason::local("customer", Reason::return_of("customer"))
                    )
                )
            ),
            (
                "through_reborrow".into(),
                Verdict::violation(
                    "withdraw",
                    177,
                    Reason::local("wallet", Reason::return_of("wallet"))
                )
            ),
            (
                "parameter_reborrow".into(),
                Verdict::allowed(
                    "activate",
                    182,
                    Reason::local("temporary", Reason::parameter("customer"))
                )
            ),
            (
                "shared_borrow".into(),
                Verdict::allowed("wallet_shared", 253, Reason::parameter("order"))
            ),
            (
                "shared_borrow".into(),
                Verdict::violation(
                    "balance",
                    255,
                    Reason::local(
                        "borrowed",
                        Reason::local("wallet", Reason::return_of("wallet_shared"))
                    )
                )
            ),
        ]
    );
    assert_complete(&report);
}

#[test]
fn moving_owned_values_and_copying_values_or_references_do_not_wash_their_origin() {
    let project = load_fixture();

    let report = check(
        &project,
        &["shared_copy", "owned_move", "moved_parameter", "copy_value"],
    );

    assert_eq!(
        verdicts(&report),
        [
            (
                "shared_copy".into(),
                Verdict::allowed("wallet_shared", 186, Reason::parameter("order"))
            ),
            (
                "shared_copy".into(),
                Verdict::violation(
                    "balance",
                    188,
                    Reason::local(
                        "copied",
                        Reason::local("wallet", Reason::return_of("wallet_shared"))
                    )
                )
            ),
            (
                "owned_move".into(),
                Verdict::allowed("into_wallet", 192, Reason::parameter("order"))
            ),
            (
                "owned_move".into(),
                Verdict::violation(
                    "withdraw",
                    194,
                    Reason::local(
                        "moved",
                        Reason::local("wallet", Reason::return_of("into_wallet"))
                    )
                )
            ),
            (
                "moved_parameter".into(),
                Verdict::allowed(
                    "withdraw",
                    199,
                    Reason::local("moved", Reason::parameter("wallet"))
                )
            ),
            (
                "copy_value".into(),
                Verdict::allowed("ticket", 289, Reason::parameter("dispatch"))
            ),
            (
                "copy_value".into(),
                Verdict::violation(
                    "stamp",
                    291,
                    Reason::local(
                        "copied",
                        Reason::local("ticket", Reason::return_of("ticket"))
                    )
                )
            ),
        ]
    );
    assert_complete(&report);
}

#[test]
fn reference_aliases_preserve_self_identity_and_genuine_creation() {
    let project = load_fixture();
    let wallet = TypeRef::class(&project_type(&project, "Wallet"));

    let report = check(&project, &["alias_self", "alias_peer", "create_borrowed"]);

    assert_eq!(
        verdicts(&report),
        [
            (
                "alias_self".into(),
                Verdict::allowed("withdraw", 204, Reason::field("wallet"))
            ),
            (
                "alias_peer".into(),
                Verdict::violation("withdraw", 209, Reason::field_of("wallet"))
            ),
            (
                "create_borrowed".into(),
                Verdict::allowed("amount", 214, Reason::parameter("seed"))
            ),
            (
                "create_borrowed".into(),
                Verdict::allowed(
                    "withdraw",
                    216,
                    Reason::local("wallet", Reason::created(wallet))
                )
            ),
        ]
    );
    assert_complete(&report);
}

#[test]
fn custom_deref_coercions_and_raw_pointer_origins_stay_explicitly_unsupported() {
    let project = load_fixture();

    let report = check(
        &project,
        &["custom_projection", "coerced_projection", "raw_projection"],
    );

    let owners: Vec<_> = report
        .issues()
        .iter()
        .map(|issue| {
            (
                issue.kind(),
                issue.owner().and_then(Target::method_name),
                issue.line(),
            )
        })
        .collect();
    assert_eq!(
        owners,
        [
            (
                AnalysisIssueKind::Unsupported,
                Some("custom_projection"),
                Some(220)
            ),
            (
                AnalysisIssueKind::Unsupported,
                Some("coerced_projection"),
                Some(225)
            ),
            (
                AnalysisIssueKind::Unsupported,
                Some("raw_projection"),
                Some(230)
            ),
        ],
        "{report:?}"
    );
    assert!(report.issues().iter().all(|issue| {
        issue.path() == fixture().join("lib.rs")
            && issue
                .owner()
                .is_some_and(|owner| owner.class_name() == project_type(&project, "PaymentService"))
    }));
    assert_eq!(report.counts().methods, 0);
    assert!(report.findings().is_empty());
    assert!(report.source_errors().is_empty());
    assert!(report.type_errors().is_empty());
}

#[test]
fn shadowing_and_reassignment_are_not_reported_using_a_stale_local_origin() {
    let project = load_fixture();

    let report = check(&project, &["shadowed", "reassigned"]);

    let owners: Vec<_> = report
        .issues()
        .iter()
        .map(|issue| {
            (
                issue.kind(),
                issue.owner().and_then(Target::method_name),
                issue.line(),
            )
        })
        .collect();
    assert_eq!(
        owners,
        [
            (AnalysisIssueKind::Unsupported, Some("shadowed"), Some(239)),
            (
                AnalysisIssueKind::Unsupported,
                Some("reassigned"),
                Some(246)
            ),
        ],
        "{report:?}"
    );
    assert!(report.issues().iter().all(|issue| {
        issue.path() == fixture().join("lib.rs")
            && issue
                .owner()
                .is_some_and(|owner| owner.class_name() == project_type(&project, "PaymentService"))
    }));
    assert_eq!(report.counts().methods, 0);
    assert!(report.findings().is_empty());
    assert!(report.source_errors().is_empty());
    assert!(report.type_errors().is_empty());
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
