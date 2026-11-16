mod mapped;

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::{Call, Creation, Expression, Lambda, Method};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};
use mapped::{call, class, method_f_in, parameter};

fn shape(name: &str) -> TypeShape {
    TypeShape::new(name, TypeKind::Class)
}

fn interface(name: &str, parameters: &[&str]) -> TypeShape {
    parameters.iter().fold(
        TypeShape::new(name, TypeKind::Interface),
        |shape, parameter| shape.with_type_parameter(parameter),
    )
}

fn generic(name: &str, arguments: &[TypeRef]) -> TypeRef {
    TypeRef::generic(name, arguments.to_vec())
}

fn variable(name: &str) -> TypeRef {
    TypeRef::variable(name)
}

fn stream_of(element: TypeRef) -> TypeRef {
    generic("Stream", &[element])
}

fn order() -> TypeRef {
    class("Order")
}

fn customer() -> TypeRef {
    class("Customer")
}

fn wallet() -> TypeRef {
    class("Wallet")
}

fn world() -> Types {
    let mapping = MethodShape::new("map")
        .with_type_parameter("R")
        .with_parameters(vec![generic("Function", &[variable("T"), variable("R")])])
        .returning(stream_of(variable("R")));
    let applying = MethodShape::new("apply")
        .with_parameters(vec![variable("T")])
        .returning(variable("R"))
        .as_abstract();
    let supplying = MethodShape::new("get")
        .returning(variable("T"))
        .as_abstract();
    let picking = MethodShape::new("pick")
        .as_static()
        .with_type_parameter("T")
        .with_parameters(vec![variable("T"), generic("Supplier", &[variable("T")])])
        .returning(variable("T"));
    Types::new()
        .with_shape(interface("Stream", &["T"]).with_method(mapping))
        .with_shape(interface("Function", &["T", "R"]).with_method(applying))
        .with_shape(interface("Supplier", &["T"]).with_method(supplying))
        .with_shape(
            interface("List", &["E"])
                .with_method(MethodShape::new("stream").returning(stream_of(variable("E")))),
        )
        .with_shape(
            shape("Order")
                .with_method(MethodShape::new("getCustomer").returning(customer()))
                .with_method(MethodShape::new("getWallet").returning(wallet()))
                .with_method(MethodShape::new("count").returning(TypeRef::primitive("int")))
                .with_method(MethodShape::new("total").returning(class("java.lang.Integer"))),
        )
        .with_shape(
            shape("Customer").with_method(MethodShape::new("getWallet").returning(wallet())),
        )
        .with_shape(shape("Premium").with_superclass(customer()))
        .with_shape(shape("Wallet"))
        .with_shape(shape("Util").with_method(picking))
}

fn f(line: usize) -> Method {
    Method::new("OrderService", "f", line)
}

fn mapped_in(parameters: &str, body: &str) -> Method {
    let source = format!("class OrderService {{\n void f({parameters}) {{\n  {body}\n }}\n}}");

    method_f_in(&source, &world())
}

fn orders() -> Expression {
    parameter("orders", generic("List", &[order()]))
}

fn streamed() -> Expression {
    let stream = call(orders(), "stream", 3).returning(stream_of(order()));
    Expression::return_of(stream)
}

fn reading(method: &str, returned: TypeRef, line: usize) -> Expression {
    let read = call(Expression::local("o"), method, line).returning(returned);
    Expression::return_of(read)
}

fn open_stream() -> TypeRef {
    stream_of(variable("R"))
}

fn mapping(lambda: Lambda, returned: Option<TypeRef>) -> Call {
    let map = call(streamed(), "map", 3).with(lambda).returning_from(&[0]);
    match returned {
        Some(returned) => map.returning(returned),
        None => map,
    }
}

fn ordering() -> Lambda {
    Lambda::new(&["o"])
        .resolving("o", order())
        .fed_by_receiver("o")
}

fn mapped_map(lambda_source: &str) -> Method {
    mapped_in(
        "List<Order> orders, boolean flag",
        &format!("orders.stream().map({lambda_source});"),
    )
}

#[test]
fn an_expression_lambda_binds_the_result_type_variable_of_the_method() {
    let mapped = mapped_map("o -> o.getCustomer()");

    let lambda = ordering().yielding(reading("getCustomer", customer(), 3));
    assert_eq!(
        mapped,
        f(2).call(mapping(lambda, Some(stream_of(customer()))))
    );
}

#[test]
fn a_block_lambda_binds_the_result_type_through_its_return() {
    let mapped = mapped_map("o -> {\n   return o.getCustomer();\n  }");

    let lambda = ordering().yielding(reading("getCustomer", customer(), 4));
    assert_eq!(
        mapped,
        f(2).call(mapping(lambda, Some(stream_of(customer()))))
    );
}

fn branching(first: &str, second: &str) -> String {
    format!(
        "o -> {{\n   if (flag) {{\n    return o.{first}();\n   }}\n   return o.{second}();\n  }}"
    )
}

fn branching_lambda(first: (&str, TypeRef), second: (&str, TypeRef)) -> Lambda {
    let early = call(Expression::local("o"), first.0, 5).returning(first.1);
    let last = reading(second.0, second.1, 7);
    ordering().call(early).yielding(last)
}

fn mapped_branches(first: &str, second: &str) -> Method {
    mapped_map(&branching(first, second))
}

#[test]
fn returns_that_agree_bind_the_result_type() {
    let mapped = mapped_branches("getCustomer", "getCustomer");

    let lambda = branching_lambda(("getCustomer", customer()), ("getCustomer", customer()));
    assert_eq!(
        mapped,
        f(2).call(mapping(lambda, Some(stream_of(customer()))))
    );
}

#[test]
fn the_returns_of_a_block_lambda_that_disagree_leave_the_result_unresolved() {
    let mapped = mapped_branches("getCustomer", "getWallet");

    let lambda = branching_lambda(("getCustomer", customer()), ("getWallet", wallet()));
    assert_eq!(mapped, f(2).call(mapping(lambda, Some(open_stream()))));
}

#[test]
fn the_widest_of_the_returns_binds_the_result_type() {
    let source =
        "o -> {\n   if (flag) {\n    return new Premium();\n   }\n   return o.getCustomer();\n  }";

    let mapped = mapped_map(source);

    let early = Expression::created(Creation::new(class("Premium")));
    let lambda = ordering()
        .evaluate(early)
        .yielding(reading("getCustomer", customer(), 7));
    assert_eq!(
        mapped,
        f(2).call(mapping(lambda, Some(stream_of(customer()))))
    );
}

#[test]
fn a_return_of_unknown_type_leaves_the_result_unresolved() {
    let mapped = mapped_branches("getCustomer", "getMystery");

    let early = call(Expression::local("o"), "getCustomer", 5).returning(customer());
    let last = Expression::return_of(call(Expression::local("o"), "getMystery", 7));
    let lambda = ordering().call(early).yielding(last);
    assert_eq!(mapped, f(2).call(mapping(lambda, Some(open_stream()))));
}

#[test]
fn a_primitive_result_binds_its_boxed_type() {
    let mapped = mapped_map("o -> o.count()");

    let lambda = ordering().yielding(reading("count", TypeRef::primitive("int"), 3));
    assert_eq!(
        mapped,
        f(2).call(mapping(lambda, Some(stream_of(class("java.lang.Integer")))))
    );
}

#[test]
fn the_returns_of_a_nested_lambda_are_not_returns_of_the_outer_one() {
    let source = "o -> {\n   Function<Order, Wallet> inner = x -> x.getWallet();\n   return o.getCustomer();\n  }";

    let mapped = mapped_map(source);

    let wallet_of_x = call(Expression::local("x"), "getWallet", 4).returning(wallet());
    let inner = Lambda::new(&["x"])
        .resolving("x", order())
        .yielding(Expression::return_of(wallet_of_x));
    let lambda = ordering()
        .assign(
            "inner",
            generic("Function", &[order(), wallet()]),
            Expression::lambda(inner),
        )
        .yielding(reading("getCustomer", customer(), 5));
    assert_eq!(
        mapped,
        f(2).call(mapping(lambda, Some(stream_of(customer()))))
    );
}

#[test]
fn a_variable_bound_by_another_argument_keeps_its_binding() {
    let mapped = mapped_in(
        "Customer customer, Premium premium",
        "Util.pick(customer, () -> premium).go();",
    );

    let pick = call(Expression::static_type(class("Util")), "pick", 3)
        .with(parameter("customer", customer()))
        .with(Lambda::new(&[]).yielding(parameter("premium", class("Premium"))))
        .returning(customer())
        .returning_from(&[1]);
    assert_eq!(mapped, f(2).call(pick.then("go", 3)));
}

#[test]
fn the_result_of_a_mapped_stream_types_the_next_lambda() {
    let mapped = mapped_map("o -> o.getCustomer()).map(c -> c.getWallet()");

    let inner = mapping(
        ordering().yielding(reading("getCustomer", customer(), 3)),
        Some(stream_of(customer())),
    );
    let wallet_of_c = call(Expression::local("c"), "getWallet", 3).returning(wallet());
    let second = call(Expression::return_of(inner), "map", 3)
        .with(
            Lambda::new(&["c"])
                .resolving("c", customer())
                .fed_by_receiver("c")
                .yielding(Expression::return_of(wallet_of_c)),
        )
        .returning(stream_of(wallet()))
        .returning_from(&[0]);
    assert_eq!(mapped, f(2).call(second));
}

#[test]
fn a_primitive_and_its_wrapper_among_the_returns_bind_the_wrapper() {
    let mapped = mapped_branches("count", "total");

    let lambda = branching_lambda(
        ("count", TypeRef::primitive("int")),
        ("total", class("java.lang.Integer")),
    );
    assert_eq!(
        mapped,
        f(2).call(mapping(lambda, Some(stream_of(class("java.lang.Integer")))))
    );
}
