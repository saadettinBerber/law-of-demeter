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
