mod mapped;

use law_of_demeter_core::domain::method::{Creation, Expression, Method};
use law_of_demeter_core::domain::type_shape::TypeRef;
use mapped::{call, class, method_f, parameter};

fn f(line: usize) -> Method {
    Method::new("OrderService", "f", line)
}

#[test]
fn a_call_on_a_parameter_has_the_parameter_with_its_declared_type_as_receiver() {
    let source = "class OrderService {\n void f(Order order) {\n  order.go();\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).call(call(parameter("order", class("Order")), "go", 3))
    );
}

#[test]
fn declared_types_keep_generics_arrays_variables_and_wildcards() {
    let source = "import java.util.List;\nclass OrderService<T> {\n void f(List<String> a, int[] b, T c, List<? extends Order> d, List<?> e, Ghost g) {\n  a.go();\n  b.go();\n  c.go();\n  d.go();\n  e.go();\n  g.go();\n }\n}";

    let mapped = method_f(source);

    let receivers = [
        (
            "a",
            TypeRef::generic("java.util.List", vec![class("java.lang.String")]),
        ),
        ("b", TypeRef::array(TypeRef::primitive("int"))),
        ("c", TypeRef::variable("T")),
        (
            "d",
            TypeRef::generic("java.util.List", vec![class("Order")]),
        ),
        (
            "e",
            TypeRef::generic("java.util.List", vec![class("java.lang.Object")]),
        ),
        ("g", class("Ghost")),
    ];
    let expected = receivers
        .into_iter()
        .zip(4..)
        .fold(f(3), |method, ((name, type_ref), line)| {
            method.call(call(parameter(name, type_ref), "go", line))
        });
    assert_eq!(mapped, expected);
}

#[test]
fn a_chain_maps_to_calls_on_the_returns_of_the_previous_calls() {
    let source = "class OrderService {\n void f(Order order, int amount) {\n  order.getCustomer().getWallet().withdraw(amount);\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).call(
            call(parameter("order", class("Order")), "getCustomer", 3)
                .then("getWallet", 3)
                .then("withdraw", 3)
                .with(parameter("amount", TypeRef::primitive("int")))
        )
    );
}

#[test]
fn every_call_of_a_chain_carries_its_own_line() {
    let source = "class OrderService {\n void f(Order order) {\n  order\n   .getCustomer()\n   .getWallet();\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).call(call(parameter("order", class("Order")), "getCustomer", 4).then("getWallet", 5))
    );
}

#[test]
fn fields_are_receivers_with_their_declared_types_with_or_without_this() {
    let source = "class OrderService {\n private Wallet wallet;\n void f(int amount) {\n  wallet.withdraw(amount);\n  this.wallet.withdraw(amount);\n }\n}";

    let mapped = method_f(source);

    let wallet = Expression::field("wallet", class("Wallet"));
    let amount = || parameter("amount", TypeRef::primitive("int"));
    assert_eq!(
        mapped,
        Method::new("OrderService", "f", 3)
            .call(call(wallet.clone(), "withdraw", 4).with(amount()))
            .call(call(wallet, "withdraw", 5).with(amount()))
    );
}

#[test]
fn own_super_and_outer_methods_are_called_on_this() {
    let source = "class OrderService {\n void f() {\n  this.audit();\n  audit();\n  super.audit();\n }\n void audit() {}\n class Inner {\n  void f() {\n   audit();\n  }\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).call(call(Expression::This, "audit", 3))
            .call(call(Expression::This, "audit", 4))
            .call(call(Expression::This, "audit", 5))
    );
}

#[test]
fn a_call_on_a_type_name_is_a_static_call() {
    let source = "class OrderService {\n void f() {\n  Order.builder();\n  Ghost.make();\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).call(call(Expression::static_type(class("Order")), "builder", 3))
            .call(call(Expression::unresolved("Ghost"), "make", 4))
    );
}

#[test]
fn a_statically_imported_method_is_called_on_the_type_that_provides_it() {
    let source = "import static x.Y.max;\nclass OrderService {\n void f() {\n  max(1, 2);\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        Method::new("OrderService", "f", 3).call(
            call(Expression::static_type(class("x.Y")), "max", 4)
                .with(Expression::literal())
                .with(Expression::literal())
        )
    );
}

#[test]
fn arguments_keep_literals_parameters_nested_calls_and_creations() {
    let source = "class OrderService {\n private Notifier notifier;\n void f(Order order) {\n  notifier.send(1, order, order.getCustomer(), new Wallet(order));\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        Method::new("OrderService", "f", 3).call(
            call(Expression::field("notifier", class("Notifier")), "send", 4)
                .with(Expression::literal())
                .with(parameter("order", class("Order")))
                .with(Expression::return_of(call(
                    parameter("order", class("Order")),
                    "getCustomer",
                    4
                )))
                .with(Expression::created(
                    Creation::new(class("Wallet")).with(parameter("order", class("Order")))
                ))
        )
    );
}

#[test]
fn a_name_nothing_resolves_reaches_the_domain_as_unresolved() {
    let source = "class OrderService {\n void f() {\n  log.info(\"x\");\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).call(call(Expression::unresolved("log"), "info", 3).with(Expression::literal()))
    );
}

#[test]
fn a_method_called_on_a_created_object_has_the_creation_as_receiver() {
    let source = "class OrderService {\n void f(String name) {\n  new StringBuilder().append(\"Hello \").append(name);\n }\n}";

    let mapped = method_f(source);

    assert_eq!(
        mapped,
        f(2).call(
            call(
                Expression::created(Creation::new(class("java.lang.StringBuilder"))),
                "append",
                3
            )
            .with(Expression::literal())
            .then("append", 3)
            .with(parameter("name", class("java.lang.String")))
        )
    );
}

#[test]
fn a_name_from_on_demand_static_imports_carries_their_types_as_candidates_in_order() {
    let source = "import static x.Y.*;\nimport static java.lang.Math.*;\nclass OrderService {\n void f(Order order) {\n  order.go(PAID);\n  PAID.name();\n }\n}";

    let mapped = method_f(source);

    let paid = || Expression::unresolved_among("PAID", vec![class("x.Y"), class("java.lang.Math")]);
    assert_eq!(
        mapped,
        Method::new("OrderService", "f", 4)
            .call(call(parameter("order", class("Order")), "go", 5).with(paid()))
            .call(call(paid(), "name", 6))
    );
}
