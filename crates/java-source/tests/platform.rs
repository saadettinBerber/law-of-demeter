use law_of_demeter_core::domain::classification::{Classification, Types};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};
use law_of_demeter_java_source::java_platform;

fn platform_types() -> Types {
    Types::new().with_platform(java_platform())
}

fn classify_names(types: &Types, names: &[&str]) -> Vec<Classification> {
    names
        .iter()
        .map(|name| types.classify(&TypeRef::class(name)))
        .collect()
}

fn getter(name: &str, field: &str) -> MethodShape {
    MethodShape::new(name)
        .returning(TypeRef::class("java.lang.String"))
        .reading(field)
}

fn setter(name: &str, field: &str) -> MethodShape {
    MethodShape::new(name)
        .with_parameters(vec![TypeRef::class("java.lang.String")])
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

fn with_jdk_shapes(names: &[&str], kind: TypeKind) -> Types {
    names.iter().fold(platform_types(), |types, name| {
        types.with_shape(
            TypeShape::new(name, kind)
                .with_method(behavior("compute"))
                .with_method(setter("setValue", "value")),
        )
    })
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

    let classifications = classify_names(&platform_types(), &names);

    assert_eq!(classifications, vec![Classification::Value; names.len()]);
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

    let classifications = classify_names(&platform_types(), &names);

    assert_eq!(
        classifications,
        vec![Classification::Container; names.len()]
    );
}

#[test]
fn a_map_entry_is_a_data_structure_without_a_shape() {
    let names = [
        "java.util.Map$Entry",
        "java.util.AbstractMap$SimpleEntry",
        "java.util.AbstractMap$SimpleImmutableEntry",
    ];

    let classifications = classify_names(&platform_types(), &names);

    assert_eq!(
        classifications,
        vec![Classification::DataStructure; names.len()]
    );
}

#[test]
fn java_lang_class_is_a_value_with_or_without_a_shape() {
    let class_shape = TypeShape::new("java.lang.Class", TypeKind::Class)
        .with_type_parameter("T")
        .with_method(behavior("getName"));
    let name = TypeRef::generic("java.lang.Class", vec![TypeRef::class("Order")]);

    let classifications = (
        platform_types().classify(&name),
        platform_types().with_shape(class_shape).classify(&name),
    );

    assert_eq!(
        classifications,
        (Classification::Value, Classification::Value)
    );
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
    let types = platform_types().with_shape(entry);

    let classification = types.classify(&TypeRef::class("java.util.Map$Entry"));

    assert_eq!(classification, Classification::DataStructure);
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
        .fold(platform_types(), |types, (supertype, dto)| {
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
fn record_and_enum_add_no_behavior_to_a_class_that_extends_them() {
    let supertypes = ["java.lang.Record", "java.lang.Enum"];
    let dtos: Vec<String> = (0..supertypes.len()).map(|n| format!("Dto{n}")).collect();
    let types = supertypes
        .iter()
        .zip(&dtos)
        .fold(platform_types(), |types, (supertype, dto)| {
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
fn every_java_type_finds_the_methods_of_object() {
    let object = TypeShape::new("java.lang.Object", TypeKind::Class)
        .with_method(MethodShape::new("hashCode").returning(TypeRef::primitive("int")));
    let types = platform_types()
        .with_shape(object)
        .with_shape(TypeShape::new("shop.Order", TypeKind::Class));

    let returned = types.return_type_on(&TypeRef::class("shop.Order"), "hashCode", 0);

    assert_eq!(returned, Some(TypeRef::primitive("int")));
}

#[test]
fn equals_hash_code_to_string_and_can_equal_are_not_behavior() {
    let object = TypeRef::class("java.lang.Object");
    let address = class(
        "shop.Address",
        "java.lang.Object",
        vec![
            getter("getCity", "city"),
            MethodShape::new("equals")
                .with_parameters(vec![object.clone()])
                .returning(TypeRef::primitive("boolean")),
            MethodShape::new("hashCode").returning(TypeRef::primitive("int")),
            MethodShape::new("toString").returning(TypeRef::class("java.lang.String")),
            MethodShape::new("canEqual")
                .with_parameters(vec![object])
                .returning(TypeRef::primitive("boolean")),
        ],
    );
    let types = platform_types().with_shape(address);

    let classification = types.classify(&TypeRef::class("shop.Address"));

    assert_eq!(classification, Classification::DataStructure);
}
