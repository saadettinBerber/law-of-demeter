mod mapped;

use law_of_demeter_core::domain::method::{Expression, Method, MethodReference};
use law_of_demeter_core::domain::type_shape::TypeRef;
use mapped::{class, method_f, method_in};

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
