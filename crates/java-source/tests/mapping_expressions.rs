mod mapped;

use law_of_demeter_core::domain::method::{Creation, Expression, Lambda, Method, MethodReference};
use law_of_demeter_core::domain::type_shape::TypeRef;
use mapped::{call, class, method_f, method_in, parameter};

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
fn assigning_a_field_expects_its_declared_type_from_the_outermost_call() {
    let source = "import java.util.List;\nclass OrderService {\n Customer customer;\n int total;\n List<Order> orders;\n void f(Order order) {\n  customer = order.getCustomer();\n  this.customer = order.getOwner().getCustomer();\n  total = order.count();\n  orders = order.find();\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        Method::new("OrderService", "f", 6)
            .call(call(order(), "getCustomer", 7).expecting(class("Customer")))
            .call(call(get("getOwner", 8), "getCustomer", 8).expecting(class("Customer")))
            .call(call(order(), "count", 9).expecting(TypeRef::primitive("int")))
            .call(
                call(order(), "find", 10)
                    .expecting(TypeRef::generic("java.util.List", vec![class("Order")]))
            )
    );
}

#[test]
fn assigning_a_created_object_to_a_field_expects_nothing_of_its_arguments() {
    let source = "class OrderService {\n Customer customer;\n void f(Order order) {\n  customer = new Customer(order.getName());\n  customer = null;\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(3).evaluate(Expression::created(
            Creation::new(class("Customer")).with(get("getName", 4))
        ))
    );
}

#[test]
fn assigning_a_field_of_an_enclosing_class_expects_the_type_declared_there() {
    let source = "class OrderService {\n Customer customer;\n class Inner {\n  void g(Order order) {\n   customer = order.getCustomer();\n  }\n }\n}";

    let mapped = method_in(source, "OrderService$Inner", "g");

    assert_eq!(
        mapped,
        Method::new("OrderService$Inner", "g", 4)
            .call(call(order(), "getCustomer", 5).expecting(class("Customer")))
    );
}

#[test]
fn assigning_into_another_objects_field_expects_nothing() {
    let source = "class OrderService {\n void f(Order order) {\n  order.customer = order.getCustomer();\n }\n}";

    let mapped = method_f(source);

    assert_eq!(mapped, f(2).call(call(order(), "getCustomer", 3)));
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
fn an_expression_lambda_argument_yields_its_value_instead_of_judging_it_in_the_body() {
    let source = "class OrderService {\n void f(Order order) {\n  order.lines().map(line -> line.price());\n  order.lines().map(line -> 1);\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).call(
            call(get("lines", 3), "map", 3).with(Lambda::new(&["line"]).yielding(
                Expression::return_of(call(Expression::local("line"), "price", 3))
            ))
        )
        .call(
            call(get("lines", 4), "map", 4)
                .with(Lambda::new(&["line"]).yielding(Expression::literal()))
        )
    );
}

#[test]
fn a_block_lambda_ending_in_a_return_yields_that_value_and_keeps_the_rest_in_the_body() {
    let source = "class OrderService {\n void f(Order order, boolean flag) {\n  order.lines().map(line -> {\n   Wallet w = line.wallet();\n   if (flag) {\n    return line.first();\n   }\n   return w.balance();\n  });\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).call(
            call(get("lines", 3), "map", 3).with(
                Lambda::new(&["line"])
                    .assign(
                        "w",
                        class("Wallet"),
                        Expression::return_of(call(Expression::local("line"), "wallet", 4))
                    )
                    .call(call(Expression::local("line"), "first", 6))
                    .yielding(Expression::return_of(call(
                        Expression::local("w"),
                        "balance",
                        8
                    )))
            )
        )
    );
}

#[test]
fn a_block_lambda_that_does_not_end_in_a_return_value_yields_nothing() {
    let source = "class OrderService {\n void f(Order order) {\n  order.lines().forEach(line -> {\n   line.apply();\n  });\n  order.lines().forEach(line -> {\n   line.apply();\n   return;\n  });\n }\n}";

    let mapped = method_f(source);

    let apply = |line| call(Expression::local("line"), "apply", line);
    assert_eq!(
        mapped,
        f(2).call(call(get("lines", 3), "forEach", 3).with(Lambda::new(&["line"]).call(apply(4))))
            .call(call(get("lines", 6), "forEach", 6).with(Lambda::new(&["line"]).call(apply(7))))
    );
}

#[test]
fn a_method_reference_argument_keeps_its_target_and_name() {
    let source = "class OrderService {\n void f(Order order) {\n  order.lines().forEach(this::go);\n  order.lines().forEach(Order::go);\n  order.lines().forEach(order::go);\n  order.lines().forEach(order.getCustomer()::notify);\n  order.lines().forEach(Order::new);\n }\n}";

    let mapped = method_f(source);

    let for_each = |line, target, name| {
        call(get("lines", line), "forEach", line).with(MethodReference::new(target, name, line))
    };
    assert_eq!(
        mapped,
        f(2).call(for_each(3, Expression::This, "go"))
            .call(for_each(4, Expression::static_type(class("Order")), "go"))
            .call(for_each(5, order(), "go"))
            .call(for_each(6, get("getCustomer", 6), "notify"))
            .call(for_each(7, Expression::static_type(class("Order")), "new"))
    );
}

#[test]
fn a_method_reference_outside_an_argument_stays_a_value_with_its_target() {
    let source = "class OrderService {\n void f(Order order) {\n  Runnable a = this::go;\n  Runnable b = order.getCustomer()::notify;\n }\n}";

    let mapped = method_f(source);

    let reference =
        |target, name, line| Expression::method_reference(MethodReference::new(target, name, line));
    assert_eq!(
        mapped,
        f(2).assign("a", class("Runnable"), reference(Expression::This, "go", 3))
            .assign(
                "b",
                class("Runnable"),
                reference(get("getCustomer", 4), "notify", 4)
            )
    );
}

#[test]
fn a_class_literal_is_a_literal_typed_as_the_class_of_its_type() {
    let source = "class OrderService {\n void f(Order order) {\n  order.read(Order.class, Ghost.class, Order[].class);\n }\n}";

    let mapped = method_f(source);

    let literal_of = |type_ref| {
        Expression::cast(
            TypeRef::generic("java.lang.Class", vec![type_ref]),
            Expression::literal(),
        )
    };
    assert_eq!(
        mapped,
        f(2).call(
            call(order(), "read", 3)
                .with(literal_of(class("Order")))
                .with(literal_of(class("Ghost")))
                .with(literal_of(TypeRef::array(class("Order"))))
        )
    );
}
