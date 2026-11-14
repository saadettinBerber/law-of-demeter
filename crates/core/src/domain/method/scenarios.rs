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
            Verdict::allowed("truncatedTo", 2, Reason::parameter("issuedAt")),
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

#[test]
fn objects_from_factories_and_new_are_friends() {
    let types =
        Types::new()
            .with_shape(
                class("ReportFactory")
                    .with_method(MethodShape::new("create").returning(TypeRef::class("Report"))),
            )
            .with_shape(class("Report").with_method(
                MethodShape::new("render").returning(TypeRef::class("java.lang.String")),
            ));
    let report = Method::new("Reports", "report", 1).call(
        Call::new(
            Expression::parameter("factory", TypeRef::class("ReportFactory")),
            "create",
            2,
        )
        .then("render", 2),
    );

    let verdicts = report.verdicts(&types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("create", 2, Reason::parameter("factory")),
            Verdict::allowed("render", 2, Reason::factory_result("create")),
        ]
    );
}

#[test]
fn talking_to_the_object_a_repository_query_returned_is_allowed() {
    let types = Types::new()
        .with_shape(
            class("OrderRepository").with_method(
                MethodShape::new("findById")
                    .with_parameters(vec![TypeRef::class("OrderId")])
                    .returning(TypeRef::class("Order")),
            ),
        )
        .with_shape(class("Order").with_method(MethodShape::new("ship")));
    let dispatch = Method::new("Dispatcher", "dispatch", 1).call(
        Call::new(
            Expression::field("orders", TypeRef::class("OrderRepository")),
            "findById",
            2,
        )
        .with(Expression::parameter("id", TypeRef::class("OrderId")))
        .then("ship", 2),
    );

    let verdicts = dispatch.verdicts(&types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("findById", 2, Reason::field("orders")),
            Verdict::allowed("ship", 2, Reason::query_result("findById")),
        ]
    );
}

#[test]
fn builder_chains_stay_with_the_builder() {
    let order = TypeRef::class("Order");
    let builder = TypeRef::class("Order$Builder");
    let types = Types::new()
        .with_shape(
            class("Order").with_method(
                MethodShape::new("builder")
                    .as_static()
                    .returning(builder.clone()),
            ),
        )
        .with_shape(
            class("Order$Builder")
                .with_method(
                    MethodShape::new("id")
                        .with_parameters(vec![TypeRef::class("java.lang.String")])
                        .returning(builder.clone()),
                )
                .with_method(
                    MethodShape::new("total")
                        .with_parameters(vec![TypeRef::primitive("int")])
                        .returning(builder.clone()),
                )
                .with_method(MethodShape::new("build").returning(order.clone())),
        );
    let create = Method::new("Orders", "create", 1).call(
        Call::new(Expression::static_type(order.clone()), "builder", 2)
            .then("id", 2)
            .with(Expression::parameter(
                "id",
                TypeRef::class("java.lang.String"),
            ))
            .then("total", 2)
            .with(Expression::parameter("total", TypeRef::primitive("int")))
            .then("build", 2),
    );
    let constructor = Method::new("Order", "<init>", 10);

    let verdicts = [create.verdicts(&types), constructor.verdicts(&types)];

    assert_eq!(
        verdicts,
        [
            vec![
                Verdict::allowed("builder", 2, Reason::static_type(order)),
                Verdict::allowed("id", 2, Reason::static_result("builder")),
                Verdict::allowed("total", 2, Reason::static_result("builder")),
                Verdict::allowed("build", 2, Reason::static_result("builder")),
            ],
            vec![],
        ]
    );
}
