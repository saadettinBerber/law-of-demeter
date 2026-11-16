mod mapped;

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::{Expression, Method, MethodReference};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};
use mapped::{call, class, method_f_in, parameter};

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

fn stream_of(element: TypeRef) -> TypeRef {
    generic("Stream", &[element])
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

fn world() -> Types {
    let mapping = MethodShape::new("map")
        .with_type_parameter("R")
        .with_parameters(vec![generic("Function", &[variable("T"), variable("R")])])
        .returning(stream_of(variable("R")));
    let applying = MethodShape::new("apply")
        .with_parameters(vec![variable("T")])
        .returning(variable("R"))
        .as_abstract();
    let supplying = MethodShape::new("get")
        .returning(variable("T"))
        .as_abstract();
    let picking = MethodShape::new("pick")
        .as_static()
        .with_type_parameter("T")
        .with_parameters(vec![variable("T"), generic("Supplier", &[variable("T")])])
        .returning(variable("T"));
    let customers = shape("Customers")
        .with_method(
            MethodShape::new("of")
                .as_static()
                .with_parameters(vec![order()])
                .returning(customer()),
        )
        .with_method(
            MethodShape::new("wallet")
                .as_static()
                .with_parameters(vec![customer()])
                .returning(wallet()),
        );
    Types::new()
        .with_shape(interface("Stream", &["T"]).with_method(mapping))
        .with_shape(interface("Function", &["T", "R"]).with_method(applying))
        .with_shape(interface("Supplier", &["T"]).with_method(supplying))
        .with_shape(
            interface("List", &["E"])
                .with_method(MethodShape::new("stream").returning(stream_of(variable("E")))),
        )
        .with_shape(
            shape("Order")
                .with_method(MethodShape::new("getCustomer").returning(customer()))
                .with_method(MethodShape::new("count").returning(TypeRef::primitive("int")))
                .with_method(MethodShape::new("touch"))
                .with_method(
                    MethodShape::new("walletOf")
                        .with_parameters(vec![customer()])
                        .returning(wallet()),
                )
                .with_method(
                    MethodShape::new("lookup")
                        .as_static()
                        .with_parameters(vec![order()])
                        .returning(customer()),
                )
                .with_method(MethodShape::new("lookup").returning(wallet())),
        )
        .with_shape(
            shape("Customer").with_method(MethodShape::new("getWallet").returning(wallet())),
        )
        .with_shape(shape("Wallet"))
        .with_shape(
            interface("Optional", &["T"])
                .with_method(MethodShape::new("get").returning(variable("T"))),
        )
        .with_shape(
            shape("Taker")
                .with_method(
                    MethodShape::new("take")
                        .with_parameters(vec![customer()])
                        .returning(wallet()),
                )
                .with_method(
                    MethodShape::new("take")
                        .with_parameters(vec![generic("Function", &[order(), customer()])])
                        .returning(customer()),
                ),
        )
        .with_shape(
            shape("Runner")
                .with_method(
                    MethodShape::new("run")
                        .with_type_parameter("T")
                        .with_parameters(vec![variable("T")])
                        .returning(wallet()),
                )
                .with_method(
                    MethodShape::new("run")
                        .with_parameters(vec![generic("Function", &[order(), customer()])])
                        .returning(customer()),
                ),
        )
        .with_shape(customers)
        .with_shape(
            shape("OrderService").with_method(
                MethodShape::new("handle")
                    .with_parameters(vec![order()])
                    .returning(customer()),
            ),
        )
        .with_shape(shape("Util").with_method(picking))
}

fn f(line: usize) -> Method {
    Method::new("OrderService", "f", line)
}

fn mapped_in(parameters: &str, body: &str) -> Method {
    let source = format!("class OrderService {{\n void f({parameters}) {{\n  {body}\n }}\n}}");

    method_f_in(&source, &world())
}

fn orders() -> Expression {
    parameter("orders", generic("List", &[order()]))
}

fn streamed() -> Expression {
    let stream = call(orders(), "stream", 3).returning(stream_of(order()));
    Expression::return_of(stream)
}

fn mapping(reference: MethodReference, returned: Option<TypeRef>) -> Method {
    let map = call(streamed(), "map", 3)
        .with(reference.fed_by_receiver())
        .returning_from(&[0]);
    match returned {
        Some(returned) => f(2).call(map.returning(returned)),
        None => f(2).call(map),
    }
}

fn through(owner: TypeRef, name: &str) -> MethodReference {
    MethodReference::new(Expression::static_type(owner), name, 3)
}

fn mapped_map(reference_source: &str) -> Method {
    mapped_in(
        "List<Order> orders, Customer customer",
        &format!("orders.stream().map({reference_source});"),
    )
}

#[test]
fn an_unbound_reference_binds_the_result_type_variable_with_the_method_return() {
    let mapped = mapped_map("Order::getCustomer");

    let reference = through(order(), "getCustomer")
        .returning(customer())
        .resolving(order());
    assert_eq!(mapped, mapping(reference, Some(stream_of(customer()))));
}

#[test]
fn a_static_reference_binds_the_result_type_variable_with_the_method_return() {
    let mapped = mapped_map("Customers::of");

    let reference = through(class("Customers"), "of")
        .returning(customer())
        .resolving(order());
    assert_eq!(mapped, mapping(reference, Some(stream_of(customer()))));
}

#[test]
fn a_reference_on_this_binds_the_result_with_the_return_of_the_own_method() {
    let mapped = mapped_map("this::handle");

    let reference = MethodReference::new(Expression::This, "handle", 3).returning(customer());
    assert_eq!(mapped, mapping(reference, Some(stream_of(customer()))));
}

#[test]
fn a_reference_on_a_parameter_binds_the_result_with_the_return_of_its_type() {
    let mapped = mapped_in(
        "List<Customer> customers, Order order",
        "customers.stream().map(order::walletOf);",
    );

    let reference =
        MethodReference::new(parameter("order", order()), "walletOf", 3).returning(wallet());
    let customers = parameter("customers", generic("List", &[customer()]));
    let stream = call(customers, "stream", 3).returning(stream_of(customer()));
    let map = call(Expression::return_of(stream), "map", 3)
        .with(reference.fed_by_receiver())
        .returning(stream_of(wallet()))
        .returning_from(&[0]);
    assert_eq!(mapped, f(2).call(map));
}

#[test]
fn a_constructor_reference_binds_the_result_with_the_class_it_creates() {
    let mapped = mapped_map("Customer::new");

    let reference = through(customer(), "new").returning(customer());
    assert_eq!(mapped, mapping(reference, Some(stream_of(customer()))));
}

#[test]
fn a_primitive_return_binds_the_boxed_type() {
    let mapped = mapped_map("Order::count");

    let reference = through(order(), "count")
        .returning(TypeRef::primitive("int"))
        .resolving(order());
    let boxed = stream_of(class("java.lang.Integer"));
    assert_eq!(mapped, mapping(reference, Some(boxed)));
}

#[test]
fn a_reference_to_a_method_that_returns_nothing_leaves_the_result_unresolved() {
    let mapped = mapped_map("Order::touch");

    assert_eq!(
        mapped,
        mapping(through(order(), "touch").resolving(order()), None)
    );
}

#[test]
fn a_reference_to_a_method_the_type_does_not_have_leaves_the_result_unresolved() {
    let mapped = mapped_map("Order::vanish");

    assert_eq!(
        mapped,
        mapping(through(order(), "vanish").resolving(order()), None)
    );
}

#[test]
fn a_reference_on_a_type_the_catalog_does_not_know_leaves_the_result_unresolved() {
    let mapped = mapped_map("Ghost::getCustomer");

    assert_eq!(
        mapped,
        mapping(
            MethodReference::new(Expression::unresolved("Ghost"), "getCustomer", 3),
            None
        )
    );
}

#[test]
fn a_constructor_reference_to_a_generic_class_leaves_the_result_unresolved() {
    let mapped = mapped_map("Stream::new");

    assert_eq!(mapped, mapping(through(class("Stream"), "new"), None));
}

#[test]
fn a_reference_that_is_both_static_and_unbound_leaves_the_result_unresolved() {
    let mapped = mapped_map("Order::lookup");

    assert_eq!(
        mapped,
        mapping(through(order(), "lookup").resolving(order()), None)
    );
}

#[test]
fn the_result_of_a_reference_types_the_next_call_of_the_chain() {
    let mapped = mapped_map("Order::getCustomer).map(Customer::getWallet");

    let first = through(order(), "getCustomer")
        .returning(customer())
        .resolving(order())
        .fed_by_receiver();
    let first_map = call(streamed(), "map", 3)
        .with(first)
        .returning(stream_of(customer()))
        .returning_from(&[0]);
    let second = through(customer(), "getWallet")
        .returning(wallet())
        .resolving(customer())
        .fed_by_receiver();
    let second_map = call(Expression::return_of(first_map), "map", 3)
        .with(second)
        .returning(stream_of(wallet()))
        .returning_from(&[0]);
    assert_eq!(mapped, f(2).call(second_map));
}

#[test]
fn the_type_arguments_of_the_owner_come_from_the_receiver_the_stream_feeds() {
    let mapped = mapped_in(
        "List<Optional<Order>> maybes",
        "maybes.stream().map(Optional::get);",
    );

    let maybe_order = generic("Optional", &[order()]);
    let maybes = parameter(
        "maybes",
        generic("List", std::slice::from_ref(&maybe_order)),
    );
    let stream = call(maybes, "stream", 3).returning(stream_of(maybe_order.clone()));
    let reference = through(class("Optional"), "get")
        .returning(order())
        .resolving(maybe_order.clone())
        .fed_by_receiver();
    let map = call(Expression::return_of(stream), "map", 3)
        .with(reference)
        .returning(stream_of(order()))
        .returning_from(&[0]);
    assert_eq!(mapped, f(2).call(map));
}

#[test]
fn a_reference_only_fits_the_overload_that_takes_a_functional_interface() {
    let mapped = mapped_in("Taker taker", "taker.take(Order::getCustomer);");

    let reference = through(order(), "getCustomer")
        .returning(customer())
        .resolving(order())
        .fed_by_receiver();
    let take = call(parameter("taker", class("Taker")), "take", 3)
        .with(reference)
        .returning(customer())
        .returning_from(&[]);
    assert_eq!(mapped, f(2).call(take));
}

#[test]
fn a_type_variable_parameter_also_fits_the_reference_and_keeps_the_overload_ambiguous() {
    let mapped = mapped_in("Runner runner", "runner.run(Order::getCustomer);");

    let reference = through(order(), "getCustomer");
    let run = call(parameter("runner", class("Runner")), "run", 3).with(reference);
    assert_eq!(mapped, f(2).call(run));
}
