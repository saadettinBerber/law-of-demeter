mod mapped;

use law_of_demeter_core::domain::method::{Creation, Expression, Lambda, Method};
use law_of_demeter_core::domain::type_shape::TypeRef;
use mapped::{call, class, method_f, parameter};

fn f(line: usize) -> Method {
    Method::new("OrderService", "f", line)
}

fn order() -> Expression {
    parameter("order", class("Order"))
}

fn get_customer(line: usize) -> Expression {
    Expression::return_of(call(order(), "getCustomer", line))
}

#[test]
fn a_local_with_a_declared_type_is_assigned_with_that_type_and_used_by_name() {
    let source = "class OrderService {\n void f(Order order, int amount) {\n  Customer customer = order.getCustomer();\n  customer.getWallet().withdraw(amount);\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).assign("customer", class("Customer"), get_customer(3))
            .call(
                call(Expression::local("customer"), "getWallet", 4)
                    .then("withdraw", 4)
                    .with(parameter("amount", TypeRef::primitive("int")))
            )
    );
}

#[test]
fn a_var_local_is_assigned_without_a_declared_type() {
    let source = "class OrderService {\n void f(Order order) {\n  var customer = order.getCustomer();\n }\n}";

    let mapped = method_f(source);

    assert_eq!(mapped, f(2).assign_var("customer", get_customer(3)));
}

#[test]
fn a_local_without_a_value_makes_no_step() {
    let mapped =
        method_f("class OrderService {\n void f() {\n  Customer customer;\n  int a, b;\n }\n}");

    assert_eq!(mapped, f(2));
}

#[test]
fn a_return_judges_its_value_and_a_bare_return_does_nothing() {
    let source = "class OrderService {\n Status f(Order order, boolean done) {\n  if (done) return null;\n  return order.getStatus();\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).call(call(order(), "getStatus", 4).expecting(class("Status")))
    );
}

#[test]
fn a_returned_call_expects_the_declared_return_type_from_its_outermost_call() {
    let source = "import java.util.List;\nclass OrderService<T> {\n List<Order> f(Order order) {\n  return order.getOwner().find();\n }\n int g(Order order) {\n  return order.count();\n }\n T h(Order order) {\n  return order.item();\n }\n}";

    let mapped = mapped::mapped_methods(source);

    let owner = Expression::return_of(call(order(), "getOwner", 4));
    let named = |name: &str| mapped.iter().find(|method| method.name() == name).unwrap();
    assert_eq!(
        named("f"),
        &Method::new("OrderService", "f", 3).call(
            call(owner, "find", 4)
                .expecting(TypeRef::generic("java.util.List", vec![class("Order")]))
        )
    );
    assert_eq!(
        named("g"),
        &Method::new("OrderService", "g", 6)
            .call(call(order(), "count", 7).expecting(TypeRef::primitive("int")))
    );
    assert_eq!(
        named("h"),
        &Method::new("OrderService", "h", 9)
            .call(call(order(), "item", 10).expecting(TypeRef::variable("T")))
    );
}

#[test]
fn a_returned_value_that_is_not_a_call_expects_nothing() {
    let source = "class OrderService {\n Order f(Order order) {\n  return order;\n }\n}";

    let mapped = method_f(source);

    assert_eq!(mapped, f(2));
}

#[test]
fn a_return_inside_a_lambda_does_not_take_the_return_type_of_the_method() {
    let source = "class OrderService {\n Order f(Order order) {\n  order.lines().map(line -> {\n   return line.price();\n  });\n  Supplier<Wallet> supplier = () -> {\n   return order.wallet();\n  };\n  return order.next();\n }\n}";

    let mapped = method_f(source);

    let lines = Expression::return_of(call(order(), "lines", 3));
    assert_eq!(
        mapped,
        f(2).call(call(lines, "map", 3).with(Lambda::new(&["line"]).yielding(
            Expression::return_of(call(Expression::local("line"), "price", 4))
        )))
        .call(call(order(), "wallet", 7))
        .assign(
            "supplier",
            TypeRef::generic("Supplier", vec![class("Wallet")]),
            Expression::literal()
        )
        .call(call(order(), "next", 9).expecting(class("Order")))
    );
}

#[test]
fn an_if_judges_its_condition_then_both_branches_in_order() {
    let source = "class OrderService {\n void f(Order a, Order b, Order c) {\n  if (a.ok()) {\n   b.go();\n  } else {\n   c.go();\n  }\n }\n}";

    let mapped = method_f(source);

    let order = |name: &str| parameter(name, class("Order"));
    assert_eq!(
        mapped,
        f(2).call(call(order("a"), "ok", 3))
            .call(call(order("b"), "go", 4))
            .call(call(order("c"), "go", 6))
    );
}

#[test]
fn a_for_loop_runs_init_condition_body_then_update() {
    let source = "class OrderService {\n void f(Order order) {\n  for (int i = 0; order.more(i); order.next()) {\n   order.go();\n  }\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).assign("i", TypeRef::primitive("int"), Expression::literal())
            .call(call(order(), "more", 3).with(Expression::local("i")))
            .call(call(order(), "go", 4))
            .call(call(order(), "next", 3))
    );
}

#[test]
fn an_enhanced_for_maps_to_for_each_and_the_body_uses_the_variable() {
    let source = "class OrderService {\n void f(Order order) {\n  for (OrderLine line : order.getLines()) {\n   line.applyDiscount();\n  }\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).for_each("line", Expression::return_of(call(order(), "getLines", 3)))
            .call(call(Expression::local("line"), "applyDiscount", 4))
    );
}

#[test]
fn while_and_do_while_judge_their_condition_and_body() {
    let source = "class OrderService {\n void f(Order a) {\n  while (a.ok()) {\n   a.go();\n  }\n  do {\n   a.stop();\n  } while (a.more());\n }\n}";

    let mapped = method_f(source);

    let a = || parameter("a", class("Order"));
    assert_eq!(
        mapped,
        f(2).call(call(a(), "ok", 3))
            .call(call(a(), "go", 4))
            .call(call(a(), "stop", 7))
            .call(call(a(), "more", 8))
    );
}

#[test]
fn try_judges_resources_body_catches_and_finally_with_typed_caught_values() {
    let source = "class OrderService {\n void f(Order a) {\n  try (Customer r = a.open()) {\n   r.use();\n  } catch (java.io.IOException | IllegalStateException e) {\n   e.print();\n  } catch (IllegalStateException x) {\n   x.print();\n  } finally {\n   a.close();\n  }\n }\n}";

    let mapped = method_f(source);

    let a = || parameter("a", class("Order"));
    let throwable = class("java.lang.Throwable");
    let illegal = class("java.lang.IllegalStateException");
    assert_eq!(
        mapped,
        f(2).assign(
            "r",
            class("Customer"),
            Expression::return_of(call(a(), "open", 3))
        )
        .call(call(Expression::local("r"), "use", 4))
        .assign("e", throwable.clone(), parameter("e", throwable))
        .call(call(Expression::local("e"), "print", 6))
        .assign("x", illegal.clone(), parameter("x", illegal))
        .call(call(Expression::local("x"), "print", 8))
        .call(call(a(), "close", 10))
    );
}

#[test]
fn blocks_synchronized_and_labeled_statements_are_flattened_in_order() {
    let source = "class OrderService {\n void f(Order a) {\n  {\n   a.one();\n  }\n  synchronized (a.lock()) {\n   a.two();\n  }\n  outer: while (a.ok()) { a.three(); }\n }\n}";

    let mapped = method_f(source);

    let a = || parameter("a", class("Order"));
    assert_eq!(
        mapped,
        f(2).call(call(a(), "one", 4))
            .call(call(a(), "lock", 6))
            .call(call(a(), "two", 7))
            .call(call(a(), "ok", 9))
            .call(call(a(), "three", 9))
    );
}

#[test]
fn throwing_a_created_object_judges_the_calls_in_its_arguments() {
    let source = "class OrderService {\n void f(Order order) {\n  throw new IllegalStateException(order.describe());\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).evaluate(Expression::created(
            Creation::new(class("java.lang.IllegalStateException"))
                .with(Expression::return_of(call(order(), "describe", 3)))
        ))
    );
}
