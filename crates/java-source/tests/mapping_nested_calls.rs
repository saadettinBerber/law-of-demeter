mod mapped;

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::{Call, Expression, Method};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};
use mapped::{call, class, method_f_in};

fn shape(name: &str) -> TypeShape {
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

fn list_of(element: TypeRef) -> TypeRef {
    generic("List", &[element])
}

fn map_of(key: TypeRef, value: TypeRef) -> TypeRef {
    generic("Map", &[key, value])
}

fn stream_of(element: TypeRef) -> TypeRef {
    generic("Stream", &[element])
}

fn function_of(argument: TypeRef, result: TypeRef) -> TypeRef {
    generic("Function", &[argument, result])
}

fn collector_of(element: TypeRef, accumulation: TypeRef, result: TypeRef) -> TypeRef {
    generic("Collector", &[element, accumulation, result])
}

fn collected(element: TypeRef, result: TypeRef) -> TypeRef {
    collector_of(element, class("java.lang.Object"), result)
}

fn order() -> TypeRef {
    class("Order")
}

fn customer() -> TypeRef {
    class("Customer")
}

fn wallet() -> TypeRef {
    class("Wallet")
}

fn money() -> TypeRef {
    class("Money")
}

fn collector_method(name: &str, variables: &[&str]) -> MethodShape {
    variables
        .iter()
        .fold(MethodShape::new(name).as_static(), |method, variable| {
            method.with_type_parameter(variable)
        })
}

fn collectors() -> TypeShape {
    let to_list = collector_method("toList", &["T"])
        .returning(collected(variable("T"), list_of(variable("T"))));
    let grouping = collector_method("groupingBy", &["T", "K"])
        .with_parameters(vec![function_of(variable("T"), variable("K"))])
        .returning(collected(
            variable("T"),
            map_of(variable("K"), list_of(variable("T"))),
        ));
    let grouping_downstream = collector_method("groupingBy", &["T", "K", "A", "D"])
        .with_parameters(vec![
            function_of(variable("T"), variable("K")),
            collector_of(variable("T"), variable("A"), variable("D")),
        ])
        .returning(collected(
            variable("T"),
            map_of(variable("K"), variable("D")),
        ));
    let mapping = collector_method("mapping", &["T", "U", "A", "R"])
        .with_parameters(vec![
            function_of(variable("T"), variable("U")),
            collector_of(variable("U"), variable("A"), variable("R")),
        ])
        .returning(collected(variable("T"), variable("R")));
    shape("Collectors")
        .with_method(to_list)
        .with_method(grouping)
        .with_method(grouping_downstream)
        .with_method(mapping)
}

fn world() -> Types {
    let collecting = MethodShape::new("collect")
        .with_type_parameter("R")
        .with_type_parameter("A")
        .with_parameters(vec![collector_of(
            variable("T"),
            variable("A"),
            variable("R"),
        )])
        .returning(variable("R"));
    let applying = MethodShape::new("apply")
        .with_parameters(vec![variable("T")])
        .returning(variable("R"))
        .as_abstract();
    Types::new()
        .with_shape(shape("java.lang.Object"))
        .with_shape(shape("java.lang.String"))
        .with_shape(collectors())
        .with_shape(interface("Collector", &["T", "A", "R"]))
        .with_shape(interface("Stream", &["T"]).with_method(collecting))
        .with_shape(interface("Function", &["T", "R"]).with_method(applying))
        .with_shape(
            interface("List", &["E"])
                .with_method(MethodShape::new("stream").returning(stream_of(variable("E")))),
        )
        .with_shape(interface("Map", &["K", "V"]))
        .with_shape(
            shape("Order")
                .with_method(MethodShape::new("getCustomer").returning(customer()))
                .with_method(MethodShape::new("getTotal").returning(money())),
        )
        .with_shape(
            shape("Customer").with_method(MethodShape::new("getWallet").returning(wallet())),
        )
        .with_shape(shape("Wallet"))
        .with_shape(shape("Money"))
        .with_shape(
            shape("Util")
                .with_method(taking(order()))
                .with_method(taking_text()),
        )
}

fn taking(element: TypeRef) -> MethodShape {
    MethodShape::new("take")
        .as_static()
        .with_parameters(vec![collected(element.clone(), list_of(element))])
        .returning(wallet())
}

fn taking_text() -> MethodShape {
    MethodShape::new("take")
        .as_static()
        .with_parameters(vec![class("java.lang.String")])
        .returning(customer())
}

fn f(line: usize) -> Method {
    Method::new("OrderService", "f", line)
}

fn mapped_collect(collector: &str) -> Method {
    let source = format!(
        "class OrderService {{\n void f(List<Order> orders, List<Customer> customers) {{\n  {collector};\n }}\n}}"
    );

    method_f_in(&source, &world())
}

fn collectors_call(name: &str) -> Call {
    call(Expression::static_type(class("Collectors")), name, 3)
}

#[test]
fn an_argument_that_is_a_generic_call_selects_the_overload_by_its_return_class() {
    let mapped = mapped_collect("Util.take(Collectors.toList())");

    let take = call(Expression::static_type(class("Util")), "take", 3)
        .with(Expression::return_of(collectors_call("toList")))
        .returning(wallet());
    assert_eq!(mapped, f(2).call(take));
}
