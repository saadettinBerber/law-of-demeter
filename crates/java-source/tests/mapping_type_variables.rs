mod mapped;

use law_of_demeter_core::domain::method::{Expression, Method};
use law_of_demeter_core::domain::type_shape::TypeRef;
use mapped::{call, class, method_in, parameter};

fn item(type_ref: TypeRef) -> Expression {
    parameter("item", type_ref)
}

#[test]
fn a_method_type_variable_maps_to_its_bound() {
    let source = "class OrderService {\n <T extends Order> void f(T item) {\n  item.go();\n }\n}";

    let mapped = method_in(source, "OrderService", "f");

    assert_eq!(
        mapped,
        Method::new("OrderService", "f", 2).call(call(item(class("Order")), "go", 3))
    );
}

#[test]
fn a_class_type_variable_maps_to_its_bound_in_the_class_and_its_inner_classes() {
    let source = "class OrderService<T extends Order> {\n T held;\n void f(T item) {\n  item.go();\n  held.go();\n }\n class Inner {\n  void g(T item) {\n   item.go();\n  }\n }\n}";

    let outer = method_in(source, "OrderService", "f");
    let inner = method_in(source, "OrderService$Inner", "g");

    assert_eq!(
        outer,
        Method::new("OrderService", "f", 3)
            .call(call(item(class("Order")), "go", 4))
            .call(call(Expression::field("held", class("Order")), "go", 5))
    );
    assert_eq!(
        inner,
        Method::new("OrderService$Inner", "g", 8).call(call(item(class("Order")), "go", 9))
    );
}

#[test]
fn only_the_first_bound_of_an_intersection_is_kept() {
    let source =
        "class OrderService {\n <T extends Order & Customer> void f(T item) {\n  item.go();\n }\n}";

    let mapped = method_in(source, "OrderService", "f");

    assert_eq!(
        mapped,
        Method::new("OrderService", "f", 2).call(call(item(class("Order")), "go", 3))
    );
}

#[test]
fn an_unbounded_type_variable_stays_a_variable() {
    let source = "class OrderService<T> {\n <U> void f(T item, U other) {\n  item.go();\n  other.go();\n }\n}";

    let mapped = method_in(source, "OrderService", "f");

    assert_eq!(
        mapped,
        Method::new("OrderService", "f", 2)
            .call(call(item(TypeRef::variable("T")), "go", 3))
            .call(call(parameter("other", TypeRef::variable("U")), "go", 4))
    );
}

#[test]
fn a_method_type_variable_hides_a_class_type_variable_of_the_same_name() {
    let source = "class OrderService<T extends Order> {\n <T> void f(T item) {\n  item.go();\n }\n <T extends Customer> void g(T item) {\n  item.go();\n }\n}";

    let unbounded = method_in(source, "OrderService", "f");
    let bounded = method_in(source, "OrderService", "g");

    assert_eq!(
        unbounded,
        Method::new("OrderService", "f", 2).call(call(item(TypeRef::variable("T")), "go", 3))
    );
    assert_eq!(
        bounded,
        Method::new("OrderService", "g", 5).call(call(item(class("Customer")), "go", 6))
    );
}

#[test]
fn a_bound_that_names_its_own_variable_keeps_that_variable_inside() {
    let source =
        "class OrderService {\n <T extends Comparable<T>> void f(T item) {\n  item.go();\n }\n}";

    let mapped = method_in(source, "OrderService", "f");

    assert_eq!(
        mapped,
        Method::new("OrderService", "f", 2).call(call(
            item(TypeRef::generic("Comparable", vec![TypeRef::variable("T")])),
            "go",
            3
        ))
    );
}

#[test]
fn type_variables_are_replaced_wherever_a_declared_type_is_used() {
    let source = "import java.util.List;\nclass OrderService {\n <T extends Order> T f(List<T> items, T[] more) {\n  T first = items.get(0);\n  first.go();\n  ((T) more[0]).go();\n  return items.pick();\n }\n}";

    let mapped = method_in(source, "OrderService", "f");

    let list = TypeRef::generic("java.util.List", vec![class("Order")]);
    assert_eq!(
        mapped,
        Method::new("OrderService", "f", 3)
            .assign(
                "first",
                class("Order"),
                Expression::return_of(
                    call(parameter("items", list.clone()), "get", 4).with(Expression::literal())
                )
            )
            .call(call(Expression::local("first"), "go", 5))
            .call(call(
                Expression::cast(
                    class("Order"),
                    Expression::operation(vec![
                        parameter("more", TypeRef::array(class("Order"))),
                        Expression::literal()
                    ])
                ),
                "go",
                6
            ))
            .call(call(parameter("items", list), "pick", 7))
    );
}
