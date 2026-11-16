#![allow(dead_code)]

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

const STREAM: &str = "java.util.stream.Stream";
const COLLECTOR: &str = "java.util.stream.Collector";
const COLLECTORS: &str = "java.util.stream.Collectors";
const LIST: &str = "java.util.List";
const MAP: &str = "java.util.Map";
const FUNCTION: &str = "java.util.function.Function";

pub const IMPORTS: &str = "import java.util.List; import java.util.stream.Collectors;";

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

fn collector(element: TypeRef, accumulation: TypeRef, result: TypeRef) -> TypeRef {
    generic(COLLECTOR, &[element, accumulation, result])
}

fn function(argument: TypeRef, result: TypeRef) -> TypeRef {
    generic(FUNCTION, &[argument, result])
}

pub fn world() -> Types {
    let object = TypeRef::class("java.lang.Object");
    let collectors = class(COLLECTORS)
        .with_method(
            MethodShape::new("toList")
                .as_static()
                .with_type_parameter("T")
                .returning(collector(
                    variable("T"),
                    object.clone(),
                    generic(LIST, &[variable("T")]),
                )),
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
                    object,
                    generic(MAP, &[variable("K"), variable("D")]),
                )),
        );
    Types::new()
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
        .with_shape(interface(MAP, &["K", "V"]))
        .with_shape(
            class("Order")
                .with_method(MethodShape::new("getCustomer").returning(TypeRef::class("Customer"))),
        )
        .with_shape(class("Customer"))
}

pub fn nested_collectors(depth: usize) -> String {
    let collector = (0..depth).fold("Collectors.toList()".to_owned(), |inner, _| {
        format!("Collectors.groupingBy(Order::getCustomer, {inner})")
    });
    format!("orders.stream().collect({collector});")
}

pub fn nested_lambda_collectors(depth: usize) -> String {
    let collector = (0..depth).fold("Collectors.toList()".to_owned(), |inner, _| {
        format!("Collectors.groupingBy(order -> order.getCustomer(), {inner})")
    });
    format!("orders.stream().collect({collector});")
}

pub fn collected_chain(length: usize) -> String {
    let collects = vec!["collect(Collectors.toList()).stream()"; length].join(".");
    format!("orders.stream().{collects};")
}

pub fn snippet(statement: &str) -> String {
    format!("void group(List<Order> orders) {{\n    {statement}\n}}")
}
