use std::cell::RefCell;
use std::path::Path;

use super::{
    FakeSources, FakeTypes, order_service_sources, pay_method, run_method, shop_types,
    types_nesting_with,
};
use crate::application::{
    AnalysisIssue, AnalysisIssueKind, CheckProject, Finding, ReadError, Report, Selection,
    SourceFile, SourceReader, Target, TypeReading, TypeSource,
};
use crate::domain::classification::Types;
use crate::domain::method::{Call, Expression, Method, Reason, Verdict};
use crate::domain::type_shape::TypeRef;

#[test]
fn a_partial_source_keeps_its_known_violation_and_unresolved_target_issue() {
    let path = Path::new("src/OrderService.rs");
    let sources = PartialSources {
        files: FakeSources(vec![Ok(SourceFile::new(
            path,
            vec!["OrderService".to_owned(), "Customer".to_owned()],
            vec![pay_method()],
        ))]),
        issues: vec![
            AnalysisIssue::new(
                AnalysisIssueKind::TargetResolution,
                ReadError::at_line(path, 12, "cannot resolve missing_debit"),
            )
            .in_body(Target::method("OrderService", "pay")),
        ],
    };

    let report = CheckProject::new(&sources, &shop_types()).execute();

    assert_eq!(report.counts().violations, 1);
    assert_eq!(
        report.issues(),
        [AnalysisIssue::new(
            AnalysisIssueKind::TargetResolution,
            ReadError::at_line(path, 12, "cannot resolve missing_debit"),
        )
        .in_body(Target::method("OrderService", "pay"))]
    );
}

#[test]
fn selecting_a_method_keeps_its_body_issue_without_requiring_a_readable_body() {
    let sources = sources_with_issues(vec![
        target_issue("OrderService", "pay"),
        target_issue("OrderService", "refund"),
    ]);

    let report = CheckProject::new(&sources, &shop_types())
        .with_selection(Selection::of(vec![Target::method("OrderService", "pay")]))
        .execute();

    assert_eq!(report.issues(), [target_issue("OrderService", "pay")]);
    assert_eq!(report.counts().methods, 0);
}

#[test]
fn selecting_a_class_keeps_its_nested_body_issues_and_excludes_other_classes() {
    let sources = sources_with_issues(vec![
        target_issue("OrderService", "pay"),
        target_issue("OrderService::Worker", "run"),
        target_issue("Reports", "report"),
    ]);

    let report = CheckProject::new(&sources, &types_nesting_with("::"))
        .with_selection(Selection::of(vec![Target::class("OrderService")]))
        .execute();

    assert_eq!(
        report.issues(),
        [
            target_issue("OrderService", "pay"),
            target_issue("OrderService::Worker", "run"),
        ]
    );
}

#[test]
fn selecting_a_method_excludes_the_same_named_body_of_a_nested_class() {
    let sources = sources_with_issues(vec![
        target_issue("OrderService", "pay"),
        target_issue("OrderService::Worker", "pay"),
    ]);

    let report = CheckProject::new(&sources, &types_nesting_with("::"))
        .with_selection(Selection::of(vec![Target::method("OrderService", "pay")]))
        .execute();

    assert_eq!(report.issues(), [target_issue("OrderService", "pay")]);
}

#[test]
fn selecting_a_method_keeps_issues_of_bodies_declared_within_it() {
    let sources = PartialSources {
        files: FakeSources(vec![Ok(SourceFile::new(
            Path::new("src/OrderService.rs"),
            vec!["OrderService".to_owned(), "OrderService::Worker".to_owned()],
            vec![run_method("OrderService::Worker").declared_within("OrderService", "pay")],
        ))]),
        issues: vec![target_issue("OrderService::Worker", "run")],
    };

    let report = CheckProject::new(&sources, &types_nesting_with("::"))
        .with_selection(Selection::of(vec![Target::method("OrderService", "pay")]))
        .execute();

    assert_eq!(
        report.issues(),
        [target_issue("OrderService::Worker", "run")]
    );
}

#[test]
fn selecting_a_method_excludes_class_body_issues() {
    let sources = sources_with_issues(vec![
        target_issue("OrderService", "pay"),
        AnalysisIssue::new(
            AnalysisIssueKind::Unsupported,
            ReadError::new(Path::new("src/OrderService.rs"), "unsupported body"),
        )
        .in_body(Target::class("OrderService")),
    ]);

    let report = CheckProject::new(&sources, &shop_types())
        .with_selection(Selection::of(vec![Target::method("OrderService", "pay")]))
        .execute();

    assert_eq!(report.issues(), [target_issue("OrderService", "pay")]);
}

#[test]
fn project_issues_keep_their_kind_and_location_outside_the_selection() {
    let sources = sources_with_issues(vec![
        AnalysisIssue::new(
            AnalysisIssueKind::Read,
            ReadError::at_line(Path::new("src/Reports.rs"), 7, "cannot read module"),
        ),
        AnalysisIssue::new(
            AnalysisIssueKind::Build,
            ReadError::new(Path::new("Cargo.toml"), "build failed"),
        ),
    ]);

    let report = CheckProject::new(&sources, &shop_types())
        .with_selection(Selection::of(vec![Target::method("OrderService", "pay")]))
        .execute();

    let issues: Vec<_> = report
        .issues()
        .iter()
        .map(|issue| {
            (
                issue.kind(),
                issue.path(),
                issue.line(),
                issue.message(),
                issue.owner(),
            )
        })
        .collect();
    assert_eq!(
        issues,
        [
            (
                AnalysisIssueKind::Read,
                Path::new("src/Reports.rs"),
                Some(7),
                "cannot read module",
                None
            ),
            (
                AnalysisIssueKind::Build,
                Path::new("Cargo.toml"),
                None,
                "build failed",
                None
            ),
        ]
    );
}

#[test]
fn a_known_project_type_from_an_unreadable_source_does_not_gain_the_library_exemption() {
    let path = Path::new("src/OrderService.rs");
    let sources = FakeSources(vec![
        Ok(SourceFile::new(
            path,
            vec!["OrderService".to_owned()],
            vec![pay_method()],
        )),
        Err(ReadError::new(
            Path::new("src/Customer.rs"),
            "cannot read module",
        )),
    ]);
    let types = CataloguedTypes {
        types: shop_types(),
        names: vec!["Customer".to_owned()],
    };

    let report = CheckProject::new(&sources, &types)
        .with_selection(Selection::of(vec![Target::method("OrderService", "pay")]))
        .execute();

    assert_eq!(
        report.findings()[1],
        Finding::new(
            path,
            "OrderService",
            "pay",
            Verdict::violation("getWallet", 11, Reason::return_of("getCustomer")),
        )
    );
    assert_eq!(
        report.source_errors(),
        [ReadError::new(
            Path::new("src/Customer.rs"),
            "cannot read module"
        )]
    );
}

#[test]
fn project_membership_reaches_the_source_reader_without_inventing_a_type_shape() {
    let sources = InspectingSources(RefCell::new(Vec::new()));
    let types = CataloguedTypes {
        types: FakeTypes::new(Vec::new()),
        names: vec!["Wallet".to_owned()],
    };

    CheckProject::new(&sources, &types).execute();

    assert_eq!(*sources.0.borrow(), [(true, false)]);
}

#[test]
fn a_known_project_type_without_a_shape_stays_unresolved() {
    let path = Path::new("src/OrderService.rs");
    let sources = FakeSources(vec![Ok(SourceFile::new(
        path,
        vec!["OrderService".to_owned()],
        vec![
            Method::new("OrderService", "pay", 10).call(
                Call::new(
                    Expression::parameter("order", TypeRef::class("Order")),
                    "wallet",
                    11,
                )
                .returning(TypeRef::class("Wallet"))
                .then("missing_debit", 12),
            ),
        ],
    ))]);
    let types = CataloguedTypes {
        types: FakeTypes::new(Vec::new()),
        names: vec!["Wallet".to_owned()],
    };

    let report = CheckProject::new(&sources, &types).execute();

    assert_eq!(
        report.findings()[1],
        Finding::new(
            path,
            "OrderService",
            "pay",
            Verdict::violation(
                "missing_debit",
                12,
                Reason::unresolved_type(
                    Some(TypeRef::class("Wallet")),
                    Reason::return_of("wallet"),
                )
            ),
        )
    );
    assert_eq!(report.counts().unresolved, 1);
}

#[test]
fn readers_without_issue_or_project_catalog_support_preserve_the_legacy_report() {
    let sources = order_service_sources();

    let report = CheckProject::new(&sources, &shop_types()).execute();

    assert_eq!(
        report,
        Report::new(
            1,
            1,
            vec![
                Finding::new(
                    Path::new("src/OrderService.java"),
                    "OrderService",
                    "pay",
                    Verdict::allowed("getCustomer", 11, Reason::parameter("order"))
                ),
                Finding::new(
                    Path::new("src/OrderService.java"),
                    "OrderService",
                    "pay",
                    Verdict::allowed("getWallet", 11, Reason::library(TypeRef::class("Customer")))
                ),
            ],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
    );
}

fn sources_with_issues(issues: Vec<AnalysisIssue>) -> PartialSources {
    PartialSources {
        files: FakeSources(Vec::new()),
        issues,
    }
}

fn target_issue(class: &str, method: &str) -> AnalysisIssue {
    AnalysisIssue::new(
        AnalysisIssueKind::TargetResolution,
        ReadError::at_line(Path::new("src/OrderService.rs"), 12, "unresolved target"),
    )
    .in_body(Target::method(class, method))
}

struct PartialSources {
    files: FakeSources,
    issues: Vec<AnalysisIssue>,
}

impl SourceReader for PartialSources {
    fn read(&self, types: &Types) -> Vec<Result<SourceFile, ReadError>> {
        self.files.read(types)
    }

    fn issues(&self) -> Vec<AnalysisIssue> {
        self.issues.clone()
    }
}

struct CataloguedTypes {
    types: FakeTypes,
    names: Vec<String>,
}

impl TypeSource for CataloguedTypes {
    fn read(&self) -> TypeReading {
        self.types.read()
    }

    fn project_types(&self) -> Vec<String> {
        self.names.clone()
    }
}

struct InspectingSources(RefCell<Vec<(bool, bool)>>);

impl SourceReader for InspectingSources {
    fn read(&self, types: &Types) -> Vec<Result<SourceFile, ReadError>> {
        self.0.borrow_mut().push((
            types.is_project_type("Wallet"),
            types.shape("Wallet").is_some(),
        ));
        Vec::new()
    }
}
