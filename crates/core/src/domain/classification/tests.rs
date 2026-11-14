use super::{Classification, Types};
use crate::domain::type_shape::{FieldShape, MethodShape, TypeKind, TypeRef, TypeShape};

#[test]
fn a_record_is_a_data_structure() {
    let types = Types::new().with_shape(TypeShape::new("Server", TypeKind::Record));

    let classification = types.classify(&TypeRef::class("Server"));

    assert_eq!(classification, Classification::DataStructure);
}

fn string() -> TypeRef {
    TypeRef::class("java.lang.String")
}

fn classify(shape: TypeShape) -> Classification {
    let name = shape.name().to_owned();
    Types::new()
        .with_shape(shape)
        .classify(&TypeRef::class(&name))
}

#[test]
fn a_class_with_only_getters_and_setters_is_a_data_structure() {
    let address = TypeShape::new("Address", TypeKind::Class)
        .with_method(
            MethodShape::new("getCity")
                .returning(string())
                .reading("city"),
        )
        .with_method(
            MethodShape::new("setCity")
                .with_parameters(vec![string()])
                .writing("city"),
        );

    let classification = classify(address);

    assert_eq!(classification, Classification::DataStructure);
}

fn int() -> TypeRef {
    TypeRef::primitive("int")
}

#[test]
fn a_class_with_behavior_and_no_setters_is_an_object() {
    let order = TypeShape::new("Order", TypeKind::Class)
        .with_method(MethodShape::new("getId").returning(string()).reading("id"))
        .with_method(MethodShape::new("charge").with_parameters(vec![int()]));

    let classification = classify(order);

    assert_eq!(classification, Classification::Object);
}

#[test]
fn a_getter_that_computes_instead_of_reading_a_field_is_behavior() {
    let invoice = TypeShape::new("Invoice", TypeKind::Class)
        .with_field(FieldShape::new("total", int()))
        .with_method(MethodShape::new("getTotal").returning(int()));

    let classification = classify(invoice);

    assert_eq!(classification, Classification::Object);
}

#[test]
fn a_class_with_behavior_and_a_setter_is_a_hybrid() {
    let account = TypeShape::new("Account", TypeKind::Class)
        .with_method(
            MethodShape::new("getBalance")
                .returning(int())
                .reading("balance"),
        )
        .with_method(
            MethodShape::new("setBalance")
                .with_parameters(vec![int()])
                .writing("balance"),
        )
        .with_method(MethodShape::new("withdraw").with_parameters(vec![int()]));

    let classification = classify(account);

    assert_eq!(classification, Classification::Hybrid);
}

#[test]
fn generated_and_hand_written_accessors_are_classified_the_same() {
    let generated = TypeShape::new("Account", TypeKind::Class)
        .with_method(
            MethodShape::new("getBalance")
                .returning(int())
                .reading("balance")
                .generated(),
        )
        .with_method(
            MethodShape::new("setBalance")
                .with_parameters(vec![int()])
                .writing("balance")
                .generated(),
        )
        .with_method(MethodShape::new("withdraw").with_parameters(vec![int()]));
    let hand_written = TypeShape::new("Account", TypeKind::Class)
        .with_method(
            MethodShape::new("getBalance")
                .returning(int())
                .reading("balance"),
        )
        .with_method(
            MethodShape::new("setBalance")
                .with_parameters(vec![int()])
                .writing("balance"),
        )
        .with_method(MethodShape::new("withdraw").with_parameters(vec![int()]));

    let classifications = (classify(generated), classify(hand_written));

    assert_eq!(
        classifications,
        (Classification::Hybrid, Classification::Hybrid)
    );
}

#[test]
fn constructors_object_methods_statics_and_non_public_methods_are_not_behavior() {
    let dto = TypeShape::new("Customer", TypeKind::Class)
        .with_method(
            MethodShape::new("getName")
                .returning(string())
                .reading("name"),
        )
        .with_method(MethodShape::new("<init>"))
        .with_method(MethodShape::new("toString").returning(string()).generated())
        .with_method(MethodShape::new("hashCode").returning(int()).generated())
        .with_method(
            MethodShape::new("equals")
                .with_parameters(vec![TypeRef::class("java.lang.Object")])
                .generated(),
        )
        .with_method(
            MethodShape::new("builder")
                .returning(TypeRef::class("Customer$Builder"))
                .as_static(),
        )
        .with_method(MethodShape::new("validate").not_public());

    let classification = classify(dto);

    assert_eq!(classification, Classification::DataStructure);
}

#[test]
fn an_overload_of_an_object_method_with_other_parameters_is_behavior() {
    let customer = TypeShape::new("Customer", TypeKind::Class)
        .with_method(MethodShape::new("toString").with_parameters(vec![int()]));

    let classification = classify(customer);

    assert_eq!(classification, Classification::Object);
}

#[test]
fn an_enum_is_a_value() {
    let status = TypeShape::new("Status", TypeKind::Enum)
        .with_method(MethodShape::new("isFinal").returning(TypeRef::primitive("boolean")));

    let classification = classify(status);

    assert_eq!(classification, Classification::Value);
}

#[test]
fn an_interface_is_an_object() {
    let wallet = TypeShape::new("Wallet", TypeKind::Interface)
        .with_method(MethodShape::new("getBalance").returning(int()));

    let classification = classify(wallet);

    assert_eq!(classification, Classification::Object);
}

#[test]
fn a_data_structure_and_the_object_it_carries_are_told_apart() {
    let types = Types::new()
        .with_shape(
            TypeShape::new("OrderShippedEvent", TypeKind::Class)
                .with_field(FieldShape::new("order", TypeRef::class("Order")))
                .with_method(
                    MethodShape::new("getOrder")
                        .returning(TypeRef::class("Order"))
                        .reading("order"),
                ),
        )
        .with_shape(
            TypeShape::new("Order", TypeKind::Class)
                .with_method(MethodShape::new("getTotal").returning(int())),
        );

    let event = types.classify(&TypeRef::class("OrderShippedEvent"));
    let carried = types.classify(&TypeRef::class("Order"));

    assert_eq!(
        (event, carried),
        (Classification::DataStructure, Classification::Object)
    );
}

fn classify_names(types: &Types, names: &[&str]) -> Vec<Classification> {
    names
        .iter()
        .map(|name| types.classify(&TypeRef::class(name)))
        .collect()
}

#[test]
fn well_known_jdk_values_are_values_without_a_shape() {
    let names = [
        "java.lang.String",
        "java.lang.StringBuilder",
        "java.lang.Integer",
        "java.lang.Boolean",
        "java.lang.Character",
        "java.lang.Number",
        "java.math.BigDecimal",
        "java.time.Instant",
        "java.time.Duration",
        "java.time.temporal.ChronoUnit",
        "java.util.UUID",
        "java.util.Date",
    ];

    let classifications = classify_names(&Types::new(), &names);

    assert_eq!(classifications, vec![Classification::Value; names.len()]);
}

#[test]
fn primitives_are_values() {
    let classification = Types::new().classify(&int());

    assert_eq!(classification, Classification::Value);
}

#[test]
fn jdk_collections_maps_optionals_and_streams_are_containers_without_a_shape() {
    let names = [
        "java.util.List",
        "java.util.ArrayList",
        "java.util.Set",
        "java.util.Map",
        "java.util.HashMap",
        "java.util.Collection",
        "java.util.Optional",
        "java.util.stream.Stream",
        "java.util.stream.IntStream",
        "java.util.concurrent.ConcurrentHashMap",
        "java.lang.Iterable",
    ];

    let classifications = classify_names(&Types::new(), &names);

    assert_eq!(
        classifications,
        vec![Classification::Container; names.len()]
    );
}

#[test]
fn an_array_is_a_container() {
    let classification = Types::new().classify(&TypeRef::array(TypeRef::class("Order")));

    assert_eq!(classification, Classification::Container);
}

#[test]
fn a_subtype_of_number_is_a_value() {
    let types = Types::new()
        .with_shape(
            TypeShape::new("Money", TypeKind::Class)
                .extending(TypeRef::class("java.lang.Number"))
                .with_method(
                    MethodShape::new("add").with_parameters(vec![TypeRef::class("Money")]),
                ),
        )
        .with_shape(TypeShape::new("Euro", TypeKind::Class).extending(TypeRef::class("Money")));

    let classifications = classify_names(&types, &["Money", "Euro"]);

    assert_eq!(
        classifications,
        [Classification::Value, Classification::Value]
    );
}

#[test]
fn a_subtype_of_a_collection_is_a_container() {
    let order_list = TypeShape::new("OrderList", TypeKind::Class).extending(TypeRef::generic(
        "java.util.ArrayList",
        vec![TypeRef::class("Order")],
    ));
    let order_source = TypeShape::new("OrderSource", TypeKind::Interface).extending(
        TypeRef::generic("java.util.List", vec![TypeRef::class("Order")]),
    );
    let types = Types::new().with_shape(order_list).with_shape(order_source);

    let classifications = classify_names(&types, &["OrderList", "OrderSource"]);

    assert_eq!(classifications, [Classification::Container; 2]);
}

#[test]
fn a_type_with_no_shape_is_unknown() {
    let types = Types::new();

    let classification = types.classify(&TypeRef::class("com.example.Mystery"));

    assert_eq!(classification, Classification::Unknown);
}

#[test]
fn a_type_variable_is_unknown() {
    let classification = Types::new().classify(&TypeRef::variable("T"));

    assert_eq!(classification, Classification::Unknown);
}

#[test]
fn a_supertype_that_is_neither_value_nor_container_does_not_decide_the_classification() {
    let types = Types::new()
        .with_shape(TypeShape::new("Base", TypeKind::Class).with_method(MethodShape::new("charge")))
        .with_shape(
            TypeShape::new("Address", TypeKind::Class)
                .extending(TypeRef::class("Base"))
                .extending(TypeRef::class("java.lang.Comparable"))
                .with_method(
                    MethodShape::new("getCity")
                        .returning(string())
                        .reading("city"),
                ),
        );

    let classification = types.classify(&TypeRef::class("Address"));

    assert_eq!(classification, Classification::DataStructure);
}

#[test]
fn the_catalog_returns_the_shape_registered_under_a_name() {
    let types = Types::new().with_shape(TypeShape::new("Order", TypeKind::Class));

    let shape = types.shape("Order").map(TypeShape::name);

    assert_eq!(shape, Some("Order"));
}

#[test]
fn the_catalog_has_no_shape_for_an_unregistered_name() {
    let types = Types::new();

    let shape = types.shape("Order");

    assert_eq!(shape, None);
}

fn order_with_customer_lookups() -> Types {
    Types::new().with_shape(
        TypeShape::new("Order", TypeKind::Class)
            .with_method(MethodShape::new("getCustomer").returning(TypeRef::class("Customer")))
            .with_method(
                MethodShape::new("line")
                    .with_parameters(vec![int()])
                    .returning(TypeRef::class("OrderLine")),
            )
            .with_method(
                MethodShape::new("line")
                    .with_parameters(vec![int(), int()])
                    .returning(TypeRef::class("OrderLineGroup")),
            ),
    )
}

#[test]
fn a_return_type_is_found_by_method_name() {
    let types = order_with_customer_lookups();

    let returned = types.return_type("Order", "getCustomer", 0);

    assert_eq!(returned, Some(&TypeRef::class("Customer")));
}

#[test]
fn overloads_are_told_apart_by_parameter_count() {
    let types = order_with_customer_lookups();

    let returned = (
        types.return_type("Order", "line", 1),
        types.return_type("Order", "line", 2),
    );

    assert_eq!(
        returned,
        (
            Some(&TypeRef::class("OrderLine")),
            Some(&TypeRef::class("OrderLineGroup"))
        )
    );
}

#[test]
fn overloads_with_the_same_parameter_count_and_different_returns_are_ambiguous() {
    let types = Types::new().with_shape(
        TypeShape::new("Order", TypeKind::Class)
            .with_method(
                MethodShape::new("find")
                    .with_parameters(vec![int()])
                    .returning(TypeRef::class("OrderLine")),
            )
            .with_method(
                MethodShape::new("find")
                    .with_parameters(vec![string()])
                    .returning(TypeRef::class("Customer")),
            ),
    );

    let returned = types.return_type("Order", "find", 1);

    assert_eq!(returned, None);
}

#[test]
fn a_return_type_is_unknown_for_a_missing_type_or_method() {
    let types = order_with_customer_lookups();

    let returned = (
        types.return_type("Mystery", "getCustomer", 0),
        types.return_type("Order", "getMystery", 0),
        types.return_type("Order", "getCustomer", 1),
    );

    assert_eq!(returned, (None, None, None));
}

#[test]
fn a_return_type_is_found_on_a_supertype() {
    let types = Types::new()
        .with_shape(
            TypeShape::new("BaseEntity", TypeKind::Class)
                .with_method(MethodShape::new("getId").returning(TypeRef::class("EntityId"))),
        )
        .with_shape(
            TypeShape::new("Order", TypeKind::Class).extending(TypeRef::class("BaseEntity")),
        );

    let returned = types.return_type("Order", "getId", 0);

    assert_eq!(returned, Some(&TypeRef::class("EntityId")));
}

#[test]
fn a_return_type_declared_on_the_type_wins_over_the_supertype() {
    let types = Types::new()
        .with_shape(
            TypeShape::new("BaseEntity", TypeKind::Class)
                .with_method(MethodShape::new("copy").returning(TypeRef::class("BaseEntity"))),
        )
        .with_shape(
            TypeShape::new("Order", TypeKind::Class)
                .extending(TypeRef::class("BaseEntity"))
                .with_method(MethodShape::new("copy").returning(TypeRef::class("Order"))),
        );

    let returned = types.return_type("Order", "copy", 0);

    assert_eq!(returned, Some(&TypeRef::class("Order")));
}
