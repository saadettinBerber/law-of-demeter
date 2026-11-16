mod mapped;

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::{Expression, Method};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};
use mapped::{call, class, method_f_in, parameter};

fn f(line: usize) -> Method {
    Method::new("OrderService", "f", line)
}

fn shape(name: &str) -> TypeShape {
    TypeShape::new(name, TypeKind::Class)
}

fn returning(name: &str, returned: &str) -> MethodShape {
    MethodShape::new(name).returning(class(returned))
}

fn shop() -> Types {
    Types::new()
        .with_shape(shape("java.lang.String"))
        .with_shape(shape("Customer").with_method(returning("getWallet", "Wallet")))
        .with_shape(shape("Wallet"))
}

#[test]
fn a_method_declared_on_the_receiver_gives_the_call_its_return_type() {
    let source = "class OrderService {\n void f(Order order) {\n  order.getCustomer().go();\n }\n}";
    let types = shop().with_shape(shape("Order").with_method(returning("getCustomer", "Customer")));

    let mapped = method_f_in(source, &types);

    assert_eq!(
        mapped,
        f(2).call(
            call(parameter("order", class("Order")), "getCustomer", 3)
                .returning(class("Customer"))
                .then("go", 3)
        )
    );
}

#[test]
fn a_method_inherited_from_a_superclass_gives_the_call_its_return_type() {
    let source = "class OrderService {\n void f(Order order) {\n  order.getOwner().go();\n }\n}";
    let types = shop()
        .with_shape(shape("Document").with_method(returning("getOwner", "Customer")))
        .with_shape(shape("Order").with_superclass(class("Document")));

    let mapped = method_f_in(source, &types);

    assert_eq!(
        mapped,
        f(2).call(
            call(parameter("order", class("Order")), "getOwner", 3)
                .returning(class("Customer"))
                .then("go", 3)
        )
    );
}

#[test]
fn a_method_of_java_lang_object_gives_the_call_its_return_type() {
    let source = "class OrderService {\n void f(Order order) {\n  order.toString().go();\n }\n}";
    let types = shop()
        .with_shape(
            shape("java.lang.Object").with_method(returning("toString", "java.lang.String")),
        )
        .with_shape(shape("Order"));

    let mapped = method_f_in(source, &types);

    assert_eq!(
        mapped,
        f(2).call(
            call(parameter("order", class("Order")), "toString", 3)
                .returning(class("java.lang.String"))
                .then("go", 3)
        )
    );
}

#[test]
fn every_call_of_a_chain_gets_the_return_type_of_the_one_before_it_as_receiver() {
    let source = "class OrderService {\n void f(Order order) {\n  order.getCustomer().getWallet().go();\n }\n}";
    let types = shop().with_shape(shape("Order").with_method(returning("getCustomer", "Customer")));

    let mapped = method_f_in(source, &types);

    assert_eq!(
        mapped,
        f(2).call(
            call(parameter("order", class("Order")), "getCustomer", 3)
                .returning(class("Customer"))
                .then("getWallet", 3)
                .returning(class("Wallet"))
                .then("go", 3)
        )
    );
}

#[test]
fn overloads_of_different_arity_each_give_their_own_return_type() {
    let source = "class OrderService {\n void f(Order order) {\n  order.pick().go();\n  order.pick(1).go();\n }\n}";
    let types = shop().with_shape(
        shape("Order")
            .with_method(returning("pick", "Customer"))
            .with_method(
                returning("pick", "Wallet").with_parameters(vec![TypeRef::primitive("int")]),
            ),
    );

    let mapped = method_f_in(source, &types);

    let order = || parameter("order", class("Order"));
    assert_eq!(
        mapped,
        f(2).call(
            call(order(), "pick", 3)
                .returning(class("Customer"))
                .then("go", 3)
        )
        .call(
            call(order(), "pick", 4)
                .with(Expression::literal())
                .returning(class("Wallet"))
                .then("go", 4)
        )
    );
}

#[test]
fn an_overload_of_another_arity_in_a_subclass_does_not_hide_the_inherited_one() {
    let source = "class OrderService {\n void f(Order order) {\n  order.pick().go();\n }\n}";
    let types = shop()
        .with_shape(shape("Document").with_method(returning("pick", "Customer")))
        .with_shape(
            shape("Order")
                .with_superclass(class("Document"))
                .with_method(
                    returning("pick", "Wallet").with_parameters(vec![TypeRef::primitive("int")]),
                ),
        );

    let mapped = method_f_in(source, &types);

    assert_eq!(
        mapped,
        f(2).call(
            call(parameter("order", class("Order")), "pick", 3)
                .returning(class("Customer"))
                .then("go", 3)
        )
    );
}

#[test]
fn overloads_of_the_same_arity_that_return_the_same_type_give_the_call_that_type() {
    let source = "class OrderService {\n void f(Order order) {\n  order.find(1).go();\n }\n}";
    let types = shop().with_shape(
        shape("Order")
            .with_method(
                returning("find", "Customer").with_parameters(vec![TypeRef::primitive("int")]),
            )
            .with_method(
                returning("find", "Customer").with_parameters(vec![class("java.lang.String")]),
            ),
    );

    let mapped = method_f_in(source, &types);

    assert_eq!(
        mapped,
        f(2).call(
            call(parameter("order", class("Order")), "find", 3)
                .with(Expression::literal())
                .returning(class("Customer"))
                .then("go", 3)
        )
    );
}

#[test]
fn a_field_receiver_gives_the_call_its_return_type() {
    let source = "class OrderService {\n private Order order;\n void f() {\n  order.getCustomer().go();\n }\n}";
    let types = shop().with_shape(shape("Order").with_method(returning("getCustomer", "Customer")));

    let mapped = method_f_in(source, &types);

    assert_eq!(
        mapped,
        Method::new("OrderService", "f", 3).call(
            call(Expression::field("order", class("Order")), "getCustomer", 4)
                .returning(class("Customer"))
                .then("go", 4)
        )
    );
}

#[test]
fn a_typed_local_receiver_gives_the_call_its_return_type() {
    let source =
        "class OrderService {\n void f() {\n  Order order;\n  order.getCustomer().go();\n }\n}";
    let types = shop().with_shape(shape("Order").with_method(returning("getCustomer", "Customer")));

    let mapped = method_f_in(source, &types);

    assert_eq!(
        mapped,
        f(2).call(
            call(Expression::local("order"), "getCustomer", 4)
                .returning(class("Customer"))
                .then("go", 4)
        )
    );
}

#[test]
fn a_call_without_a_receiver_looks_its_method_up_on_the_own_class() {
    let source = "class OrderService {\n void f() {\n  customer().go();\n }\n}";
    let types =
        shop().with_shape(shape("OrderService").with_method(returning("customer", "Customer")));

    let mapped = method_f_in(source, &types);

    assert_eq!(
        mapped,
        f(2).call(
            call(Expression::This, "customer", 3)
                .returning(class("Customer"))
                .then("go", 3)
        )
    );
}

#[test]
fn a_static_type_receiver_gives_the_call_its_return_type() {
    let source = "class OrderService {\n void f() {\n  Customers.create().go();\n }\n}";
    let types = shop().with_shape(shape("Customers").with_method(returning("create", "Customer")));

    let mapped = method_f_in(source, &types);

    assert_eq!(
        mapped,
        f(2).call(
            call(Expression::static_type(class("Customers")), "create", 3)
                .returning(class("Customer"))
                .then("go", 3)
        )
    );
}

#[test]
fn a_receiver_the_adapter_cannot_type_leaves_the_call_unresolved() {
    let source =
        "class OrderService {\n void f(Ghost ghost) {\n  ghost.getCustomer().getWallet();\n }\n}";

    let mapped = method_f_in(source, &shop());

    assert_eq!(
        mapped,
        f(2).call(call(parameter("ghost", class("Ghost")), "getCustomer", 3).then("getWallet", 3))
    );
}

#[test]
fn a_local_declared_with_var_leaves_the_call_unresolved() {
    let source = "class OrderService {\n void f(Order order) {\n  var customer = order;\n  customer.getCustomer().go();\n }\n}";
    let types = shop().with_shape(shape("Order").with_method(returning("getCustomer", "Customer")));

    let mapped = method_f_in(source, &types);

    assert_eq!(
        mapped,
        f(2).assign_var("customer", Expression::parameter("order", class("Order")))
            .call(call(Expression::local("customer"), "getCustomer", 4).then("go", 4))
    );
}

#[test]
fn overloads_of_the_same_arity_that_return_different_types_leave_the_call_unresolved() {
    let source = "class OrderService {\n void f(Order order) {\n  order.find(1).go();\n }\n}";
    let types = shop().with_shape(
        shape("Order")
            .with_method(
                returning("find", "Customer").with_parameters(vec![TypeRef::primitive("int")]),
            )
            .with_method(
                returning("find", "Wallet").with_parameters(vec![class("java.lang.String")]),
            ),
    );

    let mapped = method_f_in(source, &types);

    assert_eq!(
        mapped,
        f(2).call(
            call(parameter("order", class("Order")), "find", 3)
                .with(Expression::literal())
                .then("go", 3)
        )
    );
}

#[test]
fn a_generic_return_type_leaves_the_call_unresolved() {
    let source = "class OrderService {\n void f(Order order) {\n  order.getItems().go();\n }\n}";
    let items = TypeRef::generic("java.util.List", vec![TypeRef::variable("T")]);
    let types = shop()
        .with_shape(shape("Order").with_method(MethodShape::new("getItems").returning(items)));

    let mapped = method_f_in(source, &types);

    assert_eq!(
        mapped,
        f(2).call(call(parameter("order", class("Order")), "getItems", 3).then("go", 3))
    );
}

#[test]
fn a_varargs_method_leaves_the_call_unresolved() {
    let source = "class OrderService {\n void f(Order order) {\n  order.pick(1).go();\n }\n}";
    let types = shop().with_shape(
        shape("Order").with_method(
            returning("pick", "Customer")
                .as_varargs()
                .with_parameters(vec![TypeRef::array(class("java.lang.Object"))]),
        ),
    );

    let mapped = method_f_in(source, &types);

    assert_eq!(
        mapped,
        f(2).call(
            call(parameter("order", class("Order")), "pick", 3)
                .with(Expression::literal())
                .then("go", 3)
        )
    );
}

#[test]
fn a_method_that_returns_nothing_leaves_the_call_unresolved() {
    let source = "class OrderService {\n void f(Order order) {\n  order.cancel().go();\n }\n}";
    let types = shop().with_shape(shape("Order").with_method(MethodShape::new("cancel")));

    let mapped = method_f_in(source, &types);

    assert_eq!(
        mapped,
        f(2).call(call(parameter("order", class("Order")), "cancel", 3).then("go", 3))
    );
}

#[test]
fn a_varargs_overload_next_to_a_fixed_one_leaves_the_call_unresolved() {
    let source = "class OrderService {\n void f(Order order) {\n  order.pick(1).go();\n }\n}";
    let types = shop().with_shape(
        shape("Order")
            .with_method(
                returning("pick", "Customer").with_parameters(vec![TypeRef::primitive("int")]),
            )
            .with_method(
                returning("pick", "Wallet")
                    .as_varargs()
                    .with_parameters(vec![TypeRef::array(class("java.lang.Object"))]),
            ),
    );

    let mapped = method_f_in(source, &types);

    assert_eq!(
        mapped,
        f(2).call(
            call(parameter("order", class("Order")), "pick", 3)
                .with(Expression::literal())
                .then("go", 3)
        )
    );
}

fn list_of(element: &str) -> TypeRef {
    TypeRef::generic("java.util.List", vec![class(element)])
}

fn list_shape() -> TypeShape {
    shape("java.util.List")
        .with_type_parameter("E")
        .with_method(
            MethodShape::new("get")
                .with_parameters(vec![TypeRef::primitive("int")])
                .returning(TypeRef::variable("E")),
        )
}

#[test]
fn a_type_argument_of_the_receiver_gives_the_call_its_return_type() {
    let source = "import java.util.List;\nclass OrderService {\n void f(List<Order> orders) {\n  orders.get(0).go();\n }\n}";
    let types = shop().with_shape(list_shape());

    let mapped = method_f_in(source, &types);

    assert_eq!(
        mapped,
        Method::new("OrderService", "f", 3).call(
            call(parameter("orders", list_of("Order")), "get", 4)
                .with(Expression::literal())
                .returning(class("Order"))
                .then("go", 4)
        )
    );
}

#[test]
fn type_arguments_are_substituted_inside_a_nested_return_type() {
    let source = "import java.util.Map;\nclass OrderService {\n void f(Map<String, Order> orders) {\n  orders.entrySet().go();\n }\n}";
    let entry = |key, value| TypeRef::generic("java.util.Map$Entry", vec![key, value]);
    let entries = TypeRef::generic(
        "java.util.Set",
        vec![entry(TypeRef::variable("K"), TypeRef::variable("V"))],
    );
    let types = shop().with_shape(
        shape("java.util.Map")
            .with_type_parameter("K")
            .with_type_parameter("V")
            .with_method(MethodShape::new("entrySet").returning(entries)),
    );

    let mapped = method_f_in(source, &types);

    let orders = TypeRef::generic(
        "java.util.Map",
        vec![class("java.lang.String"), class("Order")],
    );
    let resolved = TypeRef::generic(
        "java.util.Set",
        vec![entry(class("java.lang.String"), class("Order"))],
    );
    assert_eq!(
        mapped,
        Method::new("OrderService", "f", 3).call(
            call(parameter("orders", orders), "entrySet", 4)
                .returning(resolved)
                .then("go", 4)
        )
    );
}

#[test]
fn type_arguments_are_carried_through_the_supertypes_of_the_receiver() {
    let source = "class OrderService {\n void f(OrderList orders) {\n  orders.get(0).go();\n }\n}";
    let array_list = |element| TypeRef::generic("java.util.ArrayList", vec![element]);
    let types = shop()
        .with_shape(list_shape())
        .with_shape(
            shape("java.util.ArrayList")
                .with_type_parameter("E")
                .extending(TypeRef::generic(
                    "java.util.List",
                    vec![TypeRef::variable("E")],
                )),
        )
        .with_shape(shape("OrderList").with_superclass(array_list(class("Order"))));

    let mapped = method_f_in(source, &types);

    assert_eq!(
        mapped,
        f(2).call(
            call(parameter("orders", class("OrderList")), "get", 3)
                .with(Expression::literal())
                .returning(class("Order"))
                .then("go", 3)
        )
    );
}

#[test]
fn a_raw_receiver_leaves_a_call_returning_a_type_variable_unresolved() {
    let source = "import java.util.List;\nclass OrderService {\n void f(List orders) {\n  orders.get(0).go();\n }\n}";
    let types = shop().with_shape(list_shape());

    let mapped = method_f_in(source, &types);

    assert_eq!(
        mapped,
        Method::new("OrderService", "f", 3).call(
            call(parameter("orders", class("java.util.List")), "get", 4)
                .with(Expression::literal())
                .then("go", 4)
        )
    );
}

#[test]
fn a_type_variable_of_the_method_itself_leaves_the_call_unresolved_even_if_the_class_names_it_too()
{
    let source = "class OrderService {\n void f(Box box) {\n  box.pick().go();\n }\n}";
    let types = shop().with_shape(
        shape("Box").with_type_parameter("T").with_method(
            MethodShape::new("pick")
                .with_type_parameter("T")
                .returning(TypeRef::variable("T")),
        ),
    );

    let mapped = method_f_in(source, &types);

    assert_eq!(
        mapped,
        f(2).call(call(parameter("box", class("Box")), "pick", 3).then("go", 3))
    );
}

#[test]
fn a_raw_receiver_leaves_a_call_returning_a_parameterized_type_unresolved() {
    let source = "import java.util.List;\nclass OrderService {\n void f(List orders) {\n  orders.names().go();\n }\n}";
    let names = TypeRef::generic("java.util.List", vec![class("java.lang.String")]);
    let types = shop().with_shape(
        shape("java.util.List")
            .with_type_parameter("E")
            .with_method(MethodShape::new("names").returning(names)),
    );

    let mapped = method_f_in(source, &types);

    assert_eq!(
        mapped,
        Method::new("OrderService", "f", 3)
            .call(call(parameter("orders", class("java.util.List")), "names", 4).then("go", 4))
    );
}

#[test]
fn a_raw_receiver_still_gives_a_call_returning_a_plain_type_that_type() {
    let source = "import java.util.List;\nclass OrderService {\n void f(List orders) {\n  orders.owner().go();\n }\n}";
    let types = shop().with_shape(
        shape("java.util.List")
            .with_type_parameter("E")
            .with_method(returning("owner", "Customer")),
    );

    let mapped = method_f_in(source, &types);

    assert_eq!(
        mapped,
        Method::new("OrderService", "f", 3).call(
            call(parameter("orders", class("java.util.List")), "owner", 4)
                .returning(class("Customer"))
                .then("go", 4)
        )
    );
}
