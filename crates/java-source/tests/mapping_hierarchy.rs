mod mapped;

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::{Expression, Method};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};
use mapped::{call, class, method_f_in, parameter};

fn shape(name: &str, kind: TypeKind) -> TypeShape {
    TypeShape::new(name, kind)
}

fn returning(name: &str, returned: TypeRef) -> MethodShape {
    MethodShape::new(name).returning(returned)
}

fn string() -> TypeRef {
    class("java.lang.String")
}

fn generic(name: &str, arguments: Vec<TypeRef>) -> TypeRef {
    TypeRef::generic(name, arguments)
}

fn f(line: usize) -> Method {
    Method::new("OrderService", "f", line)
}

fn mapped_in(parameters: &str, body: &str, types: &Types) -> Method {
    let source = format!("class OrderService {{\n void f({parameters}) {{\n  {body}\n }}\n}}");

    method_f_in(&source, types)
}

fn calling(receiver: Expression, name: &str, returned: Option<TypeRef>) -> Method {
    let call = call(receiver, name, 3);
    let call = match returned {
        Some(returned) => call.returning(returned),
        None => call,
    };

    f(2).call(call.then("go", 3))
}

fn object() -> TypeShape {
    shape("java.lang.Object", TypeKind::Class)
        .with_method(returning("getClass", class("java.lang.Class")))
        .with_method(returning("toString", string()))
}

fn enum_supertype() -> TypeShape {
    shape("java.lang.Enum", TypeKind::Class)
        .with_type_parameter("E")
        .with_method(returning("name", string()))
        .with_method(returning("ordinal", TypeRef::primitive("int")))
        .with_method(returning(
            "getDeclaringClass",
            generic("java.lang.Class", vec![TypeRef::variable("E")]),
        ))
}

fn record_supertype() -> TypeShape {
    shape("java.lang.Record", TypeKind::Class).with_method(returning("describe", string()))
}

fn root_types() -> Types {
    Types::new()
        .with_shape(object())
        .with_shape(enum_supertype())
        .with_shape(record_supertype())
        .with_shape(shape("java.lang.String", TypeKind::Class))
}

fn status() -> TypeShape {
    shape("Status", TypeKind::Enum)
        .with_superclass(generic("java.lang.Enum", vec![class("Status")]))
}

#[test]
fn a_type_the_catalog_does_not_know_still_finds_the_methods_of_object() {
    let mapped = mapped_in("Ghost ghost", "ghost.getClass().go();", &root_types());

    assert_eq!(
        mapped,
        calling(
            parameter("ghost", class("Ghost")),
            "getClass",
            Some(class("java.lang.Class"))
        )
    );
}

#[test]
fn a_method_declared_on_the_type_wins_over_the_methods_of_object() {
    let order = shape("Order", TypeKind::Class).with_method(returning("toString", class("Text")));
    let types = root_types().with_shape(order);

    let mapped = mapped_in("Order order", "order.toString().go();", &types);

    assert_eq!(
        mapped,
        calling(
            parameter("order", class("Order")),
            "toString",
            Some(class("Text"))
        )
    );
}

#[test]
fn an_enum_finds_the_methods_of_the_enum_supertype() {
    let types = root_types().with_shape(status());

    let mapped = mapped_in(
        "Status status",
        "status.ordinal().go();\n  status.name().go();",
        &types,
    );

    let status = parameter("status", class("Status"));
    let ordinal = call(status.clone(), "ordinal", 3).returning(TypeRef::primitive("int"));
    let name = call(status, "name", 4).returning(string());
    assert_eq!(
        mapped,
        f(2).call(ordinal.then("go", 3)).call(name.then("go", 4))
    );
}

#[test]
fn the_enum_supertype_is_bound_to_the_enum_itself() {
    let types = root_types().with_shape(status());

    let mapped = mapped_in("Status status", "status.getDeclaringClass().go();", &types);

    assert_eq!(
        mapped,
        calling(
            parameter("status", class("Status")),
            "getDeclaringClass",
            Some(generic("java.lang.Class", vec![class("Status")]))
        )
    );
}

#[test]
fn a_record_finds_the_methods_of_the_record_supertype() {
    let server = shape("Server", TypeKind::Record).with_superclass(class("java.lang.Record"));
    let types = root_types().with_shape(server);

    let mapped = mapped_in("Server server", "server.describe().go();", &types);

    assert_eq!(
        mapped,
        calling(
            parameter("server", class("Server")),
            "describe",
            Some(string())
        )
    );
}

#[test]
fn a_class_does_not_see_the_methods_of_the_enum_or_record_supertypes() {
    let types = root_types().with_shape(shape("Order", TypeKind::Class));

    let mapped = mapped_in(
        "Order order",
        "order.name().go();\n  order.describe().go();",
        &types,
    );

    let order = parameter("order", class("Order"));
    assert_eq!(
        mapped,
        f(2).call(call(order.clone(), "name", 3).then("go", 3))
            .call(call(order, "describe", 4).then("go", 4))
    );
}

fn cyclic_interfaces() -> Types {
    Types::new()
        .with_shape(
            shape("Left", TypeKind::Interface)
                .with_type_parameter("T")
                .extending(generic("Right", vec![TypeRef::variable("T")]))
                .with_method(returning("describe", string())),
        )
        .with_shape(
            shape("Right", TypeKind::Interface)
                .with_type_parameter("T")
                .extending(generic("Left", vec![TypeRef::variable("T")]))
                .with_method(returning("value", TypeRef::variable("T"))),
        )
        .with_shape(shape("java.lang.String", TypeKind::Class))
}

#[test]
fn a_method_is_found_through_a_cyclic_hierarchy() {
    let receiver = generic("Right", vec![class("Order")]);

    let mapped = mapped_in(
        "Right<Order> right",
        "right.describe().go();\n  right.missing().go();",
        &cyclic_interfaces(),
    );

    let right = parameter("right", receiver);
    assert_eq!(
        mapped,
        f(2).call(
            call(right.clone(), "describe", 3)
                .returning(string())
                .then("go", 3)
        )
        .call(call(right, "missing", 4).then("go", 4))
    );
}

#[test]
fn a_type_argument_is_substituted_through_a_cyclic_hierarchy() {
    let mapped = mapped_in(
        "Left<Order> left",
        "left.value().go();",
        &cyclic_interfaces(),
    );

    assert_eq!(
        mapped,
        calling(
            parameter("left", generic("Left", vec![class("Order")])),
            "value",
            Some(class("Order"))
        )
    );
}
