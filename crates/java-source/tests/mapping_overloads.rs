mod mapped;

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::{Creation, Expression, Method};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};
use mapped::{call, class, method_f_in, parameter};

const PARAMETERS: &str = "Order order, Customer customer, Premium premium, Integer boxed, Object any, Order[] orders, Ghost ghost";

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
        .returning(class(returned))
        .with_parameters(parameters)
}

fn int() -> TypeRef {
    TypeRef::primitive("int")
}

fn long() -> TypeRef {
    TypeRef::primitive("long")
}

fn double() -> TypeRef {
    TypeRef::primitive("double")
}

fn mapped_pick(arguments: &str, overloads: Vec<MethodShape>) -> Method {
    mapped_pick_in(arguments, order_with(overloads), world())
}

fn mapped_pick_in(arguments: &str, order: TypeShape, types: Types) -> Method {
    let source = format!(
        "class OrderService {{\n void f({PARAMETERS}) {{\n  order.pick({arguments}).go();\n }}\n}}"
    );

    method_f_in(&source, &types.with_shape(order))
}

fn order_with(overloads: Vec<MethodShape>) -> TypeShape {
    overloads
        .into_iter()
        .fold(shape("Order"), TypeShape::with_method)
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

fn boxed() -> Expression {
    parameter("boxed", class("java.lang.Integer"))
}

fn ghost() -> Expression {
    parameter("ghost", class("Ghost"))
}

#[test]
fn an_identical_parameter_type_is_chosen() {
    let overloads = vec![
        pick("Customer", vec![class("java.lang.String")]),
        pick("Wallet", vec![class("Wallet")]),
    ];

    let mapped = mapped_pick("\"text\"", overloads);

    assert_eq!(mapped, expected_pick(vec![literal()], Some("Customer")));
}

#[test]
fn a_new_object_argument_has_the_type_it_creates() {
    let overloads = vec![
        pick("Customer", vec![class("Customer")]),
        pick("Wallet", vec![class("Wallet")]),
    ];

    let mapped = mapped_pick("new Premium()", overloads);

    let created = Expression::created(Creation::new(class("Premium")));
    assert_eq!(mapped, expected_pick(vec![created], Some("Customer")));
}

#[test]
fn a_cast_argument_has_the_type_it_casts_to() {
    let overloads = vec![
        pick("Customer", vec![class("Customer")]),
        pick("Wallet", vec![class("Wallet")]),
    ];

    let mapped = mapped_pick("(Premium) any", overloads);

    let cast = Expression::cast(
        class("Premium"),
        parameter("any", class("java.lang.Object")),
    );
    assert_eq!(mapped, expected_pick(vec![cast], Some("Customer")));
}

#[test]
fn a_cast_to_a_primitive_gives_the_argument_that_type() {
    let overloads = vec![
        pick("Customer", vec![int()]),
        pick("Wallet", vec![class("java.lang.String")]),
    ];

    let mapped = mapped_pick("(int) boxed", overloads);

    let cast = Expression::cast(int(), boxed());
    assert_eq!(mapped, expected_pick(vec![cast], Some("Customer")));
}

#[test]
fn an_array_argument_fits_a_parameter_of_object_but_not_of_a_class() {
    let overloads = vec![
        pick("Customer", vec![class("Customer")]),
        pick("Wallet", vec![class("java.lang.Object")]),
    ];

    let mapped = mapped_pick("orders", overloads);

    let orders = parameter("orders", TypeRef::array(class("Order")));
    assert_eq!(mapped, expected_pick(vec![orders], Some("Wallet")));
}

#[test]
fn an_int_argument_widens_to_a_long_parameter() {
    let overloads = vec![
        pick("Customer", vec![long()]),
        pick("Wallet", vec![class("java.lang.String")]),
    ];

    let mapped = mapped_pick("1", overloads);

    assert_eq!(mapped, expected_pick(vec![literal()], Some("Customer")));
}

#[test]
fn a_char_argument_widens_to_an_int_parameter() {
    let overloads = vec![
        pick("Customer", vec![int()]),
        pick("Wallet", vec![class("java.lang.String")]),
    ];

    let mapped = mapped_pick("'a'", overloads);

    assert_eq!(mapped, expected_pick(vec![literal()], Some("Customer")));
}

#[test]
fn a_boolean_argument_does_not_widen_to_an_int_parameter() {
    let overloads = vec![
        pick("Customer", vec![int()]),
        pick("Wallet", vec![class("java.lang.String")]),
    ];

    let mapped = mapped_pick("true", overloads);

    assert_eq!(mapped, expected_pick(vec![literal()], None));
}

#[test]
fn a_subclass_argument_fits_the_parameter_of_its_superclass() {
    let overloads = vec![
        pick("Customer", vec![class("Customer")]),
        pick("Wallet", vec![class("Wallet")]),
    ];

    let mapped = mapped_pick("premium", overloads);

    assert_eq!(mapped, expected_pick(vec![premium()], Some("Customer")));
}

#[test]
fn an_int_argument_is_boxed_when_no_overload_fits_without_boxing() {
    let overloads = vec![
        pick("Customer", vec![class("java.lang.Integer")]),
        pick("Wallet", vec![class("java.lang.String")]),
    ];

    let mapped = mapped_pick("1", overloads);

    assert_eq!(mapped, expected_pick(vec![literal()], Some("Customer")));
}

#[test]
fn a_boxed_argument_is_unboxed_when_no_overload_fits_without_unboxing() {
    let overloads = vec![
        pick("Customer", vec![int()]),
        pick("Wallet", vec![class("java.lang.String")]),
    ];

    let mapped = mapped_pick("boxed", overloads);

    assert_eq!(mapped, expected_pick(vec![boxed()], Some("Customer")));
}

#[test]
fn an_unboxed_argument_widens_to_a_long_parameter() {
    let overloads = vec![
        pick("Customer", vec![long()]),
        pick("Wallet", vec![class("java.lang.String")]),
    ];

    let mapped = mapped_pick("boxed", overloads);

    assert_eq!(mapped, expected_pick(vec![boxed()], Some("Customer")));
}

#[test]
fn a_boxed_int_argument_fits_a_supertype_of_integer() {
    let overloads = vec![
        pick("Customer", vec![class("java.lang.Number")]),
        pick("Wallet", vec![class("java.lang.String")]),
    ];

    let mapped = mapped_pick("1", overloads);

    assert_eq!(mapped, expected_pick(vec![literal()], Some("Customer")));
}

#[test]
fn an_overload_that_fits_without_boxing_wins_over_one_that_needs_boxing() {
    let overloads = vec![
        pick("Customer", vec![class("java.lang.Integer")]),
        pick("Wallet", vec![long()]),
    ];

    let mapped = mapped_pick("1", overloads);

    assert_eq!(mapped, expected_pick(vec![literal()], Some("Wallet")));
}

#[test]
fn an_overload_that_fits_by_subtyping_wins_over_one_that_needs_unboxing() {
    let overloads = vec![
        pick("Customer", vec![int()]),
        pick("Wallet", vec![class("java.lang.Object")]),
    ];

    let mapped = mapped_pick("boxed", overloads);

    assert_eq!(mapped, expected_pick(vec![boxed()], Some("Wallet")));
}

#[test]
fn the_most_specific_primitive_parameter_is_chosen() {
    let overloads = vec![
        pick("Wallet", vec![double()]),
        pick("Customer", vec![long()]),
    ];

    let mapped = mapped_pick("1", overloads);

    assert_eq!(mapped, expected_pick(vec![literal()], Some("Customer")));
}

#[test]
fn the_most_specific_reference_parameter_is_chosen() {
    let overloads = vec![
        pick("Wallet", vec![class("java.lang.Object")]),
        pick("Customer", vec![class("Customer")]),
    ];

    let mapped = mapped_pick("premium", overloads);

    assert_eq!(mapped, expected_pick(vec![premium()], Some("Customer")));
}

#[test]
fn null_picks_the_most_specific_reference_parameter() {
    let overloads = vec![
        pick("Wallet", vec![class("java.lang.Object")]),
        pick("Customer", vec![class("java.lang.String")]),
    ];

    let mapped = mapped_pick("null", overloads);

    assert_eq!(mapped, expected_pick(vec![literal()], Some("Customer")));
}

#[test]
fn null_does_not_fit_a_primitive_parameter() {
    let overloads = vec![
        pick("Wallet", vec![int()]),
        pick("Customer", vec![class("java.lang.String")]),
    ];

    let mapped = mapped_pick("null", overloads);

    assert_eq!(mapped, expected_pick(vec![literal()], Some("Customer")));
}

#[test]
fn unrelated_overloads_that_both_fit_leave_the_call_unresolved() {
    let overloads = vec![
        pick("Customer", vec![class("java.lang.String")]),
        pick("Wallet", vec![class("Customer")]),
    ];

    let mapped = mapped_pick("null", overloads);

    assert_eq!(mapped, expected_pick(vec![literal()], None));
}

#[test]
fn overloads_none_of_which_is_more_specific_in_every_parameter_leave_the_call_unresolved() {
    let overloads = vec![
        pick("Customer", vec![int(), long()]),
        pick("Wallet", vec![long(), int()]),
    ];

    let mapped = mapped_pick("1, 1", overloads);

    assert_eq!(mapped, expected_pick(vec![literal(), literal()], None));
}

#[test]
fn overloads_none_of_which_fits_leave_the_call_unresolved() {
    let overloads = vec![
        pick("Customer", vec![class("java.lang.String")]),
        pick("Wallet", vec![class("Customer")]),
    ];

    let mapped = mapped_pick("1", overloads);

    assert_eq!(mapped, expected_pick(vec![literal()], None));
}

#[test]
fn an_argument_of_unknown_type_resolves_only_when_the_overloads_agree() {
    let overloads = vec![
        pick("Customer", vec![class("java.lang.String")]),
        pick("Customer", vec![class("Wallet")]),
    ];

    let mapped = mapped_pick("ghost", overloads);

    assert_eq!(mapped, expected_pick(vec![ghost()], Some("Customer")));
}

#[test]
fn an_argument_of_unknown_type_fits_a_class_parameter_but_not_a_primitive_one() {
    let overloads = vec![
        pick("Wallet", vec![int()]),
        pick("Customer", vec![class("Wallet")]),
    ];

    let mapped = mapped_pick("ghost", overloads);

    assert_eq!(mapped, expected_pick(vec![ghost()], Some("Customer")));
}

#[test]
fn overloads_none_of_which_fits_still_resolve_when_they_agree_on_the_return() {
    let overloads = vec![
        pick("Customer", vec![class("java.lang.String")]),
        pick("Customer", vec![class("Wallet")]),
    ];

    let mapped = mapped_pick("1", overloads);

    assert_eq!(mapped, expected_pick(vec![literal()], Some("Customer")));
}

#[test]
fn overloads_that_both_fit_resolve_when_they_agree_on_the_return() {
    let overloads = vec![
        pick("Customer", vec![class("java.lang.String")]),
        pick("Customer", vec![class("Wallet")]),
    ];

    let mapped = mapped_pick("null", overloads);

    assert_eq!(mapped, expected_pick(vec![literal()], Some("Customer")));
}

#[test]
fn a_fixed_arity_overload_is_chosen_though_the_variable_arity_one_cannot_take_the_argument() {
    let overloads = vec![
        pick("Customer", vec![TypeRef::variable("T")]).with_type_parameter("T"),
        pick("Wallet", vec![TypeRef::array(TypeRef::variable("T"))])
            .with_type_parameter("T")
            .as_varargs(),
    ];

    let mapped = mapped_pick("premium", overloads);

    assert_eq!(mapped, expected_pick(vec![premium()], Some("Customer")));
}

#[test]
fn an_argument_whose_ancestors_are_unknown_leaves_disagreeing_overloads_unresolved() {
    let types = world().with_shape(shape("Gold").with_superclass(class("Ghost")));
    let overloads = vec![
        pick("Customer", vec![class("Wallet")]),
        pick("Wallet", vec![class("java.lang.Object")]),
    ];
    let source =
        "class OrderService {\n void f(Order order, Gold gold) {\n  order.pick(gold).go();\n }\n}";

    let mapped = method_f_in(source, &types.with_shape(order_with(overloads)));

    let call =
        call(parameter("order", class("Order")), "pick", 3).with(parameter("gold", class("Gold")));
    assert_eq!(
        mapped,
        Method::new("OrderService", "f", 2).call(call.then("go", 3))
    );
}

#[test]
fn an_overload_with_a_parameter_of_a_type_variable_leaves_disagreeing_overloads_unresolved() {
    let overloads = vec![
        pick("Customer", vec![TypeRef::variable("T")]),
        pick("Wallet", vec![class("java.lang.String")]),
    ];

    let mapped = mapped_pick("\"text\"", overloads);

    assert_eq!(mapped, expected_pick(vec![literal()], None));
}

#[test]
fn an_overload_of_a_superclass_takes_part_in_the_selection() {
    let types =
        world().with_shape(shape("Document").with_method(pick("Wallet", vec![class("Customer")])));
    let order = shape("Order")
        .with_superclass(class("Document"))
        .with_method(pick("Customer", vec![class("java.lang.String")]));

    let mapped = mapped_pick_in("premium", order, types);

    assert_eq!(mapped, expected_pick(vec![premium()], Some("Wallet")));
}

#[test]
fn an_override_hides_the_overload_it_overrides() {
    let types = world().with_shape(
        shape("Document")
            .with_method(pick("Wallet", vec![class("Customer")]))
            .with_method(pick("Wallet", vec![class("java.lang.String")])),
    );
    let order = shape("Order")
        .with_superclass(class("Document"))
        .with_method(pick("Customer", vec![class("Customer")]));

    let mapped = mapped_pick_in("premium", order, types);

    assert_eq!(mapped, expected_pick(vec![premium()], Some("Customer")));
}

#[test]
fn a_type_argument_of_the_receiver_decides_which_overload_fits() {
    let source = "class OrderService {\n void f(Holder<Customer> holder, Premium premium) {\n  holder.pick(premium).go();\n }\n}";
    let holder = shape("Holder")
        .with_type_parameter("T")
        .with_method(pick("Customer", vec![TypeRef::variable("T")]))
        .with_method(pick("Wallet", vec![class("Wallet")]));

    let mapped = method_f_in(source, &world().with_shape(holder));

    let holder_type = TypeRef::generic("Holder", vec![class("Customer")]);
    let call = call(parameter("holder", holder_type), "pick", 3)
        .with(premium())
        .returning(class("Customer"));
    assert_eq!(
        mapped,
        Method::new("OrderService", "f", 2).call(call.then("go", 3))
    );
}
