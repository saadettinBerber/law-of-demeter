use super::{Call, Creation, Expression, Method, Reason, Verdict};
use crate::domain::classification::sample_types;
use crate::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

#[test]
fn f_may_call_methods_of_c_itself() {
    let f = Method::new("Order", "ship", 1).call(Call::new(Expression::This, "audit", 2));

    let verdicts = f.verdicts(&sample_types());

    assert_eq!(verdicts, [Verdict::allowed("audit", 2, Reason::This)]);
}

#[test]
fn f_may_call_methods_of_an_object_created_in_f() {
    let f = Method::new("Order", "ship", 1).call(Call::new(
        Expression::created(Creation::new(TypeRef::class("StringBuilder"))),
        "append",
        2,
    ));

    let verdicts = f.verdicts(&sample_types());

    assert_eq!(
        verdicts,
        [Verdict::allowed(
            "append",
            2,
            Reason::created(TypeRef::class("StringBuilder"))
        )]
    );
}

#[test]
fn f_may_call_methods_of_an_object_given_to_f_as_an_argument() {
    let f = Method::new("Shipping", "canShip", 1).call(Call::new(
        Expression::parameter("order", TypeRef::class("Order")),
        "getStatus",
        2,
    ));

    let verdicts = f.verdicts(&sample_types());

    assert_eq!(
        verdicts,
        [Verdict::allowed("getStatus", 2, Reason::parameter("order"))]
    );
}

#[test]
fn f_may_call_methods_of_an_object_held_in_a_field_of_c() {
    let f = Method::new("PaymentService", "pay", 1).call(Call::new(
        Expression::field("wallet", TypeRef::class("Wallet")),
        "withdraw",
        2,
    ));

    let verdicts = f.verdicts(&sample_types());

    assert_eq!(
        verdicts,
        [Verdict::allowed("withdraw", 2, Reason::field("wallet"))]
    );
}

#[test]
fn f_may_not_call_methods_of_what_a_friend_returns() {
    let types = sample_types()
        .with_shape(
            TypeShape::new("Order", TypeKind::Class).with_method(
                MethodShape::new("getCustomer")
                    .reading("customer")
                    .returning(TypeRef::class("Customer")),
            ),
        )
        .with_shape(
            TypeShape::new("Customer", TypeKind::Class)
                .with_method(
                    MethodShape::new("getWallet")
                        .reading("wallet")
                        .returning(TypeRef::class("Wallet")),
                )
                .with_method(MethodShape::new("rename")),
        )
        .with_shape(
            TypeShape::new("Wallet", TypeKind::Class).with_method(MethodShape::new("withdraw")),
        );
    let f = Method::new("OrderService", "pay", 1).call(
        Call::new(
            Expression::parameter("order", TypeRef::class("Order")),
            "getCustomer",
            2,
        )
        .returning(TypeRef::class("Customer"))
        .then("getWallet", 2)
        .returning(TypeRef::class("Wallet"))
        .then("withdraw", 2),
    );

    let verdicts = f.verdicts(&types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 2, Reason::parameter("order")),
            Verdict::violation("getWallet", 2, Reason::return_of("getCustomer")),
            Verdict::violation("withdraw", 2, Reason::return_of("getWallet")),
        ]
    );
}
