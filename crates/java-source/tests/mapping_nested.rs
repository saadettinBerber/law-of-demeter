mod mapped;

use law_of_demeter_core::domain::method::{Expression, Method};
use mapped::{call, class, mapped_methods, method_in, parameter};

#[test]
fn an_anonymous_class_method_sees_captured_variables_and_outer_members() {
    let source = "class OrderService {\n Notifier notifier;\n void helper() {}\n void f(Order order) {\n  Runnable r = new Runnable() {\n   public void run() {\n    order.getCustomer().getWallet();\n    notifier.send();\n    helper();\n   }\n  };\n }\n}";

    let mapped = method_in(source, "OrderService$1", "run");

    assert_eq!(
        mapped,
        Method::new("OrderService$1", "run", 6)
            .declared_within("OrderService", "f")
            .call(call(
                Expression::return_of(call(
                    Expression::field("order", class("Order")),
                    "getCustomer",
                    7
                )),
                "getWallet",
                7
            ))
            .call(call(
                Expression::field("notifier", class("Notifier")),
                "send",
                8
            ))
            .call(call(Expression::This, "helper", 9))
    );
}

#[test]
fn a_call_to_a_method_of_an_enclosing_class_goes_to_this() {
    let source = "class OrderService {\n void helper() {}\n class Inner {\n  void g(Order order) {\n   helper();\n   order.go();\n  }\n }\n}";

    let mapped = method_in(source, "OrderService$Inner", "g");

    assert_eq!(
        mapped,
        Method::new("OrderService$Inner", "g", 4)
            .call(call(Expression::This, "helper", 5))
            .call(call(parameter("order", class("Order")), "go", 6))
    );
}

#[test]
fn field_initializers_become_the_synthetic_initializer_methods_expecting_the_field_type() {
    let source = "class OrderService {\n static Order first = Order.latest();\n Order order = Order.latest();\n Wallet wallet = order.getCustomer().getWallet();\n}";

    let methods = mapped_methods(source);

    let named = |name: &str| methods.iter().find(|method| method.name() == name).unwrap();
    assert_eq!(
        named("<static-init>"),
        &Method::new("OrderService", "<static-init>", 2).call(call(
            Expression::static_type(class("Order")),
            "latest",
            2
        ))
    );
    assert_eq!(
        named("<instance-init>"),
        &Method::new("OrderService", "<instance-init>", 3)
            .call(call(Expression::static_type(class("Order")), "latest", 3))
            .call(call(
                Expression::return_of(call(
                    Expression::field("order", class("Order")),
                    "getCustomer",
                    4
                )),
                "getWallet",
                4
            ))
    );
}

#[test]
fn constructors_are_named_init_and_keep_their_calls() {
    let source = "class OrderService {\n OrderService(Order order) {\n  order.go();\n }\n}";

    let mapped = method_in(source, "OrderService", "<init>");

    assert_eq!(
        mapped,
        Method::new("OrderService", "<init>", 2).call(call(
            parameter("order", class("Order")),
            "go",
            3
        ))
    );
}
