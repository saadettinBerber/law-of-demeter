use super::{Call, Expression, Lambda, Method, MethodReference, Reason, Verdict};
use crate::domain::classification::Types;
use crate::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

const STREAM: &str = "java.util.stream.Stream";
const LIST: &str = "java.util.List";
const FUNCTION: &str = "java.util.function.Function";

fn class(name: &str) -> TypeShape {
    TypeShape::new(name, TypeKind::Class)
}

fn interface(name: &str, parameters: &[&str]) -> TypeShape {
    parameters.iter().fold(
        TypeShape::new(name, TypeKind::Interface),
        |shape, parameter| shape.with_type_parameter(parameter),
    )
}

fn generic(name: &str, arguments: &[TypeRef]) -> TypeRef {
    TypeRef::generic(name, arguments.to_vec())
}

fn variable(name: &str) -> TypeRef {
    TypeRef::variable(name)
}

fn order() -> TypeRef {
    TypeRef::class("Order")
}

fn customer() -> TypeRef {
    TypeRef::class("Customer")
}

fn wallets() -> TypeRef {
    TypeRef::class("Wallets")
}

fn streams() -> Types {
    Types::new()
        .with_shape(
            interface(STREAM, &["T"]).with_method(
                MethodShape::new("map")
                    .with_type_parameter("R")
                    .with_parameters(vec![generic(FUNCTION, &[variable("T"), variable("R")])])
                    .returning(generic(STREAM, &[variable("R")])),
            ),
        )
        .with_shape(
            interface(FUNCTION, &["T", "R"]).with_method(
                MethodShape::new("apply")
                    .with_parameters(vec![variable("T")])
                    .returning(variable("R"))
                    .as_abstract(),
            ),
        )
        .with_shape(
            interface(LIST, &["E"]).with_method(
                MethodShape::new("stream").returning(generic(STREAM, &[variable("E")])),
            ),
        )
        .with_shape(
            class("Order").with_method(MethodShape::new("getCustomer").returning(customer())),
        )
        .with_shape(
            class("Customers").with_method(
                MethodShape::new("of")
                    .as_static()
                    .with_parameters(vec![order()])
                    .returning(customer()),
            ),
        )
        .with_shape(class("PremiumCustomers").with_superclass(TypeRef::class("Customers")))
        .with_shape(
            class("Wallets").with_method(
                MethodShape::new("of")
                    .as_static()
                    .with_parameters(vec![customer()])
                    .returning(TypeRef::class("Wallet")),
            ),
        )
}

fn orders_stream() -> Call {
    let orders = Expression::parameter("orders", generic(LIST, &[order()]));
    Call::new(orders, "stream", 2)
}

#[test]
fn a_reference_to_a_static_method_is_allowed_as_the_static_call_it_stands_for() {
    let charge = Method::new("Billing", "charge", 1).call(orders_stream().then("map", 2).with(
        MethodReference::new(
            Expression::static_type(TypeRef::class("Customers")),
            "of",
            3,
        ),
    ));

    let verdicts = charge.verdicts(&streams());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 2, Reason::parameter("orders")),
            Verdict::allowed("of", 3, Reason::static_type(TypeRef::class("Customers"))),
            Verdict::allowed("map", 2, Reason::parameter("orders")),
        ]
    );
}

#[test]
fn a_reference_to_a_static_method_is_allowed_whoever_feeds_it() {
    let order_customers = Lambda::new(&["order"]).yielding(Expression::return_of(Call::new(
        Expression::local("order"),
        "getCustomer",
        3,
    )));
    let charge = Method::new("Billing", "charge", 1).call(
        orders_stream()
            .then("map", 2)
            .with(order_customers)
            .then("map", 4)
            .with(MethodReference::new(
                Expression::static_type(wallets()),
                "of",
                4,
            )),
    );

    let verdicts = charge.verdicts(&streams());

    assert_eq!(
        verdicts[3],
        Verdict::allowed("of", 4, Reason::static_type(wallets()))
    );
}

#[test]
fn a_reference_through_a_subtype_to_an_inherited_static_method_is_static() {
    let charge = Method::new("Billing", "charge", 1).call(orders_stream().then("map", 2).with(
        MethodReference::new(
            Expression::static_type(TypeRef::class("PremiumCustomers")),
            "of",
            3,
        ),
    ));

    let verdicts = charge.verdicts(&streams());

    assert_eq!(
        verdicts[1],
        Verdict::allowed(
            "of",
            3,
            Reason::static_type(TypeRef::class("PremiumCustomers"))
        )
    );
}
