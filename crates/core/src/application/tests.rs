use std::cell::RefCell;
use std::path::Path;

use super::{
    CheckProject, Counts, Finding, HybridFinding, ReadError, Report, SourceFile, SourceReader,
    TypeReading, TypeSource,
};
use crate::domain::classification::Types;
use crate::domain::method::{Call, Expression, Method, Reason, Verdict};
use crate::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

struct FakeSources(Vec<Result<SourceFile, ReadError>>);

impl SourceReader for FakeSources {
    fn read(&self, _type_exists: &dyn Fn(&str) -> bool) -> Vec<Result<SourceFile, ReadError>> {
        self.0.clone()
    }
}

struct FakeTypes {
    shapes: Vec<TypeShape>,
    errors: Vec<ReadError>,
}

impl FakeTypes {
    fn new(shapes: Vec<TypeShape>) -> Self {
        Self {
            shapes,
            errors: Vec::new(),
        }
    }
}

impl TypeSource for FakeTypes {
    fn read(&self) -> TypeReading {
        TypeReading {
            shapes: self.shapes.clone(),
            errors: self.errors.clone(),
        }
    }
}

struct AskingSources(RefCell<Vec<bool>>);

impl SourceReader for AskingSources {
    fn read(&self, type_exists: &dyn Fn(&str) -> bool) -> Vec<Result<SourceFile, ReadError>> {
        self.0
            .borrow_mut()
            .extend([type_exists("Order"), type_exists("Missing")]);
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
