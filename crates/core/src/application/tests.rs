mod analysis_issues;

use std::cell::RefCell;
use std::path::Path;

use super::{
    CheckProject, Counts, Finding, HybridFinding, ReadError, Report, Selection, SourceFile,
    SourceReader, Target, TypeReading, TypeSource,
};
use crate::domain::classification::{Classification, Platform, Types};
use crate::domain::method::{Call, Expression, Method, Reason, Verdict};
use crate::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

struct FakeSources(Vec<Result<SourceFile, ReadError>>);

impl SourceReader for FakeSources {
    fn read(&self, _types: &Types) -> Vec<Result<SourceFile, ReadError>> {
        self.0.clone()
    }
}

struct FakeTypes {
    shapes: Vec<TypeShape>,
    errors: Vec<ReadError>,
    platform: Platform,
}

impl FakeTypes {
    fn new(shapes: Vec<TypeShape>) -> Self {
        Self {
            shapes,
            errors: Vec::new(),
            platform: Platform::new(),
        }
    }
}

impl TypeSource for FakeTypes {
    fn read(&self) -> TypeReading {
        TypeReading {
            shapes: self.shapes.clone(),
            errors: self.errors.clone(),
            platform: self.platform.clone(),
        }
    }
}

struct AskingSources(RefCell<Vec<bool>>);

impl SourceReader for AskingSources {
    fn read(&self, types: &Types) -> Vec<Result<SourceFile, ReadError>> {
        self.0.borrow_mut().extend([
            types.shape("Order").is_some(),
            types.shape("Missing").is_some(),
        ]);
        Vec::new()
    }
}

fn shop_types() -> FakeTypes {
    FakeTypes::new(vec![
        TypeShape::new("Order", TypeKind::Class).with_method(
            MethodShape::new("getCustomer")
                .reading("customer")
                .returning(TypeRef::class("Customer")),
        ),
        TypeShape::new("Customer", TypeKind::Class)
            .with_method(
                MethodShape::new("getWallet")
                    .reading("wallet")
                    .returning(TypeRef::class("Wallet")),
            )
            .with_method(MethodShape::new("rename")),
        TypeShape::new("Wallet", TypeKind::Class).with_method(MethodShape::new("withdraw")),
    ])
}

fn pay_method() -> Method {
    Method::new("OrderService", "pay", 10).call(
        Call::new(
            Expression::parameter("order", TypeRef::class("Order")),
            "getCustomer",
            11,
        )
        .returning(TypeRef::class("Customer"))
        .then("getWallet", 11),
    )
}

#[test]
fn every_call_of_every_method_becomes_a_finding_with_its_location() {
    let path = Path::new("src/OrderService.java");
    let sources = FakeSources(vec![Ok(SourceFile::new(
        path,
        vec![
            "OrderService".to_owned(),
            "Order".to_owned(),
            "Customer".to_owned(),
        ],
        vec![pay_method()],
    ))]);

    let report = CheckProject::new(&sources, &shop_types()).execute();

    assert_eq!(
        report.findings(),
        [
            Finding::new(
                path,
                "OrderService",
                "pay",
                Verdict::allowed("getCustomer", 11, Reason::parameter("order"))
            ),
            Finding::new(
                path,
                "OrderService",
                "pay",
                Verdict::violation("getWallet", 11, Reason::return_of("getCustomer"))
            ),
        ]
    );
}

fn account_shape() -> TypeShape {
    TypeShape::new("Account", TypeKind::Class)
        .with_method(
            MethodShape::new("getBalance")
                .reading("balance")
                .returning(TypeRef::primitive("int")),
        )
        .with_method(
            MethodShape::new("setBalance")
                .writing("balance")
                .with_parameters(vec![TypeRef::primitive("int")]),
        )
        .with_method(MethodShape::new("withdraw"))
}

#[test]
fn a_declared_class_that_is_a_hybrid_becomes_a_hybrid_finding() {
    let path = Path::new("src/Account.java");
    let sources = FakeSources(vec![Ok(SourceFile::new(
        path,
        vec!["Account".to_owned(), "Unknown".to_owned()],
        Vec::new(),
    ))]);
    let hybrid = Types::new()
        .with_shape(account_shape())
        .hybrid_class("Account")
        .expect("Account is a hybrid");

    let report = CheckProject::new(&sources, &FakeTypes::new(vec![account_shape()])).execute();

    assert_eq!(report.hybrid_classes(), [HybridFinding::new(path, hybrid)]);
}

#[test]
fn the_source_reader_is_told_which_types_the_catalog_knows() {
    let sources = AskingSources(RefCell::new(Vec::new()));

    CheckProject::new(&sources, &shop_types()).execute();

    assert_eq!(*sources.0.borrow(), [true, false]);
}

struct ClassifyingSources(RefCell<Vec<Classification>>);

impl SourceReader for ClassifyingSources {
    fn read(&self, types: &Types) -> Vec<Result<SourceFile, ReadError>> {
        self.0
            .borrow_mut()
            .push(types.classify(&TypeRef::class("Money")));
        Vec::new()
    }
}

#[test]
fn the_source_reader_is_given_what_the_platform_knows() {
    let sources = ClassifyingSources(RefCell::new(Vec::new()));
    let types = FakeTypes {
        platform: Platform::new().with_type("Money", Classification::Value),
        ..shop_types()
    };

    CheckProject::new(&sources, &types).execute();

    assert_eq!(*sources.0.borrow(), [Classification::Value]);
}

#[test]
fn read_errors_are_reported_and_the_other_files_are_still_judged() {
    let good = Path::new("src/OrderService.java");
    let bad = Path::new("src/Broken.java");
    let sources = FakeSources(vec![
        Err(ReadError::at_line(bad, 3, "beklenmeyen simge")),
        Ok(SourceFile::new(good, Vec::new(), vec![pay_method()])),
    ]);
    let types = FakeTypes {
        errors: vec![ReadError::new(Path::new("lib/broken.jar"), "bozuk arşiv")],
        ..shop_types()
    };

    let report = CheckProject::new(&sources, &types).execute();

    assert_eq!(
        (
            report.source_errors(),
            report.type_errors(),
            report.findings().len()
        ),
        (
            [ReadError::at_line(bad, 3, "beklenmeyen simge")].as_slice(),
            [ReadError::new(Path::new("lib/broken.jar"), "bozuk arşiv")].as_slice(),
            2
        )
    );
}

#[test]
fn the_counts_separate_real_violations_from_unresolved_types() {
    let sources = FakeSources(vec![
        Ok(SourceFile::new(
            Path::new("src/OrderService.java"),
            vec![
                "OrderService".to_owned(),
                "Order".to_owned(),
                "Customer".to_owned(),
            ],
            vec![pay_method()],
        )),
        Ok(SourceFile::new(
            Path::new("src/Account.java"),
            vec!["Account".to_owned()],
            vec![
                Method::new("Reports", "report", 5).call(
                    Call::new(
                        Expression::parameter("order", TypeRef::class("Order")),
                        "getAddress",
                        6,
                    )
                    .then("getCity", 6),
                ),
            ],
        )),
        Err(ReadError::at_line(Path::new("src/Broken.java"), 3, "hata")),
    ]);
    let mut shapes = shop_types().shapes;
    shapes.push(account_shape());
    let types = FakeTypes {
        shapes,
        errors: vec![ReadError::new(Path::new("lib/broken.jar"), "bozuk arşiv")],
        platform: Platform::new(),
    };

    let counts = CheckProject::new(&sources, &types).execute().counts();

    assert_eq!(
        counts,
        Counts {
            files: 2,
            methods: 2,
            calls: 4,
            allowed: 2,
            violations: 1,
            unresolved: 1,
            hybrid_classes: 1,
            source_errors: 1,
            type_errors: 1,
        }
    );
}

#[test]
fn a_report_built_from_its_parts_counts_them() {
    let path = Path::new("src/OrderService.java");
    let finding = |verdict| Finding::new(path, "OrderService", "pay", verdict);
    let report = Report::new(
        2,
        3,
        vec![
            finding(Verdict::allowed(
                "getCustomer",
                3,
                Reason::parameter("order"),
            )),
            finding(Verdict::violation(
                "getWallet",
                3,
                Reason::return_of("getCustomer"),
            )),
        ],
        Vec::new(),
        vec![ReadError::new(path, "hata")],
        Vec::new(),
    );

    let counts = report.counts();

    assert_eq!(
        counts,
        Counts {
            files: 2,
            methods: 3,
            calls: 2,
            allowed: 1,
            violations: 1,
            source_errors: 1,
            ..Counts::default()
        }
    );
}

#[test]
fn a_call_on_a_type_no_source_declares_is_a_library_call() {
    let path = Path::new("src/OrderService.java");
    let sources = FakeSources(vec![Ok(SourceFile::new(
        path,
        vec!["OrderService".to_owned(), "Order".to_owned()],
        vec![pay_method()],
    ))]);

    let report = CheckProject::new(&sources, &shop_types()).execute();

    assert_eq!(
        report.findings()[1],
        Finding::new(
            path,
            "OrderService",
            "pay",
            Verdict::allowed("getWallet", 11, Reason::library(TypeRef::class("Customer")))
        )
    );
}

#[test]
fn a_violation_whose_standing_traces_back_to_an_unresolved_name_counts_as_unresolved() {
    let sources = FakeSources(vec![Ok(SourceFile::new(
        Path::new("src/Reports.java"),
        vec!["Reports".to_owned(), "Customer".to_owned()],
        vec![
            Method::new("Reports", "rename", 1)
                .assign(
                    "customer",
                    TypeRef::class("Customer"),
                    Expression::unresolved("loaded"),
                )
                .call(Call::new(Expression::local("customer"), "rename", 3)),
        ],
    ))]);

    let counts = CheckProject::new(&sources, &shop_types())
        .execute()
        .counts();

    assert_eq!((counts.violations, counts.unresolved), (0, 1));
}

#[test]
fn what_the_type_source_knows_about_a_type_decides_its_classification() {
    let path = Path::new("src/OrderService.java");
    let sources = FakeSources(vec![Ok(SourceFile::new(
        path,
        vec![
            "OrderService".to_owned(),
            "Order".to_owned(),
            "Customer".to_owned(),
        ],
        vec![pay_method()],
    ))]);
    let types = FakeTypes {
        platform: Platform::new().with_type("Customer", Classification::DataStructure),
        ..shop_types()
    };

    let report = CheckProject::new(&sources, &types).execute();

    assert_eq!(
        report.findings()[1],
        Finding::new(
            path,
            "OrderService",
            "pay",
            Verdict::allowed(
                "getWallet",
                11,
                Reason::classified(Classification::DataStructure, TypeRef::class("Customer"))
            )
        )
    );
}

fn report_method() -> Method {
    Method::new("Reports", "report", 5).call(Call::new(
        Expression::parameter("order", TypeRef::class("Order")),
        "getCustomer",
        6,
    ))
}

fn refund_method() -> Method {
    Method::new("OrderService", "refund", 20).call(Call::new(
        Expression::parameter("order", TypeRef::class("Order")),
        "getCustomer",
        21,
    ))
}

#[test]
fn only_the_methods_of_a_selected_class_are_judged() {
    let sources = FakeSources(vec![
        Ok(SourceFile::new(
            Path::new("src/OrderService.java"),
            vec!["OrderService".to_owned()],
            vec![pay_method()],
        )),
        Ok(SourceFile::new(
            Path::new("src/Reports.java"),
            vec!["Reports".to_owned()],
            vec![report_method()],
        )),
    ]);

    let report = CheckProject::new(&sources, &shop_types())
        .with_selection(Selection::of(vec![Target::class("OrderService")]))
        .execute();

    let judged: Vec<&str> = report.findings().iter().map(Finding::class_name).collect();
    assert_eq!(judged, ["OrderService", "OrderService"]);
}

#[test]
fn only_the_selected_method_of_a_class_is_judged() {
    let sources = FakeSources(vec![Ok(SourceFile::new(
        Path::new("src/OrderService.java"),
        vec!["OrderService".to_owned()],
        vec![pay_method(), refund_method()],
    ))]);

    let report = CheckProject::new(&sources, &shop_types())
        .with_selection(Selection::of(vec![Target::method(
            "OrderService",
            "refund",
        )]))
        .execute();

    let judged: Vec<&str> = report.findings().iter().map(Finding::method_name).collect();
    assert_eq!(judged, ["refund"]);
}

#[test]
fn a_selected_method_is_judged_with_the_types_every_source_declares() {
    let path = Path::new("src/OrderService.java");
    let sources = FakeSources(vec![
        Ok(SourceFile::new(
            path,
            vec!["OrderService".to_owned()],
            vec![pay_method()],
        )),
        Ok(SourceFile::new(
            Path::new("src/Customer.java"),
            vec!["Order".to_owned(), "Customer".to_owned()],
            Vec::new(),
        )),
    ]);

    let report = CheckProject::new(&sources, &shop_types())
        .with_selection(Selection::of(vec![Target::class("OrderService")]))
        .execute();

    assert_eq!(
        report.findings(),
        [
            Finding::new(
                path,
                "OrderService",
                "pay",
                Verdict::allowed("getCustomer", 11, Reason::parameter("order"))
            ),
            Finding::new(
                path,
                "OrderService",
                "pay",
                Verdict::violation("getWallet", 11, Reason::return_of("getCustomer"))
            ),
        ]
    );
}

#[test]
fn only_the_selected_files_and_methods_are_counted() {
    let sources = FakeSources(vec![
        Ok(SourceFile::new(
            Path::new("src/OrderService.java"),
            vec!["OrderService".to_owned()],
            vec![pay_method(), refund_method()],
        )),
        Ok(SourceFile::new(
            Path::new("src/Reports.java"),
            vec!["Reports".to_owned()],
            vec![report_method()],
        )),
    ]);

    let counts = CheckProject::new(&sources, &shop_types())
        .with_selection(Selection::of(vec![Target::method("OrderService", "pay")]))
        .execute()
        .counts();

    assert_eq!(
        counts,
        Counts {
            files: 1,
            methods: 1,
            calls: 2,
            allowed: 2,
            ..Counts::default()
        }
    );
}

fn account_sources() -> FakeSources {
    FakeSources(vec![Ok(SourceFile::new(
        Path::new("src/Account.java"),
        vec!["Account".to_owned()],
        Vec::new(),
    ))])
}

#[test]
fn a_selected_hybrid_class_is_reported() {
    let types = FakeTypes::new(vec![account_shape()]);

    let report = CheckProject::new(&account_sources(), &types)
        .with_selection(Selection::of(vec![Target::class("Account")]))
        .execute();

    assert_eq!(report.counts().hybrid_classes, 1);
}

#[test]
fn a_hybrid_class_outside_the_selection_is_not_reported() {
    let types = FakeTypes::new(vec![account_shape()]);

    let report = CheckProject::new(&account_sources(), &types)
        .with_selection(Selection::of(vec![Target::class("OrderService")]))
        .execute();

    assert_eq!(report.hybrid_classes(), []);
}

#[test]
fn selecting_a_method_does_not_report_its_hybrid_class() {
    let types = FakeTypes::new(vec![account_shape()]);

    let report = CheckProject::new(&account_sources(), &types)
        .with_selection(Selection::of(vec![Target::method("Account", "withdraw")]))
        .execute();

    assert_eq!(report.hybrid_classes(), []);
}

fn types_nesting_with(separator: &str) -> FakeTypes {
    FakeTypes {
        platform: Platform::new().with_nested_type_separator(separator),
        ..shop_types()
    }
}

fn total_method(class: &str) -> Method {
    Method::new(class, "total", 30).call(Call::new(
        Expression::parameter("order", TypeRef::class("Order")),
        "getCustomer",
        31,
    ))
}

#[test]
fn the_classes_nested_in_a_selected_class_are_selected_too() {
    let sources = FakeSources(vec![Ok(SourceFile::new(
        Path::new("src/Order.java"),
        vec!["Order".to_owned(), "Order::Line".to_owned()],
        vec![total_method("Order::Line")],
    ))]);

    let report = CheckProject::new(&sources, &types_nesting_with("::"))
        .with_selection(Selection::of(vec![Target::class("Order")]))
        .execute();

    let judged: Vec<&str> = report.findings().iter().map(Finding::class_name).collect();
    assert_eq!(judged, ["Order::Line"]);
}

#[test]
fn selecting_a_method_leaves_out_the_same_named_method_of_a_nested_class() {
    let sources = FakeSources(vec![Ok(SourceFile::new(
        Path::new("src/Order.java"),
        vec!["Order".to_owned(), "Order::Line".to_owned()],
        vec![total_method("Order"), total_method("Order::Line")],
    ))]);

    let report = CheckProject::new(&sources, &types_nesting_with("::"))
        .with_selection(Selection::of(vec![Target::method("Order", "total")]))
        .execute();

    let judged: Vec<&str> = report.findings().iter().map(Finding::class_name).collect();
    assert_eq!(judged, ["Order"]);
}

fn order_service_sources() -> FakeSources {
    FakeSources(vec![Ok(SourceFile::new(
        Path::new("src/OrderService.java"),
        vec!["OrderService".to_owned()],
        vec![pay_method()],
    ))])
}

#[test]
fn a_target_that_names_no_class_is_unmatched() {
    let report = CheckProject::new(&order_service_sources(), &shop_types())
        .with_selection(Selection::of(vec![Target::class("OrderServise")]))
        .execute();

    assert_eq!(report.unmatched_targets(), [Target::class("OrderServise")]);
}

#[test]
fn a_target_that_names_a_method_its_class_lacks_is_unmatched() {
    let report = CheckProject::new(&order_service_sources(), &shop_types())
        .with_selection(Selection::of(vec![Target::method("OrderService", "pai")]))
        .execute();

    assert_eq!(
        report.unmatched_targets(),
        [Target::method("OrderService", "pai")]
    );
}

#[test]
fn targets_that_name_a_declared_class_or_method_are_matched() {
    let report = CheckProject::new(&order_service_sources(), &shop_types())
        .with_selection(Selection::of(vec![
            Target::class("OrderService"),
            Target::method("OrderService", "pay"),
        ]))
        .execute();

    assert_eq!(report.unmatched_targets(), []);
}

#[test]
fn a_selection_of_no_targets_judges_every_method() {
    let sources = FakeSources(vec![
        Ok(SourceFile::new(
            Path::new("src/OrderService.java"),
            vec!["OrderService".to_owned()],
            vec![pay_method()],
        )),
        Ok(SourceFile::new(
            Path::new("src/Reports.java"),
            vec!["Reports".to_owned()],
            vec![report_method()],
        )),
    ]);

    let report = CheckProject::new(&sources, &shop_types())
        .with_selection(Selection::of(Vec::new()))
        .execute();

    let judged: Vec<&str> = report.findings().iter().map(Finding::class_name).collect();
    assert_eq!(judged, ["OrderService", "OrderService", "Reports"]);
}

fn run_method(class: &str) -> Method {
    Method::new(class, "run", 40).call(Call::new(
        Expression::parameter("order", TypeRef::class("Order")),
        "getCustomer",
        41,
    ))
}

#[test]
fn the_methods_of_a_class_declared_in_a_selected_method_are_selected() {
    let sources = FakeSources(vec![Ok(SourceFile::new(
        Path::new("src/Order.java"),
        vec![
            "Order".to_owned(),
            "Order$1".to_owned(),
            "Order$2".to_owned(),
        ],
        vec![
            run_method("Order$1").declared_within("Order", "pay"),
            run_method("Order$2").declared_within("Order", "refund"),
        ],
    ))]);

    let report = CheckProject::new(&sources, &types_nesting_with("$"))
        .with_selection(Selection::of(vec![Target::method("Order", "pay")]))
        .execute();

    let judged: Vec<&str> = report.findings().iter().map(Finding::class_name).collect();
    assert_eq!(judged, ["Order$1"]);
}

#[test]
fn a_selected_method_is_judged_with_the_helpers_its_file_declares() {
    let path = Path::new("src/Shop.java");
    let check_out = Method::new("Shop", "checkOut", 10).call(
        Call::new(Expression::This, "findCart", 11)
            .running_helper_at(20)
            .returning(TypeRef::class("Cart"))
            .then("checkout", 11),
    );
    let find_cart = Method::new("Shop", "findCart", 20)
        .give_back(Expression::field("cart", TypeRef::class("Cart")));
    let sources = FakeSources(vec![Ok(SourceFile::new(
        path,
        vec!["Shop".to_owned(), "Cart".to_owned()],
        vec![check_out, find_cart],
    ))]);
    let types = FakeTypes::new(vec![
        TypeShape::new("Cart", TypeKind::Class).with_method(MethodShape::new("checkout")),
    ]);

    let report = CheckProject::new(&sources, &types)
        .with_selection(Selection::of(vec![Target::method("Shop", "checkOut")]))
        .execute();

    assert_eq!(
        report.findings(),
        [
            Finding::new(
                path,
                "Shop",
                "checkOut",
                Verdict::allowed("findCart", 11, Reason::This)
            ),
            Finding::new(
                path,
                "Shop",
                "checkOut",
                Verdict::allowed("checkout", 11, Reason::field("cart"))
            ),
        ]
    );
}
