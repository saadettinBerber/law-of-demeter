mod mapped;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};
use mapped::{class, method_f_in};

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

fn world() -> Types {
    let mapping = MethodShape::new("map")
        .with_type_parameter("R")
        .with_parameters(vec![generic("Function", &[variable("T"), variable("R")])])
        .returning(generic("Stream", &[variable("R")]));
    let applying = MethodShape::new("apply")
        .with_parameters(vec![variable("T")])
        .returning(variable("R"))
        .as_abstract();
    let node = TypeShape::new("Node", TypeKind::Class)
        .with_method(MethodShape::new("kids").returning(generic("Stream", &[class("Node")])));
    Types::new()
        .with_shape(interface("Stream", &["T"]).with_method(mapping))
        .with_shape(interface("Function", &["T", "R"]).with_method(applying))
        .with_shape(node)
}

const BUDGET: Duration = Duration::from_secs(3);

fn maps_within_budget(body: String) -> bool {
    let (done, finished) = mpsc::channel();
    thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || {
            let source = format!("class OrderService {{\n void f(Node node) {{\n  {body}\n }}\n}}");
            method_f_in(&source, &world());
            let _ = done.send(());
        })
        .unwrap();
    finished.recv_timeout(BUDGET).is_ok()
}

fn chain(length: usize) -> String {
    let maps = vec!["map(n -> n)"; length].join(".");
    format!("node.kids().{maps};")
}

fn nested(depth: usize) -> String {
    let inner = (1..depth).fold("n0".to_owned(), |inner, level| {
        format!("n{level}.kids().map(n{} -> {inner})", level - 1)
    });
    format!("node.kids().map(n{} -> {inner});", depth - 1)
}

#[test]
fn a_long_chain_of_lambda_calls_maps_in_linear_time() {
    assert!(maps_within_budget(chain(600)));
}

#[test]
fn deeply_nested_lambda_calls_map_in_linear_time() {
    assert!(maps_within_budget(nested(24)));
}
