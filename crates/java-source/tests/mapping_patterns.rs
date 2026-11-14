mod mapped;

use law_of_demeter_core::domain::method::{Expression, Method};
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

fn object() -> TypeRef {
    class("java.lang.Object")
}

fn subject() -> Expression {
    Expression::local("<subject>")
}

#[test]
fn an_instanceof_binding_is_a_local_typed_by_the_pattern_with_the_operands_origin() {
    let source = "class OrderService {\n void f(Order order) {\n  if (order.getOwner() instanceof Customer c) {\n   c.getWallet().pay();\n  }\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).assign_var("<subject>", get("getOwner", 3))
            .assign(
                "c",
                class("Customer"),
                Expression::cast(class("Customer"), subject())
            )
            .call(call(
                Expression::return_of(call(Expression::local("c"), "getWallet", 4)),
                "pay",
                4
            ))
    );
}

#[test]
fn an_instanceof_binding_of_a_quiet_operand_needs_no_helper_local() {
    let source = "class OrderService {\n void f(Object o) {\n  if (o instanceof Customer c) {\n   c.go();\n  }\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).assign(
            "c",
            class("Customer"),
            Expression::cast(class("Customer"), parameter("o", object()))
        )
        .call(call(Expression::local("c"), "go", 4))
    );
}

#[test]
fn an_instanceof_without_a_binding_judges_the_calls_in_its_operand() {
    let source = "class OrderService {\n void f(Order order) {\n  if (order.getOwner() instanceof Customer) {\n   order.go();\n  }\n  boolean same = order instanceof Order;\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).evaluate(Expression::operation(vec![get("getOwner", 3)]))
            .call(call(order(), "go", 4))
            .assign(
                "same",
                TypeRef::primitive("boolean"),
                Expression::operation(vec![order()])
            )
    );
}

#[test]
fn record_pattern_components_are_typed_locals_with_the_operands_origin() {
    let source = "class OrderService {\n void f(Object o) {\n  if (o instanceof Order(Customer a, var b)) {\n   a.go();\n   b.go();\n  }\n }\n}";

    let mapped = method_f(source);

    let o = || parameter("o", object());
    assert_eq!(
        mapped,
        f(2).assign(
            "a",
            class("Customer"),
            Expression::cast(class("Customer"), o())
        )
        .assign("b", object(), Expression::cast(object(), o()))
        .call(call(Expression::local("a"), "go", 4))
        .call(call(Expression::local("b"), "go", 5))
    );
}

#[test]
fn a_switch_judges_its_selector_guards_and_every_branch_in_order() {
    let source = "class OrderService {\n void f(Order order) {\n  switch (order.getStatus()) {\n   case 1 -> order.paid();\n   case 2 -> { order.shipped(); }\n   default -> order.other();\n  }\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).call(call(order(), "getStatus", 3))
            .call(call(order(), "paid", 4))
            .call(call(order(), "shipped", 5))
            .call(call(order(), "other", 6))
    );
}

#[test]
fn a_switch_expression_yields_a_literal_after_its_branches_ran() {
    let source = "class OrderService {\n void f(Order order) {\n  int x = switch (order.kind()) {\n   case 1 -> { yield order.a(); }\n   default -> order.b();\n  };\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).call(call(order(), "kind", 3))
            .call(call(order(), "a", 4))
            .call(call(order(), "b", 5))
            .assign("x", TypeRef::primitive("int"), Expression::literal())
    );
}

#[test]
fn switch_patterns_bind_typed_locals_from_the_selector_and_the_guard_sees_them() {
    let source = "class OrderService {\n void f(Order order) {\n  switch (order.getOwner()) {\n   case Customer c when c.ok() -> c.go();\n   case Order(var a, Wallet w) -> w.pay();\n   default -> order.other();\n  }\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).assign_var("<subject>", get("getOwner", 3))
            .assign(
                "c",
                class("Customer"),
                Expression::cast(class("Customer"), subject())
            )
            .call(call(Expression::local("c"), "ok", 4))
            .call(call(Expression::local("c"), "go", 4))
            .assign("a", object(), Expression::cast(object(), subject()))
            .assign(
                "w",
                class("Wallet"),
                Expression::cast(class("Wallet"), subject())
            )
            .call(call(Expression::local("w"), "pay", 5))
            .call(call(order(), "other", 6))
    );
}

#[test]
fn a_switch_expression_with_patterns_yields_a_literal_not_its_selector() {
    let source = "class OrderService {\n void f(Order order) {\n  Customer c = switch (order.getOwner()) {\n   case Customer x -> x;\n   default -> null;\n  };\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).assign_var("<subject>", get("getOwner", 3))
            .assign(
                "x",
                class("Customer"),
                Expression::cast(class("Customer"), subject())
            )
            .assign("c", class("Customer"), Expression::literal())
    );
}

#[test]
fn an_old_style_switch_with_colon_groups_runs_its_branches_in_source_order() {
    let source = "class OrderService {\n void f(Order order) {\n  switch (order.kind()) {\n   case 1:\n    order.paid();\n    break;\n   case 2:\n   case 3:\n    order.shipped();\n    break;\n   default:\n    order.other();\n  }\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).call(call(order(), "kind", 3))
            .call(call(order(), "paid", 5))
            .call(call(order(), "shipped", 9))
            .call(call(order(), "other", 12))
    );
}
