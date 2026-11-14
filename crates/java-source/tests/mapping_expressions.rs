mod mapped;

use law_of_demeter_core::domain::method::{Expression, Lambda, Method};
use law_of_demeter_core::domain::type_shape::TypeRef;
use mapped::{call, class, method_f, parameter};

fn f(line: usize) -> Method {
    Method::new("OrderService", "f", line)
}

fn order() -> Expression {
    parameter("order", class("Order"))
}

fn get(name: &str, line: usize) -> Expression {
    Expression::return_of(call(order(), name, line))
}

#[test]
fn assigning_a_declared_local_keeps_its_declared_type() {
    let source = "class OrderService {\n void f(Order order) {\n  Customer c;\n  c = order.getCustomer();\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).assign("c", class("Customer"), get("getCustomer", 4))
    );
}

#[test]
fn assigning_a_var_local_again_stays_untyped() {
    let source = "class OrderService {\n void f(Order order) {\n  var c = order.getCustomer();\n  c = order.getOther();\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).assign_var("c", get("getCustomer", 3))
            .assign_var("c", get("getOther", 4))
    );
}

#[test]
fn assigning_a_field_judges_only_the_value() {
    let source = "class OrderService {\n Customer customer;\n void f(Order order) {\n  customer = order.getCustomer();\n }\n}";

    let mapped = method_f(source);

    assert_eq!(mapped, f(3).call(call(order(), "getCustomer", 4)));
}

#[test]
fn assigning_into_a_call_result_judges_the_target_chain() {
    let source = "class OrderService {\n void f(Order order) {\n  order.getCustomer().name = order.describe();\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).evaluate(Expression::field_of(get("getCustomer", 3), "name"))
            .call(call(order(), "describe", 3))
    );
}

#[test]
fn an_assignment_inside_a_condition_assigns_before_the_condition_uses_it() {
    let source = "class OrderService {\n void f(Order order) {\n  Customer c;\n  while ((c = order.next()) != null) {\n   c.go();\n  }\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).assign("c", class("Customer"), get("next", 4))
            .call(call(Expression::local("c"), "go", 5))
    );
}

#[test]
fn an_assignment_used_as_a_value_is_read_through_its_local() {
    let source = "class OrderService {\n void f(Order order) {\n  Customer c;\n  order.notify(c = order.next());\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).assign("c", class("Customer"), get("next", 4))
            .call(call(order(), "notify", 4).with(Expression::local("c")))
    );
}

#[test]
fn operations_and_ternaries_judge_the_calls_in_their_operands() {
    let source = "class OrderService {\n void f(Order order, boolean flag) {\n  int total = order.count() + 1;\n  order.a().b(flag ? order.c() : order.d());\n  total += order.e();\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).assign(
            "total",
            TypeRef::primitive("int"),
            Expression::operation(vec![get("count", 3), Expression::literal()])
        )
        .call(call(get("a", 4), "b", 4).with(Expression::operation(vec![
            parameter("flag", TypeRef::primitive("boolean")),
            get("c", 4),
            get("d", 4),
        ])))
        .evaluate(Expression::operation(vec![
            Expression::local("total"),
            get("e", 5)
        ]))
    );
}

#[test]
fn a_cast_keeps_the_operand_and_the_cast_type() {
    let source = "class OrderService {\n void f(Order order) {\n  ((Customer) order.getOwner()).getWallet();\n  ((Strange) order).go();\n  int n = (int) order.size();\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).call(call(
            Expression::cast(class("Customer"), get("getOwner", 3)),
            "getWallet",
            3
        ))
        .call(call(Expression::cast(class("Strange"), order()), "go", 4))
        .assign(
            "n",
            TypeRef::primitive("int"),
            Expression::cast(TypeRef::primitive("int"), get("size", 5))
        )
    );
}

#[test]
fn a_lambda_argument_keeps_its_parameters_and_body() {
    let source = "class OrderService {\n void f(Order order) {\n  order.lines().forEach(line -> {\n   Wallet w = line.wallet();\n   w.pay();\n   line.price().cents();\n  });\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).call(
            call(get("lines", 3), "forEach", 3).with(
                Lambda::new(&["line"])
                    .assign(
                        "w",
                        class("Wallet"),
                        Expression::return_of(call(Expression::local("line"), "wallet", 4))
                    )
                    .call(call(Expression::local("w"), "pay", 5))
                    .call(call(
                        Expression::return_of(call(Expression::local("line"), "price", 6)),
                        "cents",
                        6
                    ))
            )
        )
    );
}

#[test]
fn an_expression_lambda_is_judged_like_a_block_lambda() {
    let source = "class OrderService {\n void f(Order order) {\n  order.lines().map(line -> line.price());\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).call(
            call(get("lines", 3), "map", 3).with(Lambda::new(&["line"]).call(call(
                Expression::local("line"),
                "price",
                3
            )))
        )
    );
}

#[test]
fn a_lambda_that_is_not_an_argument_runs_inline_with_friendly_parameters() {
    let source = "class OrderService {\n void f(Order order) {\n  var run = (Customer c) -> c.getWallet().pay();\n  var plain = x -> x.go();\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).assign("c", class("Customer"), parameter("c", class("Customer")))
            .call(call(
                Expression::return_of(call(Expression::local("c"), "getWallet", 3)),
                "pay",
                3
            ))
            .assign_var("run", Expression::literal())
            .assign(
                "x",
                class("java.lang.Object"),
                parameter("x", class("java.lang.Object"))
            )
            .call(call(Expression::local("x"), "go", 4))
            .assign_var("plain", Expression::literal())
    );
}

#[test]
fn a_method_reference_is_a_literal_unless_its_target_has_calls() {
    let source = "class OrderService {\n void f(Order order) {\n  order.lines().forEach(this::go);\n  order.lines().forEach(System.out::println);\n  order.lines().forEach(Order::go);\n  order.lines().forEach(order.getCustomer()::notify);\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).call(call(get("lines", 3), "forEach", 3).with(Expression::literal()))
            .call(call(get("lines", 4), "forEach", 4).with(Expression::literal()))
            .call(call(get("lines", 5), "forEach", 5).with(Expression::literal()))
            .call(
                call(get("lines", 6), "forEach", 6)
                    .with(Expression::operation(vec![get("getCustomer", 6)]))
            )
    );
}
