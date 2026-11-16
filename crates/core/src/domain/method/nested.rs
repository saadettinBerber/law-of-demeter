use super::{Call, Expression, Lambda, Method, MethodReference, Reason, Verdict};
use crate::domain::classification::{Classification, Types, sample_types};
use crate::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

const STREAM: &str = "java.util.stream.Stream";
const COLLECTOR: &str = "java.util.stream.Collector";
const COLLECTORS: &str = "java.util.stream.Collectors";
const LIST: &str = "java.util.List";
const MAP: &str = "java.util.Map";
const FUNCTION: &str = "java.util.function.Function";
const OBJECT: &str = "java.lang.Object";

fn class(name: &str) -> TypeShape {
    TypeShape::new(name, TypeKind::Class)
}

pub(super) fn interface(name: &str, parameters: &[&str]) -> TypeShape {
    parameters.iter().fold(
        TypeShape::new(name, TypeKind::Interface),
        |shape, parameter| shape.with_type_parameter(parameter),
    )
}

pub(super) fn generic(name: &str, arguments: &[TypeRef]) -> TypeRef {
    TypeRef::generic(name, arguments.to_vec())
}

pub(super) fn variable(name: &str) -> TypeRef {
    TypeRef::variable(name)
}

pub(super) fn order() -> TypeRef {
    TypeRef::class("Order")
}

pub(super) fn customer() -> TypeRef {
    TypeRef::class("Customer")
}

fn status() -> TypeRef {
    TypeRef::class("Status")
}

pub(super) fn wildcard() -> TypeRef {
    TypeRef::class(OBJECT)
}

pub(super) fn stream_of(element: TypeRef) -> TypeRef {
    generic(STREAM, &[element])
}

pub(super) fn list_of(element: TypeRef) -> TypeRef {
    generic(LIST, &[element])
}

pub(super) fn collector(element: TypeRef, accumulation: TypeRef, result: TypeRef) -> TypeRef {
    generic(COLLECTOR, &[element, accumulation, result])
}

fn function(argument: TypeRef, result: TypeRef) -> TypeRef {
    generic(FUNCTION, &[argument, result])
}

pub(super) fn collecting() -> Types {
    let collectors = class(COLLECTORS)
        .with_method(
            MethodShape::new("toList")
                .as_static()
                .with_type_parameter("T")
                .returning(collector(variable("T"), wildcard(), list_of(variable("T")))),
        )
        .with_method(
            MethodShape::new("collectingAndThen")
                .as_static()
                .with_type_parameter("T")
                .with_type_parameter("A")
                .with_type_parameter("R")
                .with_type_parameter("RR")
                .with_parameters(vec![
                    collector(variable("T"), variable("A"), variable("R")),
                    function(variable("R"), variable("RR")),
                ])
                .returning(collector(variable("T"), variable("A"), variable("RR"))),
        )
        .with_method(
            MethodShape::new("groupingBy")
                .as_static()
                .with_type_parameter("T")
                .with_type_parameter("K")
                .with_type_parameter("A")
                .with_type_parameter("D")
                .with_parameters(vec![
                    function(variable("T"), variable("K")),
                    collector(variable("T"), variable("A"), variable("D")),
                ])
                .returning(collector(
                    variable("T"),
                    wildcard(),
                    generic(MAP, &[variable("K"), variable("D")]),
                )),
        )
        .with_method(
            MethodShape::new("groupingBy")
                .as_static()
                .with_type_parameter("T")
                .with_type_parameter("K")
                .with_parameters(vec![function(variable("T"), variable("K"))])
                .returning(collector(
                    variable("T"),
                    wildcard(),
                    generic(MAP, &[variable("K"), list_of(variable("T"))]),
                )),
        );
    sample_types()
        .with_shape(collectors)
        .with_shape(interface(COLLECTOR, &["T", "A", "R"]))
        .with_shape(
            interface(STREAM, &["T"])
                .with_method(
                    MethodShape::new("collect")
                        .with_type_parameter("R")
                        .with_type_parameter("A")
                        .with_parameters(vec![collector(
                            variable("T"),
                            variable("A"),
                            variable("R"),
                        )])
                        .returning(variable("R")),
                )
                .with_method(
                    MethodShape::new("map")
                        .with_type_parameter("R")
                        .with_parameters(vec![function(variable("T"), variable("R"))])
                        .returning(stream_of(variable("R"))),
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
            interface(LIST, &["E"])
                .with_method(MethodShape::new("stream").returning(stream_of(variable("E"))))
                .with_method(
                    MethodShape::new("get")
                        .with_parameters(vec![TypeRef::primitive("int")])
                        .returning(variable("E")),
                ),
        )
        .with_shape(
            interface(MAP, &["K", "V"]).with_method(
                MethodShape::new("get")
                    .with_parameters(vec![wildcard()])
                    .returning(variable("V")),
            ),
        )
        .with_shape(
            class("Order")
                .with_method(MethodShape::new("getCustomer").returning(customer()))
                .with_method(MethodShape::new("getStatus").returning(status()))
                .with_method(MethodShape::new("ship")),
        )
        .with_shape(
            class("Customer")
                .with_method(MethodShape::new("getWallet").returning(TypeRef::class("Wallet"))),
        )
        .with_shape(class("Wallet").with_method(MethodShape::new("withdraw")))
        .with_shape(class("Status"))
}

pub(super) fn orders_stream() -> Call {
    let orders = Expression::parameter("orders", list_of(order()));
    Call::new(orders, "stream", 2).returning(stream_of(order()))
}

pub(super) fn orders_collected_by(collector: Call) -> Call {
    orders_stream()
        .then("collect", 2)
        .with(Expression::return_of(collector))
}

pub(super) fn collectors_call(name: &str) -> Call {
    Call::new(Expression::static_type(TypeRef::class(COLLECTORS)), name, 2)
}

pub(super) fn mapped_customers() -> Call {
    orders_stream()
        .then("map", 2)
        .with(MethodReference::new(Expression::static_type(order()), "getCustomer", 2).taking(1))
        .returning(stream_of(customer()))
        .returning_from(&[0])
}

fn getting_the_customers_of_the_orders() -> Verdict {
    Verdict::allowed(
        "getCustomer",
        2,
        Reason::method_reference(order(), "getCustomer", Reason::parameter("orders")),
    )
}

pub(super) fn stranger_customers() -> Reason {
    Reason::return_of("getCustomer")
}

pub(super) fn collectors_static() -> Reason {
    Reason::static_type(TypeRef::class(COLLECTORS))
}

#[test]
fn a_collector_with_a_resolved_return_types_the_elements_of_the_stream_collecting_it() {
    let charge = Method::new("Billing", "charge", 1).call(
        mapped_customers()
            .then("collect", 2)
            .with(Expression::return_of(
                collectors_call("toList").returning(listing(customer())),
            ))
            .returning(list_of(customer()))
            .then("get", 2)
            .with(Expression::literal())
            .returning(customer())
            .then("getWallet", 3)
            .returning(TypeRef::class("Wallet"))
            .then("withdraw", 3),
    );

    let verdicts = charge.verdicts(&collecting());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 2, Reason::parameter("orders")),
            getting_the_customers_of_the_orders(),
            Verdict::allowed("map", 2, Reason::parameter("orders")),
            Verdict::allowed("toList", 2, collectors_static()),
            Verdict::allowed(
                "collect",
                2,
                Reason::classified(Classification::Container, stream_of(customer()))
            ),
            Verdict::allowed(
                "get",
                2,
                Reason::classified(Classification::Container, list_of(customer()))
            ),
            Verdict::violation("getWallet", 3, stranger_customers()),
            Verdict::violation("withdraw", 3, Reason::return_of("getWallet")),
        ]
    );
}

#[test]
fn collecting_a_stream_into_a_list_keeps_the_elements_typed() {
    let ship = Method::new("Billing", "ship", 1).call(
        orders_collected_by(collectors_call("toList").returning(listing(order())))
            .returning(list_of(order()))
            .then("get", 2)
            .with(Expression::literal())
            .returning(order())
            .then("getCustomer", 3)
            .returning(customer())
            .then("rename", 3),
    );

    let verdicts = ship.verdicts(&collecting());

    assert_eq!(
        verdicts[5],
        Verdict::violation("rename", 3, Reason::return_of("getCustomer"))
    );
}

fn listing(element: TypeRef) -> TypeRef {
    collector(element.clone(), wildcard(), list_of(element))
}

fn orders_by_status() -> TypeRef {
    generic(MAP, &[status(), list_of(order())])
}

fn the_first_order_of_a_status(orders: Call) -> Call {
    orders
        .returning(orders_by_status())
        .then("get", 2)
        .with(Expression::parameter("status", status()))
        .returning(list_of(order()))
        .then("get", 2)
        .with(Expression::literal())
        .returning(order())
}

fn grouping_by_status(downstream: Option<Call>) -> Call {
    let grouping = collectors_call("groupingBy")
        .with(
            MethodReference::new(Expression::static_type(order()), "getStatus", 2)
                .taking(1)
                .fed_by_elements(),
        )
        .returning(collector(order(), wildcard(), orders_by_status()));
    match downstream {
        Some(call) => grouping.with(Expression::return_of(call)),
        None => grouping,
    }
}

#[test]
fn collecting_a_stream_by_a_method_reference_types_the_groups() {
    let ship = Method::new("Billing", "ship", 1).call(
        the_first_order_of_a_status(orders_collected_by(grouping_by_status(None)))
            .then("getCustomer", 3)
            .returning(customer())
            .then("rename", 3),
    );

    let verdicts = ship.verdicts(&collecting());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 2, Reason::parameter("orders")),
            Verdict::allowed(
                "getStatus",
                2,
                Reason::method_reference(order(), "getStatus", Reason::parameter("orders"))
            ),
            Verdict::allowed("groupingBy", 2, collectors_static()),
            Verdict::allowed("collect", 2, Reason::parameter("orders")),
            Verdict::allowed("get", 2, Reason::parameter("orders")),
            Verdict::allowed("get", 2, Reason::parameter("orders")),
            Verdict::allowed("getCustomer", 3, Reason::parameter("orders")),
            Verdict::violation("rename", 3, Reason::return_of("getCustomer")),
        ]
    );
}

#[test]
fn a_collector_wrapping_another_gets_its_result_from_its_finisher() {
    let first = Lambda::new(&["list"])
        .resolving("list", list_of(order()))
        .yielding(Expression::return_of(
            Call::new(Expression::local("list"), "get", 3)
                .with(Expression::literal())
                .returning(order()),
        ));
    let wrapping = collectors_call("collectingAndThen")
        .with(Expression::return_of(
            collectors_call("toList").returning(listing(order())),
        ))
        .with(first)
        .returning(collector(order(), wildcard(), order()))
        .returning_from(&[1]);
    let ship = Method::new("Billing", "ship", 1).call(
        orders_collected_by(wrapping)
            .returning(order())
            .then("getCustomer", 4)
            .returning(customer())
            .then("rename", 4),
    );

    let verdicts = ship.verdicts(&collecting());

    assert_eq!(
        verdicts.last(),
        Some(&Verdict::violation(
            "rename",
            4,
            Reason::return_of("getCustomer")
        ))
    );
}

fn grouping_by_the_result_of(call: Call, elements: TypeRef, key: TypeRef) -> Call {
    let groups = generic(MAP, &[key.clone(), list_of(elements.clone())]);
    collectors_call("groupingBy")
        .with(
            Lambda::new(&["each"])
                .resolving("each", elements.clone())
                .fed_by_elements("each")
                .yielding(Expression::return_of(call.returning(key.clone()))),
        )
        .returning(collector(elements, wildcard(), groups))
}

#[test]
fn a_lambda_in_a_nested_call_takes_the_standing_of_the_stranger_elements_it_receives() {
    let wallet_of_each = Call::new(Expression::local("each"), "getWallet", 3);
    let group =
        Method::new("Billing", "group", 1).call(mapped_customers().then("collect", 2).with(
            Expression::return_of(grouping_by_the_result_of(
                wallet_of_each,
                customer(),
                TypeRef::class("Wallet"),
            )),
        ));

    let verdicts = group.verdicts(&collecting());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 2, Reason::parameter("orders")),
            getting_the_customers_of_the_orders(),
            Verdict::allowed("map", 2, Reason::parameter("orders")),
            Verdict::violation(
                "getWallet",
                3,
                Reason::lambda_parameter("each", stranger_customers())
            ),
            Verdict::allowed("groupingBy", 2, collectors_static()),
            Verdict::allowed(
                "collect",
                2,
                Reason::classified(Classification::Container, stream_of(customer()))
            ),
        ]
    );
}

#[test]
fn a_lambda_in_a_nested_call_takes_the_standing_of_the_friend_elements_it_receives() {
    let status_of_each = Call::new(Expression::local("each"), "getStatus", 3);
    let group = Method::new("Billing", "group", 1).call(orders_collected_by(
        grouping_by_the_result_of(status_of_each, order(), status()),
    ));

    let verdicts = group.verdicts(&collecting());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 2, Reason::parameter("orders")),
            Verdict::allowed(
                "getStatus",
                3,
                Reason::lambda_parameter("each", Reason::parameter("orders"))
            ),
            Verdict::allowed("groupingBy", 2, collectors_static()),
            Verdict::allowed("collect", 2, Reason::parameter("orders")),
        ]
    );
}

#[test]
fn a_lambda_in_a_nested_call_that_receives_something_else_keeps_its_own_feed() {
    let first = Lambda::new(&["list"]).yielding(Expression::return_of(
        Call::new(Expression::local("list"), "get", 3).with(Expression::literal()),
    ));
    let wrapping = collectors_call("collectingAndThen")
        .with(Expression::return_of(collectors_call("toList")))
        .with(first);
    let ship = Method::new("Billing", "ship", 1).call(orders_collected_by(wrapping));

    let verdicts = ship.verdicts(&collecting());

    assert!(verdicts.contains(&Verdict::allowed(
        "get",
        3,
        Reason::lambda_parameter("list", collectors_static())
    )));
}

#[test]
fn collectors_nested_two_levels_deep_type_the_groups() {
    let ship = Method::new("Billing", "ship", 1).call(
        the_first_order_of_a_status(orders_collected_by(grouping_by_status(Some(
            collectors_call("toList").returning(listing(order())),
        ))))
        .then("getCustomer", 3)
        .returning(customer())
        .then("rename", 3),
    );

    let verdicts = ship.verdicts(&collecting());

    assert_eq!(
        verdicts.last(),
        Some(&Verdict::violation(
            "rename",
            3,
            Reason::return_of("getCustomer")
        ))
    );
}
