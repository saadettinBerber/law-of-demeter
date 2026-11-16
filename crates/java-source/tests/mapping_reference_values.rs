mod mapped;

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::{Expression, Method, MethodReference};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};
use mapped::{class, method_f, method_f_in, method_in};

fn f(line: usize) -> Method {
    Method::new("OrderService", "f", line)
}

fn function_of_order() -> TypeRef {
    TypeRef::generic("Function", vec![class("Order"), class("Customer")])
}

fn getting_the_customer() -> MethodReference {
    MethodReference::new(Expression::static_type(class("Order")), "getCustomer", 3)
}

#[test]
fn a_reference_assigned_to_a_declared_local_stays_a_value() {
    let source = "class OrderService {\n void f() {\n  Function<Order, Customer> get = Order::getCustomer;\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).assign(
            "get",
            function_of_order(),
            Expression::method_reference(getting_the_customer())
        )
    );
}

#[test]
fn a_returned_reference_expects_the_return_type_of_the_method() {
    let source = "class OrderService {\n Function<Order, Customer> f() {\n  return Order::getCustomer;\n }\n}";

    let mapped = method_f(source);

    let reference = getting_the_customer().expecting(function_of_order());
    assert_eq!(
        mapped,
        f(2).evaluate(Expression::method_reference(reference))
    );
}

#[test]
fn a_reference_on_this_is_returned_as_a_value() {
    let source =
        "class OrderService {\n Function<Order, Customer> f() {\n  return this::handle;\n }\n}";

    let mapped = method_f(source);

    let reference =
        MethodReference::new(Expression::This, "handle", 3).expecting(function_of_order());
    assert_eq!(
        mapped,
        f(2).evaluate(Expression::method_reference(reference))
    );
}

#[test]
fn a_reference_assigned_to_a_field_expects_the_type_of_the_field() {
    let source = "class OrderService {\n Function<Order, Customer> get;\n void f() {\n  get = Order::getCustomer;\n }\n}";

    let mapped = method_in(source, "OrderService", "f");

    let reference = MethodReference::new(Expression::static_type(class("Order")), "getCustomer", 4)
        .expecting(function_of_order());
    assert_eq!(
        mapped,
        Method::new("OrderService", "f", 3).evaluate(Expression::method_reference(reference))
    );
}

#[test]
fn a_cast_reference_keeps_the_cast_type_around_the_value() {
    let source = "class OrderService {\n void f() {\n  Object get = (Function<Order, Customer>) Order::getCustomer;\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).assign(
            "get",
            class("java.lang.Object"),
            Expression::cast(
                function_of_order(),
                Expression::method_reference(getting_the_customer())
            )
        )
    );
}

#[test]
fn a_reference_on_a_parameter_keeps_its_target_as_a_value() {
    let source = "class OrderService {\n void f(Customer customer) {\n  Consumer<String> rename = customer::rename;\n }\n}";

    let mapped = method_f(source);

    let reference = MethodReference::new(
        mapped::parameter("customer", class("Customer")),
        "rename",
        3,
    );
    assert_eq!(
        mapped,
        f(2).assign(
            "rename",
            TypeRef::generic("Consumer", vec![class("java.lang.String")]),
            Expression::method_reference(reference)
        )
    );
}

fn shape(name: &str) -> TypeShape {
    TypeShape::new(name, TypeKind::Class)
}

fn functional(name: &str, method: MethodShape) -> TypeShape {
    TypeShape::new(name, TypeKind::Interface)
        .with_type_parameter("T")
        .with_type_parameter("R")
        .with_method(method.as_abstract())
}

fn function_world() -> Types {
    let apply = MethodShape::new("apply")
        .with_parameters(vec![TypeRef::variable("T")])
        .returning(TypeRef::variable("R"));
    Types::new()
        .with_shape(functional("Function", apply))
        .with_shape(
            shape("Order")
                .with_method(MethodShape::new("getCustomer").returning(class("Customer"))),
        )
        .with_shape(shape("Customer").with_method(MethodShape::new("rename")))
        .with_shape(
            shape("OrderService").with_method(
                MethodShape::new("handle")
                    .with_parameters(vec![class("Order")])
                    .returning(class("Customer")),
            ),
        )
}

fn resolved_getting_the_customer(line: usize) -> MethodReference {
    MethodReference::new(Expression::static_type(class("Order")), "getCustomer", line)
        .taking(1)
        .returning(class("Customer"))
        .resolving(class("Order"))
}

#[test]
fn a_reference_assigned_to_a_declared_local_resolves_against_the_declared_function() {
    let source = "class OrderService {\n void f() {\n  Function<Order, Customer> get = Order::getCustomer;\n }\n}";

    let mapped = method_f_in(source, &function_world());

    assert_eq!(
        mapped,
        f(2).assign(
            "get",
            function_of_order(),
            Expression::method_reference(resolved_getting_the_customer(3))
        )
    );
}

#[test]
fn a_returned_reference_resolves_against_the_return_type_of_the_method() {
    let source = "class OrderService {\n Function<Order, Customer> f() {\n  return Order::getCustomer;\n }\n}";

    let mapped = method_f_in(source, &function_world());

    let reference = resolved_getting_the_customer(3).expecting(function_of_order());
    assert_eq!(
        mapped,
        f(2).evaluate(Expression::method_reference(reference))
    );
}

#[test]
fn a_reference_on_this_resolves_against_the_return_type_of_the_method() {
    let source =
        "class OrderService {\n Function<Order, Customer> f() {\n  return this::handle;\n }\n}";

    let mapped = method_f_in(source, &function_world());

    let reference = MethodReference::new(Expression::This, "handle", 3)
        .taking(1)
        .returning(class("Customer"))
        .expecting(function_of_order());
    assert_eq!(
        mapped,
        f(2).evaluate(Expression::method_reference(reference))
    );
}

#[test]
fn a_reference_assigned_to_a_field_resolves_against_the_type_of_the_field() {
    let source = "class OrderService {\n Function<Order, Customer> get;\n void f() {\n  get = Order::getCustomer;\n }\n}";

    let mapped = mapped::mapped_methods_in(source, &function_world())
        .into_iter()
        .find(|method| method.name() == "f")
        .unwrap();

    let reference = resolved_getting_the_customer(4).expecting(function_of_order());
    assert_eq!(
        mapped,
        Method::new("OrderService", "f", 3).evaluate(Expression::method_reference(reference))
    );
}

#[test]
fn a_cast_reference_resolves_against_the_cast_type() {
    let source = "class OrderService {\n void f() {\n  Object get = (Function<Order, Customer>) Order::getCustomer;\n }\n}";

    let mapped = method_f_in(source, &function_world());

    assert_eq!(
        mapped,
        f(2).assign(
            "get",
            class("Object"),
            Expression::cast(
                function_of_order(),
                Expression::method_reference(resolved_getting_the_customer(3))
            )
        )
    );
}
