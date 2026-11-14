use std::path::Path;

use super::{
    CheckProject, Finding, HybridFinding, ReadError, SourceFile, SourceReader, TypeReading,
    TypeSource,
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

struct FakeTypes(Vec<TypeShape>);

impl TypeSource for FakeTypes {
    fn read(&self) -> TypeReading {
        TypeReading {
            shapes: self.0.clone(),
            errors: Vec::new(),
        }
    }
}

fn shop_types() -> FakeTypes {
    FakeTypes(vec![
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
        vec!["OrderService".to_owned()],
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

    let report = CheckProject::new(&sources, &FakeTypes(vec![account_shape()])).execute();

    assert_eq!(report.hybrid_classes(), [HybridFinding::new(path, hybrid)]);
}
