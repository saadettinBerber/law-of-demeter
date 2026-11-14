use super::{
    Classification, HybridClass, Types, element_type, is_factory_method, is_repository_query,
};
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

fn order_builder() -> Types {
    Types::new().with_shape(
        TypeShape::new("Order$Builder", TypeKind::Class)
            .with_method(
                MethodShape::new("id")
                    .with_parameters(vec![string()])
                    .returning(TypeRef::class("Order$Builder")),
            )
            .with_method(MethodShape::new("build").returning(TypeRef::class("Order")))
            .with_method(
                MethodShape::new("of")
                    .returning(TypeRef::class("Order$Builder"))
                    .as_static(),
            ),
    )
}

#[test]
fn a_method_returning_its_receivers_type_is_fluent() {
    let types = order_builder();

    let fluent = types.is_fluent("Order$Builder", "id", 1);

    assert!(fluent);
}

#[test]
fn a_method_returning_another_type_is_not_fluent() {
    let types = order_builder();

    let fluent = types.is_fluent("Order$Builder", "build", 0);

    assert!(!fluent);
}

#[test]
fn a_static_method_returning_its_own_type_is_not_fluent() {
    let types = order_builder();

    let fluent = types.is_fluent("Order$Builder", "of", 0);

    assert!(!fluent);
}

#[test]
fn a_method_that_cannot_be_found_is_not_fluent() {
    let types = order_builder();

    let fluent = (
        types.is_fluent("Order$Builder", "id", 2),
        types.is_fluent("Mystery", "id", 1),
    );

    assert_eq!(fluent, (false, false));
}

#[test]
fn the_element_of_a_collection_stream_or_optional_is_its_type_argument() {
    let order = TypeRef::class("Order");
    let containers = [
        TypeRef::generic("java.util.List", vec![order.clone()]),
        TypeRef::generic("java.util.stream.Stream", vec![order.clone()]),
        TypeRef::generic("java.util.Optional", vec![order.clone()]),
        TypeRef::array(order.clone()),
    ];

    let elements: Vec<_> = containers.iter().map(element_type).collect();

    assert_eq!(elements, vec![Some(&order); containers.len()]);
}

#[test]
fn the_element_of_a_map_is_its_value_type() {
    let map = TypeRef::generic("java.util.Map", vec![string(), TypeRef::class("Order")]);

    let element = element_type(&map);

    assert_eq!(element, Some(&TypeRef::class("Order")));
}

#[test]
fn a_container_without_type_arguments_has_no_known_element() {
    let raw = TypeRef::class("java.util.List");

    let element = element_type(&raw);

    assert_eq!(element, None);
}

#[test]
fn a_generic_type_that_is_not_a_container_has_no_element() {
    let page = TypeRef::generic("Page", vec![TypeRef::class("Order")]);

    let element = element_type(&page);

    assert_eq!(element, None);
}

#[test]
fn names_that_create_a_new_object_are_factory_methods() {
    let names = [
        "create",
        "createOrder",
        "of",
        "ofNullable",
        "from",
        "fromJson",
        "build",
        "newInstance",
    ];

    let factories: Vec<_> = names.iter().map(|name| is_factory_method(name)).collect();

    assert_eq!(factories, vec![true; names.len()]);
}

#[test]
fn names_that_only_start_with_a_factory_word_are_not_factory_methods() {
    let names = [
        "offer", "creator", "created", "rebuild", "fromage", "getId", "newest",
    ];

    let factories: Vec<_> = names.iter().map(|name| is_factory_method(name)).collect();

    assert_eq!(factories, vec![false; names.len()]);
}

#[test]
fn names_that_look_an_object_up_are_repository_queries() {
    let names = [
        "find",
        "findById",
        "findAll",
        "load",
        "loadAll",
        "fetchOrder",
    ];

    let queries: Vec<_> = names.iter().map(|name| is_repository_query(name)).collect();

    assert_eq!(queries, vec![true; names.len()]);
}

#[test]
fn names_that_only_start_with_a_query_word_are_not_repository_queries() {
    let names = ["finder", "loader", "fetched", "getById", "create"];

    let queries: Vec<_> = names.iter().map(|name| is_repository_query(name)).collect();

    assert_eq!(queries, vec![false; names.len()]);
}

#[test]
fn a_map_entry_is_a_data_structure_without_a_shape() {
    let names = [
        "java.util.Map$Entry",
        "java.util.AbstractMap$SimpleEntry",
        "java.util.AbstractMap$SimpleImmutableEntry",
    ];

    let classifications = classify_names(&Types::new(), &names);

    assert_eq!(
        classifications,
        vec![Classification::DataStructure; names.len()]
    );
}

fn getter(name: &str, field: &str) -> MethodShape {
    MethodShape::new(name).returning(string()).reading(field)
}

fn setter(name: &str, field: &str) -> MethodShape {
    MethodShape::new(name)
        .with_parameters(vec![string()])
        .writing(field)
}

fn behavior(name: &str) -> MethodShape {
    MethodShape::new(name)
}

fn class(name: &str, superclass: &str, methods: Vec<MethodShape>) -> TypeShape {
    methods.into_iter().fold(
        TypeShape::new(name, TypeKind::Class).with_superclass(TypeRef::class(superclass)),
        TypeShape::with_method,
    )
}

fn root(name: &str, methods: Vec<MethodShape>) -> TypeShape {
    class(name, "java.lang.Object", methods)
}

#[test]
fn behavior_inherited_from_a_superclass_makes_a_class_an_object() {
    let types = Types::new()
        .with_shape(root("Base", vec![behavior("charge")]))
        .with_shape(class("Address", "Base", vec![getter("getCity", "city")]));

    let classification = types.classify(&TypeRef::class("Address"));

    assert_eq!(classification, Classification::Object);
}

#[test]
fn behavior_inherited_through_several_superclasses_counts() {
    let types = Types::new()
        .with_shape(root("Root", vec![behavior("charge")]))
        .with_shape(class("Middle", "Root", vec![]))
        .with_shape(class("Address", "Middle", vec![getter("getCity", "city")]));

    let classification = types.classify(&TypeRef::class("Address"));

    assert_eq!(classification, Classification::Object);
}

#[test]
fn an_inherited_setter_together_with_own_behavior_makes_a_hybrid() {
    let types = Types::new()
        .with_shape(root("Base", vec![setter("setName", "name")]))
        .with_shape(class("Account", "Base", vec![behavior("withdraw")]));

    let classification = types.classify(&TypeRef::class("Account"));

    assert_eq!(classification, Classification::Hybrid);
}

#[test]
fn inherited_accessors_keep_a_class_a_data_structure() {
    let types = Types::new()
        .with_shape(root(
            "BaseDto",
            vec![getter("getId", "id"), setter("setId", "id")],
        ))
        .with_shape(class("Address", "BaseDto", vec![getter("getCity", "city")]));

    let classification = types.classify(&TypeRef::class("Address"));

    assert_eq!(classification, Classification::DataStructure);
}

#[test]
fn an_override_replaces_the_inherited_method() {
    let types = Types::new()
        .with_shape(root("Base", vec![behavior("getId")]))
        .with_shape(class("Address", "Base", vec![getter("getId", "id")]));

    let classification = types.classify(&TypeRef::class("Address"));

    assert_eq!(classification, Classification::DataStructure);
}

#[test]
fn an_interface_supertype_adds_no_behavior() {
    let types = Types::new()
        .with_shape(
            TypeShape::new("Describable", TypeKind::Interface).with_method(behavior("describe")),
        )
        .with_shape(
            TypeShape::new("Address", TypeKind::Class)
                .with_superclass(TypeRef::class("java.lang.Object"))
                .extending(TypeRef::class("Describable"))
                .with_method(getter("getCity", "city")),
        );

    let classification = types.classify(&TypeRef::class("Address"));

    assert_eq!(classification, Classification::DataStructure);
}

#[test]
fn a_class_with_a_superclass_missing_from_the_catalog_is_unknown_not_a_data_structure() {
    let address = class(
        "Address",
        "com.example.LibraryBase",
        vec![getter("getCity", "city")],
    );

    let classification = classify(address);

    assert_eq!(classification, Classification::Unknown);
}

#[test]
fn a_missing_supertype_higher_up_the_chain_makes_the_class_unknown() {
    let types = Types::new()
        .with_shape(class("Base", "com.example.LibraryBase", vec![]))
        .with_shape(class("Address", "Base", vec![getter("getCity", "city")]));

    let classification = types.classify(&TypeRef::class("Address"));

    assert_eq!(classification, Classification::Unknown);
}

#[test]
fn a_missing_interface_does_not_make_a_class_without_behavior_unknown() {
    let address = root("Address", vec![getter("getCity", "city")])
        .extending(TypeRef::class("com.example.LibraryMarker"));

    let classification = classify(address);

    assert_eq!(classification, Classification::DataStructure);
}

#[test]
fn a_class_with_no_declared_superclass_is_not_unknown() {
    let address = TypeShape::new("Address", TypeKind::Class).with_method(getter("getCity", "city"));

    let classification = classify(address);

    assert_eq!(classification, Classification::DataStructure);
}

#[test]
fn the_methods_of_object_in_the_catalog_are_not_inherited_behavior() {
    let object = TypeShape::new("java.lang.Object", TypeKind::Class)
        .with_method(behavior("getClass"))
        .with_method(behavior("notify"))
        .with_method(behavior("wait"));
    let types = Types::new()
        .with_shape(object)
        .with_shape(root("Address", vec![getter("getCity", "city")]));

    let classification = types.classify(&TypeRef::class("Address"));

    assert_eq!(classification, Classification::DataStructure);
}

#[test]
fn known_behavior_wins_over_a_missing_supertype() {
    let order = class("Order", "com.example.LibraryBase", vec![behavior("charge")]);
    let account = class(
        "Account",
        "com.example.LibraryBase",
        vec![behavior("withdraw"), setter("setBalance", "balance")],
    );

    let classifications = (classify(order), classify(account));

    assert_eq!(
        classifications,
        (Classification::Object, Classification::Hybrid)
    );
}

#[test]
fn supertypes_the_rule_knows_do_not_make_a_class_unknown() {
    let supertypes = [
        "java.lang.Object",
        "java.io.Serializable",
        "java.lang.Cloneable",
        "java.lang.Comparable",
        "java.util.Map$Entry",
    ];
    let dtos: Vec<String> = (0..supertypes.len()).map(|n| format!("Dto{n}")).collect();
    let types = supertypes
        .iter()
        .zip(&dtos)
        .fold(Types::new(), |types, (supertype, dto)| {
            types.with_shape(class(dto, supertype, vec![getter("getCity", "city")]))
        });

    let classifications: Vec<_> = dtos
        .iter()
        .map(|dto| types.classify(&TypeRef::class(dto)))
        .collect();

    assert_eq!(
        classifications,
        vec![Classification::DataStructure; dtos.len()]
    );
}

#[test]
fn a_record_stays_a_data_structure_with_a_missing_supertype() {
    let server = TypeShape::new("Server", TypeKind::Record)
        .extending(TypeRef::class("java.lang.Record"))
        .extending(TypeRef::class("com.example.LibraryMarker"));

    let classification = classify(server);

    assert_eq!(classification, Classification::DataStructure);
}

fn account(generated: bool) -> TypeShape {
    let accessor = |method: MethodShape| {
        if generated {
            method.generated()
        } else {
            method
        }
    };
    TypeShape::new("Account", TypeKind::Class)
        .with_field(FieldShape::new("balance", int()))
        .with_method(accessor(
            MethodShape::new("getBalance")
                .returning(int())
                .reading("balance"),
        ))
        .with_method(accessor(
            MethodShape::new("setBalance")
                .with_parameters(vec![int()])
                .writing("balance"),
        ))
        .with_method(MethodShape::new("withdraw").with_parameters(vec![int()]))
}

#[test]
fn a_hybrid_class_is_reported_with_its_behavior_methods_and_setters() {
    let types = Types::new().with_shape(account(true));

    let hybrid = types.hybrid_class("Account");

    assert_eq!(
        hybrid,
        Some(HybridClass::new(
            "Account",
            vec!["withdraw".to_owned()],
            vec!["setBalance".to_owned()]
        ))
    );
}

#[test]
fn generated_and_hand_written_accessors_give_the_same_hybrid_finding() {
    let generated = Types::new().with_shape(account(true));
    let hand_written = Types::new().with_shape(account(false));

    let findings = (
        generated.hybrid_class("Account"),
        hand_written.hybrid_class("Account"),
    );

    assert_eq!(findings.0, findings.1);
    assert!(findings.0.is_some());
}

#[test]
fn the_hybrid_finding_names_every_behavior_method_and_setter_once() {
    let types = Types::new().with_shape(
        TypeShape::new("Account", TypeKind::Class)
            .with_method(getter("getOwner", "owner"))
            .with_method(setter("setOwner", "owner"))
            .with_method(setter("setBranch", "branch"))
            .with_method(behavior("withdraw"))
            .with_method(behavior("withdraw").with_parameters(vec![int()]))
            .with_method(behavior("close")),
    );

    let hybrid = types.hybrid_class("Account");

    assert_eq!(
        hybrid
            .as_ref()
            .map(|finding| (finding.behavior_methods(), finding.setters())),
        Some((
            &["withdraw".to_owned(), "close".to_owned()][..],
            &["setOwner".to_owned(), "setBranch".to_owned()][..]
        ))
    );
}

#[test]
fn a_hybrid_finding_includes_methods_inherited_from_a_superclass() {
    let types = Types::new()
        .with_shape(root("Base", vec![setter("setName", "name")]))
        .with_shape(class("Account", "Base", vec![behavior("withdraw")]));

    let hybrid = types.hybrid_class("Account");

    assert_eq!(
        hybrid,
        Some(HybridClass::new(
            "Account",
            vec!["withdraw".to_owned()],
            vec!["setName".to_owned()]
        ))
    );
}

#[test]
fn a_hybrid_finding_names_its_class() {
    let types = Types::new().with_shape(account(true));

    let hybrid = types.hybrid_class("Account");

    assert_eq!(hybrid.as_ref().map(HybridClass::class), Some("Account"));
}

#[test]
fn types_that_are_not_hybrid_classes_have_no_hybrid_finding() {
    let types = Types::new()
        .with_shape(root(
            "Order",
            vec![behavior("charge"), getter("getId", "id")],
        ))
        .with_shape(root(
            "Address",
            vec![getter("getCity", "city"), setter("setCity", "city")],
        ))
        .with_shape(TypeShape::new("Server", TypeKind::Record))
        .with_shape(TypeShape::new("Status", TypeKind::Enum))
        .with_shape(TypeShape::new("Wallet", TypeKind::Interface).with_method(behavior("pay")))
        .with_shape(class(
            "Unresolved",
            "com.example.LibraryBase",
            vec![setter("setCity", "city")],
        ));

    let findings: Vec<_> = [
        "Order",
        "Address",
        "Server",
        "Status",
        "Wallet",
        "Unresolved",
        "Mystery",
        "java.lang.String",
    ]
    .iter()
    .map(|name| types.hybrid_class(name))
    .collect();

    assert_eq!(findings, vec![None; 8]);
}

fn cyclic_classes() -> Types {
    Types::new()
        .with_shape(class("First", "Second", vec![getter("getCity", "city")]))
        .with_shape(class("Second", "First", vec![getter("getId", "id")]))
        .with_shape(class("Itself", "Itself", vec![getter("getCity", "city")]))
}

fn cyclic_interfaces() -> Types {
    Types::new()
        .with_shape(
            TypeShape::new("Left", TypeKind::Interface)
                .extending(TypeRef::class("Right"))
                .with_method(MethodShape::new("describe").returning(string())),
        )
        .with_shape(TypeShape::new("Right", TypeKind::Interface).extending(TypeRef::class("Left")))
}

#[test]
fn classifying_a_class_in_a_cyclic_hierarchy_terminates() {
    let types = cyclic_classes();

    let classifications = classify_names(&types, &["First", "Second", "Itself"]);

    assert_eq!(classifications, vec![Classification::DataStructure; 3]);
}

#[test]
fn classifying_an_interface_in_a_cyclic_hierarchy_terminates() {
    let types = cyclic_interfaces();

    let classifications = classify_names(&types, &["Left", "Right"]);

    assert_eq!(classifications, vec![Classification::Object; 2]);
}

#[test]
fn a_return_type_is_found_through_a_cyclic_hierarchy() {
    let types = cyclic_interfaces();

    let returned = (
        types.return_type("Right", "describe", 0),
        types.return_type("Right", "missing", 0),
    );

    assert_eq!(returned, (Some(&string()), None));
}

#[test]
fn fluency_in_a_cyclic_hierarchy_terminates() {
    let types = cyclic_classes();

    let fluent = types.is_fluent("First", "missing", 0);

    assert!(!fluent);
}

fn with_jdk_shapes(names: &[&str], kind: TypeKind) -> Types {
    names.iter().fold(Types::new(), |types, name| {
        types.with_shape(
            TypeShape::new(name, kind)
                .with_method(behavior("compute"))
                .with_method(setter("setValue", "value")),
        )
    })
}

#[test]
fn jdk_value_names_stay_values_when_their_shapes_are_in_the_catalog() {
    let names = [
        "java.lang.String",
        "java.lang.Integer",
        "java.math.BigDecimal",
        "java.time.Instant",
        "java.util.UUID",
        "java.util.Date",
    ];
    let types = with_jdk_shapes(&names, TypeKind::Class);

    let classifications = classify_names(&types, &names);

    assert_eq!(classifications, vec![Classification::Value; names.len()]);
}

#[test]
fn jdk_container_names_stay_containers_when_their_shapes_are_in_the_catalog() {
    let classes = [
        "java.util.ArrayList",
        "java.util.Optional",
        "java.util.HashMap",
    ];
    let interfaces = [
        "java.util.List",
        "java.util.stream.Stream",
        "java.lang.Iterable",
    ];
    let types = with_jdk_shapes(&classes, TypeKind::Class);
    let types = interfaces.iter().fold(types, |types, name| {
        types.with_shape(TypeShape::new(name, TypeKind::Interface).with_method(behavior("size")))
    });

    let classifications = (
        classify_names(&types, &classes),
        classify_names(&types, &interfaces),
    );

    assert_eq!(
        classifications,
        (
            vec![Classification::Container; classes.len()],
            vec![Classification::Container; interfaces.len()]
        )
    );
}

#[test]
fn a_map_entry_stays_a_data_structure_when_its_shape_is_in_the_catalog() {
    let entry = TypeShape::new("java.util.Map$Entry", TypeKind::Interface)
        .with_method(behavior("comparingByKey"))
        .with_method(getter("getKey", "key"));
    let types = Types::new().with_shape(entry);

    let classification = types.classify(&TypeRef::class("java.util.Map$Entry"));

    assert_eq!(classification, Classification::DataStructure);
}

#[test]
fn a_jdk_type_with_behavior_and_setters_is_never_reported_as_a_hybrid_class() {
    let names = [
        "java.util.Date",
        "java.util.ArrayList",
        "java.util.Map$Entry",
    ];
    let types = with_jdk_shapes(&names, TypeKind::Class);

    let findings: Vec<_> = names.iter().map(|name| types.hybrid_class(name)).collect();

    assert_eq!(findings, vec![None; names.len()]);
}

#[test]
fn a_user_class_extending_a_jdk_value_with_a_shape_is_a_value() {
    let types = with_jdk_shapes(&["java.util.Date"], TypeKind::Class).with_shape(class(
        "Deadline",
        "java.util.Date",
        vec![behavior("isOverdue")],
    ));

    let classification = types.classify(&TypeRef::class("Deadline"));

    assert_eq!(classification, Classification::Value);
}

fn variable(name: &str) -> TypeRef {
    TypeRef::variable(name)
}

fn generic_of(name: &str, arguments: Vec<TypeRef>) -> TypeRef {
    TypeRef::generic(name, arguments)
}

fn order() -> TypeRef {
    TypeRef::class("Order")
}

fn generic_collections() -> Types {
    let list = TypeShape::new("java.util.List", TypeKind::Interface)
        .with_type_parameter("E")
        .with_method(
            MethodShape::new("get")
                .with_parameters(vec![int()])
                .returning(variable("E")),
        );
    let array_list = TypeShape::new("java.util.ArrayList", TypeKind::Class)
        .with_type_parameter("E")
        .with_superclass(TypeRef::class("java.lang.Object"))
        .extending(generic_of("java.util.List", vec![variable("E")]));
    let map = TypeShape::new("java.util.Map", TypeKind::Interface)
        .with_type_parameter("K")
        .with_type_parameter("V")
        .with_method(
            MethodShape::new("get")
                .with_parameters(vec![TypeRef::class("java.lang.Object")])
                .returning(variable("V")),
        )
        .with_method(MethodShape::new("entrySet").returning(generic_of(
            "java.util.Set",
            vec![generic_of(
                "java.util.Map$Entry",
                vec![variable("K"), variable("V")],
            )],
        )));
    let optional = TypeShape::new("java.util.Optional", TypeKind::Class)
        .with_type_parameter("T")
        .with_method(MethodShape::new("orElseThrow").returning(variable("T")))
        .with_method(
            MethodShape::new("map")
                .with_parameters(vec![TypeRef::class("java.util.function.Function")])
                .returning(generic_of("java.util.Optional", vec![variable("U")])),
        );
    Types::new()
        .with_shape(list)
        .with_shape(array_list)
        .with_shape(map)
        .with_shape(optional)
}

#[test]
fn a_type_argument_is_substituted_through_the_supertype_chain() {
    let types = generic_collections();
    let receiver = generic_of("java.util.ArrayList", vec![order()]);

    let returned = types.return_type_on(&receiver, "get", 1);

    assert_eq!(returned, Some(order()));
}

#[test]
fn a_map_value_type_is_substituted_for_the_value_parameter() {
    let types = generic_collections();
    let receiver = generic_of("java.util.Map", vec![string(), order()]);

    let returned = types.return_type_on(&receiver, "get", 1);

    assert_eq!(returned, Some(order()));
}

#[test]
fn an_optional_gives_back_its_type_argument() {
    let types = generic_collections();
    let receiver = generic_of("java.util.Optional", vec![order()]);

    let returned = types.return_type_on(&receiver, "orElseThrow", 0);

    assert_eq!(returned, Some(order()));
}

#[test]
fn arguments_are_substituted_inside_nested_generic_return_types() {
    let types = generic_collections();
    let receiver = generic_of("java.util.Map", vec![string(), order()]);

    let returned = types.return_type_on(&receiver, "entrySet", 0);

    assert_eq!(
        returned,
        Some(generic_of(
            "java.util.Set",
            vec![generic_of("java.util.Map$Entry", vec![string(), order()])]
        ))
    );
}

#[test]
fn a_raw_receiver_leaves_the_type_variable() {
    let types = generic_collections();

    let returned = (
        types.return_type_on(&TypeRef::class("java.util.ArrayList"), "get", 1),
        types.return_type_on(&TypeRef::class("java.util.List"), "get", 1),
    );

    assert_eq!(returned, (Some(variable("E")), Some(variable("E"))));
}

#[test]
fn an_unbound_variable_argument_stays_a_variable() {
    let types = generic_collections();
    let receiver = generic_of("java.util.List", vec![variable("T")]);

    let returned = types.return_type_on(&receiver, "get", 1);

    assert_eq!(returned, Some(variable("T")));
}

#[test]
fn a_method_level_type_parameter_stays_a_variable() {
    let types = generic_collections();
    let receiver = generic_of("java.util.Optional", vec![order()]);

    let returned = types.return_type_on(&receiver, "map", 1);

    assert_eq!(
        returned,
        Some(generic_of("java.util.Optional", vec![variable("U")]))
    );
}

#[test]
fn concrete_arguments_in_a_subtype_declaration_are_substituted() {
    let types = generic_collections().with_shape(
        TypeShape::new("OrderList", TypeKind::Class)
            .with_superclass(generic_of("java.util.ArrayList", vec![order()])),
    );

    let returned = types.return_type_on(&TypeRef::class("OrderList"), "get", 1);

    assert_eq!(returned, Some(order()));
}

#[test]
fn renamed_type_parameters_are_substituted_along_the_whole_chain() {
    let types = generic_collections().with_shape(
        TypeShape::new("Registry", TypeKind::Class)
            .with_type_parameter("Item")
            .with_superclass(generic_of(
                "java.util.ArrayList",
                vec![generic_of("java.util.Optional", vec![variable("Item")])],
            )),
    );
    let receiver = generic_of("Registry", vec![order()]);

    let returned = types.return_type_on(&receiver, "get", 1);

    assert_eq!(
        returned,
        Some(generic_of("java.util.Optional", vec![order()]))
    );
}

#[test]
fn a_return_type_that_is_not_generic_is_returned_unchanged() {
    let types = Types::new().with_shape(
        TypeShape::new("Order", TypeKind::Class)
            .with_type_parameter("T")
            .with_method(MethodShape::new("getCustomer").returning(TypeRef::class("Customer")))
            .with_method(MethodShape::new("getIds").returning(TypeRef::array(int())))
            .with_method(MethodShape::new("getItems").returning(TypeRef::array(variable("T")))),
    );
    let receiver = generic_of("Order", vec![string()]);

    let returned = (
        types.return_type_on(&receiver, "getCustomer", 0),
        types.return_type_on(&receiver, "getIds", 0),
        types.return_type_on(&receiver, "getItems", 0),
    );

    assert_eq!(
        returned,
        (
            Some(TypeRef::class("Customer")),
            Some(TypeRef::array(int())),
            Some(TypeRef::array(string()))
        )
    );
}

#[test]
fn an_unknown_receiver_or_method_has_no_substituted_return_type() {
    let types = generic_collections();

    let returned = (
        types.return_type_on(&generic_of("Mystery", vec![order()]), "get", 1),
        types.return_type_on(&generic_of("java.util.List", vec![order()]), "missing", 1),
        types.return_type_on(&TypeRef::array(order()), "get", 1),
        types.return_type_on(&variable("T"), "get", 1),
    );

    assert_eq!(returned, (None, None, None, None));
}

#[test]
fn overloads_with_different_substituted_returns_are_ambiguous() {
    let types = Types::new().with_shape(
        TypeShape::new("Box", TypeKind::Class)
            .with_type_parameter("T")
            .with_method(
                MethodShape::new("pick")
                    .with_parameters(vec![int()])
                    .returning(variable("T")),
            )
            .with_method(
                MethodShape::new("pick")
                    .with_parameters(vec![string()])
                    .returning(string()),
            ),
    );

    let returned = types.return_type_on(&generic_of("Box", vec![order()]), "pick", 1);

    assert_eq!(returned, None);
}

#[test]
fn a_substituted_return_type_terminates_in_a_cyclic_hierarchy() {
    let types = Types::new()
        .with_shape(
            TypeShape::new("Left", TypeKind::Interface)
                .with_type_parameter("T")
                .extending(generic_of("Right", vec![variable("T")])),
        )
        .with_shape(
            TypeShape::new("Right", TypeKind::Interface)
                .with_type_parameter("T")
                .extending(generic_of("Left", vec![variable("T")]))
                .with_method(MethodShape::new("value").returning(variable("T"))),
        );

    let returned = types.return_type_on(&generic_of("Left", vec![order()]), "value", 0);

    assert_eq!(returned, Some(order()));
}

#[test]
fn the_element_of_a_direct_container_is_found_with_or_without_shapes() {
    let containers = [
        generic_of("java.util.List", vec![order()]),
        generic_of("java.util.stream.Stream", vec![order()]),
        generic_of("java.util.Optional", vec![order()]),
        generic_of("java.util.Map", vec![string(), order()]),
        generic_of("java.util.ArrayList", vec![order()]),
        TypeRef::array(order()),
    ];
    let catalogs = [Types::new(), generic_collections()];

    let elements: Vec<_> = catalogs
        .iter()
        .flat_map(|types| {
            containers
                .iter()
                .map(|container| types.element_type_of(container))
        })
        .collect();

    assert_eq!(
        elements,
        vec![Some(order()); containers.len() * catalogs.len()]
    );
}

#[test]
fn the_element_of_a_subtype_of_a_container_is_found_through_its_declaration() {
    let types = generic_collections()
        .with_shape(
            TypeShape::new("OrderList", TypeKind::Class)
                .with_superclass(generic_of("java.util.ArrayList", vec![order()])),
        )
        .with_shape(
            TypeShape::new("Registry", TypeKind::Class)
                .with_type_parameter("V")
                .with_superclass(generic_of(
                    "java.util.HashMap",
                    vec![string(), variable("V")],
                )),
        )
        .with_shape(
            TypeShape::new("Page", TypeKind::Class)
                .with_type_parameter("T")
                .extending(generic_of("java.lang.Iterable", vec![variable("T")])),
        );

    let elements = (
        types.element_type_of(&TypeRef::class("OrderList")),
        types.element_type_of(&generic_of("Registry", vec![order()])),
        types.element_type_of(&generic_of("Page", vec![order()])),
    );

    assert_eq!(elements, (Some(order()), Some(order()), Some(order())));
}

#[test]
fn the_element_of_a_raw_container_subtype_stays_a_variable() {
    let types = Types::new().with_shape(
        TypeShape::new("Registry", TypeKind::Class)
            .with_type_parameter("V")
            .with_superclass(generic_of(
                "java.util.HashMap",
                vec![string(), variable("V")],
            )),
    );

    let element = types.element_type_of(&TypeRef::class("Registry"));

    assert_eq!(element, Some(variable("V")));
}

#[test]
fn a_raw_container_or_a_non_container_has_no_element() {
    let types = Types::new().with_shape(
        TypeShape::new("Page", TypeKind::Class)
            .with_type_parameter("T")
            .with_method(MethodShape::new("first").returning(variable("T"))),
    );

    let elements = (
        types.element_type_of(&TypeRef::class("java.util.List")),
        types.element_type_of(&generic_of("Page", vec![order()])),
        types.element_type_of(&order()),
        types.element_type_of(&int()),
        types.element_type_of(&variable("T")),
    );

    assert_eq!(elements, (None, None, None, None, None));
}

#[test]
fn the_element_of_a_type_in_a_cyclic_hierarchy_terminates() {
    let types = Types::new()
        .with_shape(class("First", "Second", vec![]))
        .with_shape(class("Second", "First", vec![]));

    let element = types.element_type_of(&TypeRef::class("First"));

    assert_eq!(element, None);
}

#[test]
fn only_types_declared_in_the_analysed_sources_are_project_types() {
    let types = Types::new().with_project_types(["shop.Order".to_owned()]);

    let answers = (
        types.is_project_type("shop.Order"),
        types.is_project_type("java.lang.String"),
    );

    assert_eq!(answers, (true, false));
}
