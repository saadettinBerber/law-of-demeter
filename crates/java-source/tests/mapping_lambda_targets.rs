mod mapped;

use law_of_demeter_core::domain::method::{Creation, Expression, Lambda, Method};
use law_of_demeter_core::domain::type_shape::TypeRef;
use mapped::{call, class, method_f, method_in};

fn f(line: usize) -> Method {
    Method::new("OrderService", "f", line)
}

fn consumer_of_customer() -> TypeRef {
    TypeRef::generic("Consumer", vec![class("Customer")])
}

fn paying(parameter: &str, line: usize) -> Expression {
    let wallet = call(Expression::local(parameter), "getWallet", line);
    Expression::return_of(call(Expression::return_of(wallet), "pay", line))
}

fn payer() -> Lambda {
    Lambda::new(&["c"]).yielding(paying("c", 3))
}

#[test]
fn a_lambda_assigned_to_a_declared_local_stays_a_value_with_its_parameters_and_body() {
    let source = "class OrderService {\n void f() {\n  Consumer<Customer> run = c -> c.getWallet().pay();\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).assign("run", consumer_of_customer(), Expression::lambda(payer()))
    );
}

#[test]
fn a_lambda_assigned_to_a_var_local_stays_a_value_without_a_declared_type() {
    let source = "class OrderService {\n void f() {\n  var run = c -> c.getWallet().pay();\n }\n}";

    let mapped = method_f(source);

    assert_eq!(mapped, f(2).assign_var("run", Expression::lambda(payer())));
}

#[test]
fn a_returned_lambda_expects_the_return_type_of_the_method() {
    let source = "class OrderService {\n Consumer<Customer> f() {\n  return c -> c.getWallet().pay();\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).evaluate(Expression::lambda(
            payer().expecting(consumer_of_customer())
        ))
    );
}

#[test]
fn a_lambda_assigned_to_a_field_expects_the_type_of_the_field() {
    let source = "class OrderService {\n Consumer<Customer> run;\n void f() {\n  run = c -> c.getWallet().pay();\n }\n}";

    let mapped = method_f(source);

    let lambda = Lambda::new(&["c"]).yielding(paying("c", 4));
    assert_eq!(
        mapped,
        Method::new("OrderService", "f", 3)
            .evaluate(Expression::lambda(lambda.expecting(consumer_of_customer())))
    );
}

#[test]
fn a_cast_lambda_keeps_the_cast_type_around_the_value() {
    let source = "class OrderService {\n void f() {\n  Object run = (Consumer<Customer>) c -> c.getWallet().pay();\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).assign(
            "run",
            class("java.lang.Object"),
            Expression::cast(consumer_of_customer(), Expression::lambda(payer()))
        )
    );
}

#[test]
fn an_explicitly_typed_lambda_parameter_keeps_its_declared_type_as_a_value() {
    let source = "class OrderService {\n void f() {\n  var run = (Customer c) -> c.getWallet().pay();\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).assign_var(
            "run",
            Expression::lambda(payer().declaring("c", class("Customer")))
        )
    );
}

#[test]
fn an_explicitly_typed_lambda_parameter_keeps_its_declared_type_as_an_argument() {
    let source = "class OrderService {\n void f(Order order) {\n  order.lines().forEach((Customer c) -> c.getWallet().pay());\n }\n}";

    let mapped = method_f(source);

    let lines = Expression::return_of(call(mapped::parameter("order", class("Order")), "lines", 3));
    assert_eq!(
        mapped,
        f(2).call(call(lines, "forEach", 3).with(payer().declaring("c", class("Customer"))))
    );
}

#[test]
fn an_explicitly_typed_lambda_parameter_keeps_its_declared_type_in_a_constructor_argument() {
    let source = "class OrderService {\n void f() {\n  new Order((Customer c) -> c.getWallet().pay());\n }\n}";

    let mapped = method_f(source);

    let creation = Creation::new(class("Order")).with(payer().declaring("c", class("Customer")));
    assert_eq!(mapped, f(2).evaluate(Expression::created(creation)));
}

#[test]
fn a_constructor_argument_lambda_without_declared_types_stays_untyped() {
    let source =
        "class OrderService {\n OrderService() {\n  new Order(c -> c.getWallet().pay());\n }\n}";

    let mapped = method_in(source, "OrderService", "<init>");

    let creation = Creation::new(class("Order")).with(payer());
    assert_eq!(
        mapped,
        Method::new("OrderService", "<init>", 2).evaluate(Expression::created(creation))
    );
}
