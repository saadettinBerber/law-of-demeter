use super::{Call, Expression, Lambda, Method, MethodReference, Reason, Verdict};
use crate::domain::classification::{Classification, Types};
use crate::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

const STREAM: &str = "java.util.stream.Stream";
const COLLECTOR: &str = "java.util.stream.Collector";
const COLLECTORS: &str = "java.util.stream.Collectors";
const MAP: &str = "java.util.Map";
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

fn stream_of(element: TypeRef) -> TypeRef {
    generic(STREAM, &[element])
}

fn streams() -> Types {
    Types::new()
        .with_shape(
            interface(STREAM, &["T"])
                .with_method(
                    MethodShape::new("map")
                        .with_type_parameter("R")
                        .with_parameters(vec![generic(FUNCTION, &[variable("T"), variable("R")])])
                        .returning(stream_of(variable("R"))),
                )
                .with_method(
                    MethodShape::new("collect")
                        .with_type_parameter("R")
                        .with_type_parameter("A")
                        .with_parameters(vec![generic(
                            COLLECTOR,
                            &[variable("T"), variable("A"), variable("R")],
                        )])
                        .returning(variable("R")),
                ),
        )
        .with_shape(interface(COLLECTOR, &["T", "A", "R"]))
        .with_shape(
            class(COLLECTORS).with_method(
                MethodShape::new("groupingBy")
                    .as_static()
                    .with_type_parameter("T")
                    .with_type_parameter("K")
                    .with_parameters(vec![generic(FUNCTION, &[variable("T"), variable("K")])])
                    .returning(generic(
                        COLLECTOR,
                        &[
                            variable("T"),
                            TypeRef::class("java.lang.Object"),
                            generic(MAP, &[variable("K"), generic(LIST, &[variable("T")])]),
                        ],
                    )),
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
            class("Customer")
                .with_method(MethodShape::new("getWallet").returning(TypeRef::class("Wallet"))),
        )
        .with_shape(class("Wallet").with_method(MethodShape::new("withdraw")))
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

fn mapped_customers() -> Call {
    orders_stream().then("map", 2).with(MethodReference::new(
        Expression::static_type(order()),
        "getCustomer",
        2,
    ))
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

fn the_customers_orders_were_taken_from() -> Reason {
    Reason::method_reference(order(), "getCustomer", Reason::parameter("orders"))
}

#[test]
fn an_unbound_reference_is_judged_as_the_call_on_the_element_that_feeds_it() {
    let charge = Method::new("Billing", "charge", 1).call(mapped_customers());

    let verdicts = charge.verdicts(&streams());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 2, Reason::parameter("orders")),
            Verdict::allowed("getCustomer", 2, the_customers_orders_were_taken_from()),
            Verdict::allowed("map", 2, Reason::parameter("orders")),
        ]
    );
}

#[test]
fn an_unbound_reference_on_elements_a_stranger_returned_is_a_violation() {
    let charge = Method::new("Billing", "charge", 1).call(mapped_customers().then("map", 3).with(
        MethodReference::new(Expression::static_type(customer()), "getWallet", 3),
    ));

    let verdicts = charge.verdicts(&streams());

    assert_eq!(
        verdicts[3],
        Verdict::violation(
            "getWallet",
            3,
            Reason::method_reference(customer(), "getWallet", Reason::return_of("getCustomer"))
        )
    );
}

#[test]
fn an_unbound_reference_nested_in_a_collector_takes_the_standing_of_the_stream_collected() {
    let charge = Method::new("Billing", "charge", 1).call(
        mapped_customers()
            .then("collect", 3)
            .with(Expression::return_of(
                Call::new(
                    Expression::static_type(TypeRef::class(COLLECTORS)),
                    "groupingBy",
                    3,
                )
                .with(MethodReference::new(
                    Expression::static_type(customer()),
                    "getWallet",
                    3,
                )),
            )),
    );

    let verdicts = charge.verdicts(&streams());

    assert_eq!(
        verdicts[3..],
        [
            Verdict::violation(
                "getWallet",
                3,
                Reason::method_reference(customer(), "getWallet", Reason::return_of("getCustomer"))
            ),
            Verdict::allowed(
                "groupingBy",
                3,
                Reason::static_type(TypeRef::class(COLLECTORS))
            ),
            Verdict::allowed(
                "collect",
                3,
                Reason::classified(Classification::Container, stream_of(customer()))
            ),
        ]
    );
}

#[test]
fn an_unbound_reference_that_cannot_be_typed_is_judged_like_an_untyped_lambda_parameter() {
    let charge = Method::new("Billing", "charge", 1).call(
        Call::new(Expression::local("handlers"), "register", 2).with(MethodReference::new(
            Expression::static_type(order()),
            "getCustomer",
            2,
        )),
    );

    let verdicts = charge.verdicts(&streams());

    assert_eq!(
        verdicts,
        [
            Verdict::violation(
                "getCustomer",
                2,
                Reason::unresolved_type(
                    None,
                    Reason::method_reference(
                        order(),
                        "getCustomer",
                        Reason::unassigned_local("handlers")
                    )
                )
            ),
            Verdict::violation(
                "register",
                2,
                Reason::unresolved_type(None, Reason::unassigned_local("handlers"))
            ),
        ]
    );
}

#[test]
fn a_constructor_reference_is_not_a_call() {
    let charge = Method::new("Billing", "charge", 1).call(orders_stream().then("map", 2).with(
        MethodReference::new(Expression::static_type(customer()), "new", 2),
    ));

    let verdicts = charge.verdicts(&streams());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 2, Reason::parameter("orders")),
            Verdict::allowed("map", 2, Reason::parameter("orders")),
        ]
    );
}
