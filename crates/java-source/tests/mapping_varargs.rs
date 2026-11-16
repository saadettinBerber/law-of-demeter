mod mapped;

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::{Expression, Method};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};
use mapped::{call, class, method_f_in, parameter};

const PARAMETERS: &str =
    "Order order, Customer customer, Premium premium, Object any, Order[] orders, Ghost ghost";

fn shape(name: &str) -> TypeShape {
    TypeShape::new(name, TypeKind::Class)
}

fn world() -> Types {
    Types::new()
        .with_shape(shape("java.lang.Object"))
        .with_shape(shape("java.lang.String"))
        .with_shape(shape("java.lang.Number"))
        .with_shape(shape("java.lang.Integer").with_superclass(class("java.lang.Number")))
        .with_shape(shape("Customer"))
        .with_shape(shape("Premium").with_superclass(class("Customer")))
        .with_shape(shape("Wallet"))
}

fn pick(returned: &str, parameters: Vec<TypeRef>) -> MethodShape {
    MethodShape::new("pick")
        .as_varargs()
        .returning(class(returned))
        .with_parameters(parameters)
}

fn fixed_pick(returned: &str, parameters: Vec<TypeRef>) -> MethodShape {
    MethodShape::new("pick")
        .returning(class(returned))
        .with_parameters(parameters)
}

fn array_of(name: &str) -> TypeRef {
    TypeRef::array(class(name))
}

fn mapped_pick(arguments: &str, overloads: Vec<MethodShape>) -> Method {
    let source = format!(
        "class OrderService {{\n void f({PARAMETERS}) {{\n  order.pick({arguments}).go();\n }}\n}}"
    );
    let order = overloads
        .into_iter()
        .fold(shape("Order"), TypeShape::with_method);

    method_f_in(&source, &world().with_shape(order))
}

fn expected_pick(arguments: Vec<Expression>, returned: Option<&str>) -> Method {
    let call = arguments.into_iter().fold(
        call(parameter("order", class("Order")), "pick", 3),
        |call, argument| call.with(argument),
    );
    let call = match returned {
        Some(returned) => call.returning(class(returned)),
        None => call,
    };
    Method::new("OrderService", "f", 2).call(call.then("go", 3))
}

fn literal() -> Expression {
    Expression::literal()
}

fn premium() -> Expression {
    parameter("premium", class("Premium"))
}

fn customer() -> Expression {
    parameter("customer", class("Customer"))
}

fn orders() -> Expression {
    parameter("orders", array_of("Order"))
}

fn ghost() -> Expression {
    parameter("ghost", class("Ghost"))
}

#[test]
fn a_variable_arity_method_is_chosen_without_any_argument_for_its_array() {
    let overloads = vec![pick("Customer", vec![array_of("Customer")])];

    let mapped = mapped_pick("", overloads);

    assert_eq!(mapped, expected_pick(vec![], Some("Customer")));
}

#[test]
fn a_variable_arity_method_is_chosen_with_many_arguments() {
    let overloads = vec![pick("Customer", vec![array_of("Customer")])];

    let mapped = mapped_pick("customer, premium, customer", overloads);

    let arguments = vec![customer(), premium(), customer()];
    assert_eq!(mapped, expected_pick(arguments, Some("Customer")));
}

#[test]
fn the_arguments_after_the_fixed_parameters_fit_the_element_type() {
    let overloads = vec![pick(
        "Customer",
        vec![class("java.lang.String"), array_of("java.lang.Object")],
    )];

    let mapped = mapped_pick("\"%s %s\", customer, 1", overloads);

    let arguments = vec![literal(), customer(), literal()];
    assert_eq!(mapped, expected_pick(arguments, Some("Customer")));
}

#[test]
fn the_variable_arguments_are_boxed_to_fit_the_element_type() {
    let overloads = vec![pick("Customer", vec![array_of("java.lang.Integer")])];

    let mapped = mapped_pick("1, 2", overloads);

    assert_eq!(
        mapped,
        expected_pick(vec![literal(), literal()], Some("Customer"))
    );
}

#[test]
fn an_array_argument_is_passed_directly_to_the_variable_arity_method() {
    let overloads = vec![pick("Customer", vec![array_of("Order")])];

    let mapped = mapped_pick("orders", overloads);

    assert_eq!(mapped, expected_pick(vec![orders()], Some("Customer")));
}

#[test]
fn a_fixed_arity_overload_wins_over_a_variable_arity_one() {
    let overloads = vec![
        pick("Wallet", vec![array_of("Customer")]),
        fixed_pick("Customer", vec![class("Customer")]),
    ];

    let mapped = mapped_pick("premium", overloads);

    assert_eq!(mapped, expected_pick(vec![premium()], Some("Customer")));
}

#[test]
fn a_variable_arity_overload_is_used_only_when_no_fixed_arity_one_fits() {
    let overloads = vec![
        pick("Wallet", vec![array_of("Customer")]),
        fixed_pick("Customer", vec![class("Order")]),
    ];

    let mapped = mapped_pick("premium", overloads);

    assert_eq!(mapped, expected_pick(vec![premium()], Some("Wallet")));
}

#[test]
fn the_variable_arity_overload_with_the_most_specific_element_type_is_chosen() {
    let overloads = vec![
        pick("Wallet", vec![array_of("java.lang.Object")]),
        pick("Customer", vec![array_of("Customer")]),
    ];

    let mapped = mapped_pick("premium, customer", overloads);

    let arguments = vec![premium(), customer()];
    assert_eq!(mapped, expected_pick(arguments, Some("Customer")));
}

#[test]
fn the_variable_arity_overload_with_a_fixed_parameter_is_more_specific_than_the_one_without() {
    let overloads = vec![
        pick("Wallet", vec![array_of("java.lang.Object")]),
        pick(
            "Customer",
            vec![class("java.lang.String"), array_of("java.lang.Object")],
        ),
    ];

    let mapped = mapped_pick("\"title\", premium, premium", overloads);

    let arguments = vec![literal(), premium(), premium()];
    assert_eq!(mapped, expected_pick(arguments, Some("Customer")));
}

#[test]
fn a_fixed_parameter_decides_specificity_when_no_variable_argument_is_given() {
    let overloads = vec![
        pick("Wallet", vec![array_of("java.lang.Object")]),
        pick(
            "Customer",
            vec![class("java.lang.String"), array_of("java.lang.Object")],
        ),
    ];

    let mapped = mapped_pick("\"title\"", overloads);

    assert_eq!(mapped, expected_pick(vec![literal()], Some("Customer")));
}

#[test]
fn a_longer_parameter_list_that_is_not_more_specific_in_its_tail_leaves_the_call_unresolved() {
    let overloads = vec![
        pick("Wallet", vec![array_of("Customer")]),
        pick(
            "Customer",
            vec![class("Premium"), array_of("java.lang.Object")],
        ),
    ];

    let mapped = mapped_pick("premium", overloads);

    assert_eq!(mapped, expected_pick(vec![premium()], None));
}

#[test]
fn variable_arity_overloads_neither_of_which_is_more_specific_leave_the_call_unresolved() {
    let overloads = vec![
        pick(
            "Wallet",
            vec![class("java.lang.String"), array_of("java.lang.Object")],
        ),
        pick(
            "Customer",
            vec![class("java.lang.Object"), array_of("java.lang.String")],
        ),
    ];

    let mapped = mapped_pick("\"a\", \"b\"", overloads);

    assert_eq!(mapped, expected_pick(vec![literal(), literal()], None));
}

#[test]
fn a_variable_arity_overload_none_of_whose_elements_fit_leaves_the_call_unresolved() {
    let overloads = vec![pick("Customer", vec![array_of("Order")])];

    let mapped = mapped_pick("customer, 1", overloads);

    assert_eq!(mapped, expected_pick(vec![customer(), literal()], None));
}

#[test]
fn an_argument_of_unknown_type_resolves_variable_arity_overloads_only_when_they_agree() {
    let overloads = vec![
        pick("Customer", vec![array_of("Order")]),
        pick(
            "Customer",
            vec![class("java.lang.String"), array_of("Order")],
        ),
    ];

    let mapped = mapped_pick("ghost, ghost", overloads);

    let arguments = vec![ghost(), ghost()];
    assert_eq!(mapped, expected_pick(arguments, Some("Customer")));
}

#[test]
fn an_argument_of_unknown_type_leaves_disagreeing_variable_arity_overloads_unresolved() {
    let overloads = vec![
        pick("Wallet", vec![array_of("Order")]),
        pick(
            "Customer",
            vec![class("java.lang.String"), array_of("Order")],
        ),
    ];

    let mapped = mapped_pick("ghost, ghost", overloads);

    assert_eq!(mapped, expected_pick(vec![ghost(), ghost()], None));
}
