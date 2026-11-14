use super::{Call, Expression, Method, Reason, Verdict};
use crate::domain::classification::{Classification, Types};
use crate::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

fn record(name: &str) -> TypeShape {
    TypeShape::new(name, TypeKind::Record)
}

fn class(name: &str) -> TypeShape {
    TypeShape::new(name, TypeKind::Class)
}

fn getter(name: &str, field: &str, returns: &str) -> MethodShape {
    MethodShape::new(name)
        .reading(field)
        .returning(TypeRef::class(returns))
}

#[test]
fn reading_a_chain_through_data_structures_is_allowed() {
    let types = Types::new()
        .with_shape(record("Config").with_method(getter("server", "server", "Server")))
        .with_shape(
            record("Server").with_method(
                MethodShape::new("port")
                    .reading("port")
                    .returning(TypeRef::primitive("int")),
            ),
        )
        .with_shape(class("ShippingRequest").with_method(getter(
            "getAddress",
            "address",
            "Address",
        )))
        .with_shape(class("Address").with_method(getter("getCity", "city", "java.lang.String")))
        .with_shape(class("Customer").with_method(getter("getId", "id", "CustomerId")))
        .with_shape(class("CustomerId").with_method(getter(
            "getValue",
            "value",
            "java.lang.String",
        )));
    let port_of = Method::new("Connector", "portOf", 1).call(
        Call::new(
            Expression::parameter("config", TypeRef::class("Config")),
            "server",
            2,
        )
        .then("port", 2),
    );
    let city_of = Method::new("Shipping", "cityOf", 1).call(
        Call::new(
            Expression::parameter("request", TypeRef::class("ShippingRequest")),
            "getAddress",
            2,
        )
        .then("getCity", 2),
    );
    let key_of = Method::new("Directory", "keyOf", 1).call(
        Call::new(
            Expression::parameter("customer", TypeRef::class("Customer")),
            "getId",
            2,
        )
        .then("getValue", 2),
    );

    let verdicts = [
        port_of.verdicts(&types),
        city_of.verdicts(&types),
        key_of.verdicts(&types),
    ];

    assert_eq!(
        verdicts,
        [
            vec![
                Verdict::allowed("server", 2, Reason::parameter("config")),
                Verdict::allowed(
                    "port",
                    2,
                    Reason::classified(Classification::DataStructure, TypeRef::class("Server"))
                ),
            ],
            vec![
                Verdict::allowed("getAddress", 2, Reason::parameter("request")),
                Verdict::allowed(
                    "getCity",
                    2,
                    Reason::classified(Classification::DataStructure, TypeRef::class("Address"))
                ),
            ],
            vec![
                Verdict::allowed("getId", 2, Reason::parameter("customer")),
                Verdict::allowed(
                    "getValue",
                    2,
                    Reason::classified(Classification::DataStructure, TypeRef::class("CustomerId"))
                ),
            ],
        ]
    );
}

#[test]
fn operating_on_a_value_is_allowed() {
    let instant = TypeRef::class("java.time.Instant");
    let types = Types::new().with_shape(
        class("java.time.Instant").with_method(
            MethodShape::new("plus")
                .with_parameters(vec![TypeRef::class("java.time.Duration")])
                .returning(instant.clone()),
        ),
    );
    let expiry = Method::new("Tokens", "expiry", 1).call(
        Call::new(
            Expression::parameter("issuedAt", instant.clone()),
            "plus",
            2,
        )
        .with(Expression::parameter(
            "ttl",
            TypeRef::class("java.time.Duration"),
        ))
        .then("truncatedTo", 2)
        .with(Expression::literal()),
    );

    let verdicts = expiry.verdicts(&types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("plus", 2, Reason::parameter("issuedAt")),
            Verdict::allowed(
                "truncatedTo",
                2,
                Reason::classified(Classification::Value, instant)
            ),
        ]
    );
}

#[test]
fn transforming_a_collection_with_a_stream_is_allowed() {
    let order = TypeRef::class("Order");
    let orders = TypeRef::generic("java.util.List", vec![order.clone()]);
    let stream = TypeRef::generic("java.util.stream.Stream", vec![order]);
    let types = Types::new()
        .with_shape(
            TypeShape::new("java.util.List", TypeKind::Interface)
                .with_method(MethodShape::new("stream").returning(stream.clone())),
        )
        .with_shape(
            TypeShape::new("java.util.stream.Stream", TypeKind::Interface)
                .with_method(
                    MethodShape::new("filter")
                        .with_parameters(vec![TypeRef::class("java.util.function.Predicate")])
                        .returning(stream.clone()),
                )
                .with_method(MethodShape::new("count").returning(TypeRef::primitive("long"))),
        );
    let paid_count = Method::new("Reports", "paidCount", 1).call(
        Call::new(Expression::parameter("orders", orders), "stream", 2)
            .then("filter", 2)
            .with(Expression::literal())
            .then("count", 2),
    );

    let verdicts = paid_count.verdicts(&types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 2, Reason::parameter("orders")),
            Verdict::allowed(
                "filter",
                2,
                Reason::classified(Classification::Container, stream.clone())
            ),
            Verdict::allowed(
                "count",
                2,
                Reason::classified(Classification::Container, stream)
            ),
        ]
    );
}

#[test]
fn changing_a_collection_returned_by_a_getter_is_a_violation() {
    let lines = TypeRef::generic("java.util.List", vec![TypeRef::class("OrderLine")]);
    let types = Types::new().with_shape(
        class("Order").with_method(
            MethodShape::new("getLines")
                .reading("lines")
                .returning(lines.clone()),
        ),
    );
    let add_line = Method::new("OrderService", "addLine", 1).call(
        Call::new(
            Expression::parameter("order", TypeRef::class("Order")),
            "getLines",
            2,
        )
        .then("add", 2)
        .with(Expression::parameter("line", TypeRef::class("OrderLine"))),
    );

    let verdicts = add_line.verdicts(&types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getLines", 2, Reason::parameter("order")),
            Verdict::violation(
                "add",
                2,
                Reason::container_mutation(lines, Reason::return_of("getLines"))
            ),
        ]
    );
}
