mod mapped;

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::{Call, Expression, Lambda, Method, MethodReference};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};
use law_of_demeter_java_source::java_platform;
use mapped::{call, class, method_f_in, parameter};

const STREAM: &str = "java.util.stream.Stream";
const COLLECTOR: &str = "java.util.stream.Collector";
const COLLECTORS: &str = "java.util.stream.Collectors";
const FUNCTION: &str = "java.util.function.Function";
const LIST: &str = "java.util.List";
const MAP: &str = "java.util.Map";
const SET: &str = "java.util.Set";
const COLLECTION: &str = "java.util.Collection";
const ENTRY: &str = "java.util.Map$Entry";

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

fn order() -> TypeRef {
    class("Order")
}

fn customer() -> TypeRef {
    class("Customer")
}

fn list_of(element: TypeRef) -> TypeRef {
    generic(LIST, &[element])
}

fn stream_of(element: TypeRef) -> TypeRef {
    generic(STREAM, &[element])
}

fn map_of(key: TypeRef, value: TypeRef) -> TypeRef {
    generic(MAP, &[key, value])
}

fn function_of(argument: TypeRef, result: TypeRef) -> TypeRef {
    generic(FUNCTION, &[argument, result])
}

fn collector_of(element: TypeRef, result: TypeRef) -> TypeRef {
    generic(COLLECTOR, &[element, class("java.lang.Object"), result])
}

fn entry_of(key: TypeRef, value: TypeRef) -> TypeRef {
    generic(ENTRY, &[key, value])
}

fn collectors() -> TypeShape {
    let grouping = MethodShape::new("groupingBy")
        .as_static()
        .with_type_parameter("T")
        .with_type_parameter("K")
        .with_parameters(vec![function_of(variable("T"), variable("K"))])
        .returning(collector_of(
            variable("T"),
            map_of(variable("K"), list_of(variable("T"))),
        ));
    let to_map = MethodShape::new("toMap")
        .as_static()
        .with_type_parameter("T")
        .with_type_parameter("K")
        .with_type_parameter("U")
        .with_parameters(vec![
            function_of(variable("T"), variable("K")),
            function_of(variable("T"), variable("U")),
        ])
        .returning(collector_of(
            variable("T"),
            map_of(variable("K"), variable("U")),
        ));
    let collecting_and_then = MethodShape::new("collectingAndThen")
        .as_static()
        .with_type_parameter("T")
        .with_type_parameter("R")
        .with_type_parameter("RR")
        .with_parameters(vec![
            collector_of(variable("T"), variable("R")),
            function_of(variable("R"), variable("RR")),
        ])
        .returning(collector_of(variable("T"), variable("RR")));
    let to_list = MethodShape::new("toList")
        .as_static()
        .with_type_parameter("T")
        .returning(collector_of(variable("T"), list_of(variable("T"))));
    shape(COLLECTORS)
        .with_method(grouping)
        .with_method(to_map)
        .with_method(collecting_and_then)
        .with_method(to_list)
}

fn world() -> Types {
    let mapping = MethodShape::new("map")
        .with_type_parameter("R")
        .with_parameters(vec![function_of(variable("T"), variable("R"))])
        .returning(stream_of(variable("R")));
    let collecting = MethodShape::new("collect")
        .with_type_parameter("R")
        .with_parameters(vec![collector_of(variable("T"), variable("R"))])
        .returning(variable("R"));
    let applying = MethodShape::new("apply")
        .with_parameters(vec![variable("T")])
        .returning(variable("R"))
        .as_abstract();
    let streaming = MethodShape::new("stream").returning(stream_of(variable("E")));
    let getting = MethodShape::new("get")
        .with_parameters(vec![TypeRef::primitive("int")])
        .returning(variable("E"));
    let streaming_set = MethodShape::new("stream").returning(stream_of(variable("E")));
    let keys = MethodShape::new("keySet").returning(generic(SET, &[variable("K")]));
    let values = MethodShape::new("values").returning(generic(COLLECTION, &[variable("V")]));
    let entries = MethodShape::new("entrySet")
        .returning(generic(SET, &[entry_of(variable("K"), variable("V"))]));
    let key = MethodShape::new("getKey").returning(variable("K"));
    let value = MethodShape::new("getValue").returning(variable("V"));
    Types::new()
        .with_platform(java_platform())
        .with_shape(shape("java.lang.Object"))
        .with_shape(collectors())
        .with_shape(interface(COLLECTOR, &["T", "A", "R"]))
        .with_shape(
            interface(STREAM, &["T"])
                .with_method(mapping)
                .with_method(collecting),
        )
        .with_shape(interface(FUNCTION, &["T", "R"]).with_method(applying))
        .with_shape(
            interface(LIST, &["E"])
                .with_method(streaming)
                .with_method(getting),
        )
        .with_shape(
            interface(MAP, &["K", "V"])
                .with_method(keys)
                .with_method(values)
                .with_method(entries),
        )
        .with_shape(
            interface(ENTRY, &["K", "V"])
                .with_method(key)
                .with_method(value),
        )
        .with_shape(interface(COLLECTION, &["E"]))
        .with_shape(interface(SET, &["E"]).with_method(streaming_set))
        .with_shape(
            shape("Order").with_method(MethodShape::new("getCustomer").returning(customer())),
        )
        .with_shape(shape("Customer"))
}

fn mapped(statement: &str) -> Method {
    let source = format!(
        "import java.util.List; import java.util.Map; import java.util.stream.Collectors;\nclass OrderService {{\n void f(List<Order> orders, Map<Customer, List<Order>> grouped, Map<Customer, Order> byCustomer, Map.Entry<Customer, Order> entry) {{\n  {statement};\n }}\n}}"
    );

    method_f_in(&source, &world())
}

fn f() -> Method {
    Method::new("OrderService", "f", 3)
}

fn streamed_orders() -> Expression {
    let stream = call(parameter("orders", list_of(order())), "stream", 4);
    Expression::return_of(stream.returning(stream_of(order())).keeping_entries())
}

fn customer_of_order() -> Expression {
    let read = call(Expression::local("order"), "getCustomer", 4).returning(customer());
    Expression::return_of(read)
}

fn customer_reference() -> MethodReference {
    MethodReference::new(Expression::static_type(order()), "getCustomer", 4)
        .returning(customer())
        .resolving(order())
        .taking(1)
        .fed_by_elements()
}

fn grouping_by_customer() -> Call {
    call(Expression::static_type(class(COLLECTORS)), "groupingBy", 4)
        .with(customer_reference())
        .returning(collector_of(order(), map_of(customer(), list_of(order()))))
        .returning_from(&[])
        .keyed_by(0)
}

#[test]
fn a_lambda_given_to_a_stream_is_fed_by_its_elements_and_its_result_flows_to_the_return() {
    let mapped = mapped("orders.stream().map(order -> order.getCustomer())");

    let each = Lambda::new(&["order"])
        .resolving("order", order())
        .fed_by_elements("order")
        .yielding(customer_of_order());
    let map = call(streamed_orders(), "map", 4)
        .with(each)
        .returning(stream_of(customer()))
        .returning_from(&[0]);
    assert_eq!(mapped, f().call(map));
}

#[test]
fn a_lambda_that_receives_a_collected_result_is_fed_by_its_receiver() {
    let mapped = mapped(
        "orders.stream().collect(Collectors.collectingAndThen(Collectors.toList(), list -> list))",
    );

    let finisher = Lambda::new(&["list"])
        .resolving("list", list_of(order()))
        .fed_by_receiver("list")
        .yielding(Expression::local("list"));
    let to_list = call(Expression::static_type(class(COLLECTORS)), "toList", 4)
        .returning(collector_of(order(), list_of(order())));
    let finishing = call(
        Expression::static_type(class(COLLECTORS)),
        "collectingAndThen",
        4,
    )
    .with(Expression::return_of(to_list))
    .with(finisher)
    .returning(collector_of(order(), list_of(order())))
    .returning_from(&[1]);
    let collect = call(streamed_orders(), "collect", 4)
        .with(Expression::return_of(finishing))
        .returning(list_of(order()));
    assert_eq!(mapped, f().call(collect));
}

#[test]
fn a_lambda_in_a_nested_call_is_fed_by_the_elements_of_the_outer_stream() {
    let mapped =
        mapped("orders.stream().collect(Collectors.groupingBy(order -> order.getCustomer()))");

    let classifier = Lambda::new(&["order"])
        .resolving("order", order())
        .fed_by_elements("order")
        .yielding(customer_of_order());
    let grouping = call(Expression::static_type(class(COLLECTORS)), "groupingBy", 4)
        .with(classifier)
        .returning(collector_of(order(), map_of(customer(), list_of(order()))))
        .returning_from(&[])
        .keyed_by(0);
    let collect = call(streamed_orders(), "collect", 4)
        .with(Expression::return_of(grouping))
        .returning(map_of(customer(), list_of(order())))
        .keyed_by(0)
        .valued_by(0);
    assert_eq!(mapped, f().call(collect));
}

#[test]
fn a_classifier_in_a_nested_call_is_fed_by_the_outer_elements_and_keys_the_result() {
    let mapped = mapped("orders.stream().collect(Collectors.groupingBy(Order::getCustomer))");

    let collect = call(streamed_orders(), "collect", 4)
        .with(Expression::return_of(grouping_by_customer()))
        .returning(map_of(customer(), list_of(order())))
        .keyed_by(0)
        .valued_by(0);
    assert_eq!(mapped, f().call(collect));
}

#[test]
fn a_call_returning_the_first_type_argument_of_a_map_yields_its_keys() {
    let mapped = mapped("grouped.keySet()");

    let keys = call(
        parameter("grouped", map_of(customer(), list_of(order()))),
        "keySet",
        4,
    )
    .returning(generic(SET, &[customer()]))
    .yielding_keys();
    assert_eq!(mapped, f().call(keys));
}

fn customer_to_order() -> Call {
    let order_itself = Lambda::new(&["order"])
        .resolving("order", order())
        .fed_by_elements("order")
        .yielding(Expression::local("order"));
    call(Expression::static_type(class(COLLECTORS)), "toMap", 4)
        .with(customer_reference())
        .with(order_itself)
        .returning(collector_of(order(), map_of(customer(), order())))
        .returning_from(&[1])
        .keyed_by(0)
        .valued_by(1)
}

#[test]
fn a_value_mapper_in_a_nested_call_values_the_result_beside_the_key_mapper_keying_it() {
    let mapped =
        mapped("orders.stream().collect(Collectors.toMap(Order::getCustomer, order -> order))");

    let collect = call(streamed_orders(), "collect", 4)
        .with(Expression::return_of(customer_to_order()))
        .returning(map_of(customer(), order()))
        .keyed_by(0)
        .valued_by(0);
    assert_eq!(mapped, f().call(collect));
}

fn by_customer() -> Expression {
    parameter("byCustomer", map_of(customer(), order()))
}

#[test]
fn a_call_returning_the_second_type_argument_of_a_map_yields_its_values() {
    let mapped = mapped("byCustomer.values()");

    let values = call(by_customer(), "values", 4)
        .returning(generic(COLLECTION, &[order()]))
        .yielding_values();
    assert_eq!(mapped, f().call(values));
}

#[test]
fn a_call_returning_a_container_of_the_entries_of_a_map_keeps_the_entries() {
    let mapped = mapped("byCustomer.entrySet()");

    let entries = call(by_customer(), "entrySet", 4)
        .returning(generic(SET, &[entry_of(customer(), order())]))
        .keeping_entries();
    assert_eq!(mapped, f().call(entries));
}

#[test]
fn a_call_returning_a_container_of_the_same_elements_keeps_the_entries() {
    let mapped = mapped("byCustomer.entrySet().stream()");

    let entries = call(by_customer(), "entrySet", 4)
        .returning(generic(SET, &[entry_of(customer(), order())]))
        .keeping_entries();
    let stream = call(Expression::return_of(entries), "stream", 4)
        .returning(stream_of(entry_of(customer(), order())))
        .keeping_entries();
    assert_eq!(mapped, f().call(stream));
}

#[test]
fn a_call_returning_the_first_type_argument_of_an_entry_yields_its_key() {
    let mapped = mapped("entry.getKey()");

    let key = call(
        parameter("entry", entry_of(customer(), order())),
        "getKey",
        4,
    )
    .returning(customer())
    .yielding_keys();
    assert_eq!(mapped, f().call(key));
}

#[test]
fn a_call_returning_the_second_type_argument_of_an_entry_yields_its_value() {
    let mapped = mapped("entry.getValue()");

    let value = call(
        parameter("entry", entry_of(customer(), order())),
        "getValue",
        4,
    )
    .returning(order())
    .yielding_values();
    assert_eq!(mapped, f().call(value));
}

#[test]
fn a_call_returning_an_element_of_the_receiver_keeps_the_entries() {
    let mapped = mapped("orders.get(0)");

    let first = call(parameter("orders", list_of(order())), "get", 4)
        .with(Expression::literal())
        .returning(order())
        .keeping_entries();
    assert_eq!(mapped, f().call(first));
}
