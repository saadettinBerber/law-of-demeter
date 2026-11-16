mod mapped;

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::{Call, Expression, Method};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};
use mapped::{call, class, method_f_in, parameter};

const LIST: &str = "java.util.List";
const ARRAY_LIST: &str = "java.util.ArrayList";
const COLLECTION: &str = "java.util.Collection";
const MAP: &str = "java.util.Map";
const IMPORTS: &str = "import java.util.List; import java.util.ArrayList; import java.util.Map;";
const PARAMETERS: &str = "Order order, Customer customer, Premium premium, Order[] orders, List<Order> list, ArrayList<Order> arrayList, int[] ids";

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

fn list_of(element: TypeRef) -> TypeRef {
    generic(LIST, &[element])
}

fn order() -> TypeRef {
    class("Order")
}

fn world() -> Types {
    Types::new()
        .with_shape(shape("java.lang.Object"))
        .with_shape(shape("java.lang.String"))
        .with_shape(shape("java.lang.Number"))
        .with_shape(shape("java.lang.Integer").with_superclass(class("java.lang.Number")))
        .with_shape(shape("Order"))
        .with_shape(shape("Customer"))
        .with_shape(shape("Premium").with_superclass(class("Customer")))
        .with_shape(interface(COLLECTION, &["E"]))
        .with_shape(interface(LIST, &["E"]).extending(generic(COLLECTION, &[variable("E")])))
        .with_shape(
            shape(ARRAY_LIST)
                .with_type_parameter("E")
                .extending(generic(LIST, &[variable("E")])),
        )
        .with_shape(interface(MAP, &["K", "V"]))
}

fn pick(variables: &[&str], parameters: Vec<TypeRef>, returned: TypeRef) -> MethodShape {
    variables.iter().fold(
        MethodShape::new("pick")
            .as_static()
            .with_parameters(parameters)
            .returning(returned),
        |method, variable| method.with_type_parameter(variable),
    )
}

fn util(methods: Vec<MethodShape>) -> Types {
    let util = methods
        .into_iter()
        .fold(shape("Util"), TypeShape::with_method);

    world().with_shape(util)
}

fn mapped_in(types: &Types, body: &str) -> Method {
    let source = format!(
        "{IMPORTS} class OrderService {{\n List<Order> items;\n void f({PARAMETERS}) {{\n  {body}\n }}\n}}"
    );

    method_f_in(&source, types)
}

fn mapped_pick(arguments: &str, methods: Vec<MethodShape>) -> Method {
    mapped_in(&util(methods), &format!("Util.pick({arguments}).go();"))
}

fn util_pick(arguments: Vec<Expression>) -> Call {
    let receiver = Expression::static_type(class("Util"));

    arguments
        .into_iter()
        .fold(call(receiver, "pick", 4), |call, argument| {
            call.with(argument)
        })
}

fn f(line: usize) -> Method {
    Method::new("OrderService", "f", line)
}

fn expected_pick(arguments: Vec<Expression>, returned: Option<TypeRef>) -> Method {
    let call = util_pick(arguments);
    let call = match returned {
        Some(returned) => call.returning(returned),
        None => call,
    };

    f(3).call(call.then("go", 4))
}

fn literal() -> Expression {
    Expression::literal()
}

fn an_order() -> Expression {
    parameter("order", order())
}

fn a_customer() -> Expression {
    parameter("customer", class("Customer"))
}

fn a_premium() -> Expression {
    parameter("premium", class("Premium"))
}

fn the_orders() -> Expression {
    parameter("orders", TypeRef::array(order()))
}

fn the_list() -> Expression {
    parameter("list", list_of(order()))
}

fn the_array_list() -> Expression {
    parameter("arrayList", generic(ARRAY_LIST, &[order()]))
}

fn the_ids() -> Expression {
    parameter("ids", TypeRef::array(TypeRef::primitive("int")))
}

fn listing(parameters: Vec<TypeRef>) -> MethodShape {
    pick(&["T"], parameters, list_of(variable("T")))
}

fn listing_varargs() -> MethodShape {
    listing(vec![TypeRef::array(variable("T"))]).as_varargs()
}

#[test]
fn an_argument_binds_the_type_variable_of_the_return_type() {
    let methods = vec![listing(vec![variable("T")])];

    let mapped = mapped_pick("order", methods);

    assert_eq!(
        mapped,
        expected_pick(vec![an_order()], Some(list_of(order())))
    );
}

#[test]
fn a_type_variable_is_bound_inside_a_generic_parameter() {
    let methods = vec![pick(&["T"], vec![list_of(variable("T"))], variable("T"))];

    let mapped = mapped_pick("list", methods);

    assert_eq!(mapped, expected_pick(vec![the_list()], Some(order())));
}

#[test]
fn an_array_argument_binds_the_element_type_variable() {
    let methods = vec![listing(vec![TypeRef::array(variable("T"))])];

    let mapped = mapped_pick("orders", methods);

    assert_eq!(
        mapped,
        expected_pick(vec![the_orders()], Some(list_of(order())))
    );
}

#[test]
fn a_supertype_of_the_argument_binds_the_type_variable() {
    let methods = vec![pick(
        &["T"],
        vec![generic(COLLECTION, &[variable("T")])],
        variable("T"),
    )];

    let mapped = mapped_pick("arrayList", methods);

    assert_eq!(mapped, expected_pick(vec![the_array_list()], Some(order())));
}

#[test]
fn a_primitive_literal_is_boxed_to_bind_the_type_variable() {
    let methods = vec![listing(vec![variable("T")])];

    let mapped = mapped_pick("1", methods);

    assert_eq!(
        mapped,
        expected_pick(vec![literal()], Some(list_of(class("java.lang.Integer"))))
    );
}

#[test]
fn a_literal_argument_binds_its_own_type_variable() {
    let returned = generic(MAP, &[variable("K"), variable("V")]);
    let methods = vec![pick(
        &["K", "V"],
        vec![variable("K"), variable("V")],
        returned,
    )];

    let mapped = mapped_pick("\"created\", order", methods);

    let bound = generic(MAP, &[class("java.lang.String"), order()]);
    assert_eq!(
        mapped,
        expected_pick(vec![literal(), an_order()], Some(bound))
    );
}

#[test]
fn a_null_argument_adds_no_constraint() {
    let methods = vec![listing(vec![variable("T"), variable("T")])];

    let mapped = mapped_pick("null, order", methods);

    assert_eq!(
        mapped,
        expected_pick(vec![literal(), an_order()], Some(list_of(order())))
    );
}

#[test]
fn a_type_variable_bound_only_by_null_stays_unresolved() {
    let methods = vec![listing(vec![variable("T")])];

    let mapped = mapped_pick("null", methods);

    assert_eq!(mapped, expected_pick(vec![literal()], None));
}

#[test]
fn arguments_that_agree_bind_the_type_variable() {
    let methods = vec![listing_varargs()];

    let mapped = mapped_pick("order, order", methods);

    assert_eq!(
        mapped,
        expected_pick(vec![an_order(), an_order()], Some(list_of(order())))
    );
}

#[test]
fn the_argument_type_that_is_a_supertype_of_the_others_binds_the_type_variable() {
    let methods = vec![listing_varargs()];

    let mapped = mapped_pick("premium, customer", methods);

    assert_eq!(
        mapped,
        expected_pick(
            vec![a_premium(), a_customer()],
            Some(list_of(class("Customer")))
        )
    );
}

#[test]
fn the_widest_argument_type_wins_whatever_the_argument_order() {
    let methods = vec![listing_varargs()];

    let mapped = mapped_pick("customer, premium", methods);

    assert_eq!(
        mapped,
        expected_pick(
            vec![a_customer(), a_premium()],
            Some(list_of(class("Customer")))
        )
    );
}

#[test]
fn argument_types_without_a_widest_one_leave_the_type_variable_unresolved() {
    let methods = vec![listing_varargs()];

    let mapped = mapped_pick("order, customer", methods);

    assert_eq!(mapped, expected_pick(vec![an_order(), a_customer()], None));
}

#[test]
fn an_array_passed_to_a_variable_arity_method_binds_the_element_type_variable() {
    let methods = vec![listing_varargs()];

    let mapped = mapped_pick("orders", methods);

    assert_eq!(
        mapped,
        expected_pick(vec![the_orders()], Some(list_of(order())))
    );
}

#[test]
fn a_primitive_array_does_not_bind_an_element_type_variable() {
    let methods = vec![listing(vec![TypeRef::array(variable("T"))])];

    let mapped = mapped_pick("ids", methods);

    assert_eq!(mapped, expected_pick(vec![the_ids()], None));
}

#[test]
fn the_parameters_after_binding_decide_which_overload_applies() {
    let methods = vec![
        pick(
            &["T"],
            vec![variable("T"), TypeRef::primitive("int")],
            variable("T"),
        ),
        pick(
            &["T"],
            vec![variable("T"), class("java.lang.String")],
            class("Customer"),
        ),
    ];

    let mapped = mapped_pick("order, 1", methods);

    assert_eq!(
        mapped,
        expected_pick(vec![an_order(), literal()], Some(order()))
    );
}

#[test]
fn a_nested_generic_call_binds_through_its_return_type() {
    let wrap = MethodShape::new("wrap")
        .as_static()
        .with_type_parameter("T")
        .with_parameters(vec![variable("T")])
        .returning(list_of(variable("T")));
    let first = pick(&["T"], vec![list_of(variable("T"))], variable("T"));

    let mapped = mapped_in(
        &util(vec![wrap, first]),
        "Util.pick(Util.wrap(order)).go();",
    );

    let wrapped = call(Expression::static_type(class("Util")), "wrap", 4)
        .with(an_order())
        .returning(list_of(order()));
    let picked = util_pick(vec![Expression::return_of(wrapped)]).returning(order());
    assert_eq!(mapped, f(3).call(picked.then("go", 4)));
}

#[test]
fn a_type_variable_no_argument_binds_stays_unresolved_without_an_expected_type() {
    let methods = vec![listing(vec![])];

    let mapped = mapped_pick("", methods);

    assert_eq!(mapped, expected_pick(vec![], None));
}

#[test]
fn the_declared_type_of_a_local_binds_a_type_variable_no_argument_binds() {
    let types = util(vec![listing(vec![])]);

    let mapped = mapped_in(&types, "List<Order> orders = Util.pick();");

    let value = Expression::return_of(util_pick(vec![]).returning(list_of(order())));
    assert_eq!(mapped, f(3).assign("orders", list_of(order()), value));
}

#[test]
fn the_return_type_of_the_method_binds_a_type_variable_no_argument_binds() {
    let types = util(vec![listing(vec![])]);
    let source = format!(
        "{IMPORTS} class OrderService {{\n List<Order> items;\n List<Order> f() {{\n  return Util.pick();\n }}\n}}"
    );

    let mapped = method_f_in(&source, &types);

    let bound = list_of(order());
    let call = util_pick(vec![]).returning(bound.clone()).expecting(bound);
    assert_eq!(mapped, f(3).call(call));
}

#[test]
fn the_type_of_a_field_binds_a_type_variable_no_argument_binds_when_it_is_assigned() {
    let types = util(vec![listing(vec![])]);

    let mapped = mapped_in(&types, "items = Util.pick();");

    let bound = list_of(order());
    let call = util_pick(vec![]).returning(bound.clone()).expecting(bound);
    assert_eq!(mapped, f(3).call(call));
}

#[test]
fn the_declared_type_of_a_local_binds_a_type_variable_when_the_local_is_assigned_again() {
    let types = util(vec![listing(vec![])]);

    let mapped = mapped_in(&types, "List<Order> orders = null; orders = Util.pick();");

    let bound = list_of(order());
    let value = Expression::return_of(util_pick(vec![]).returning(bound.clone()));
    assert_eq!(
        mapped,
        f(3).assign("orders", bound.clone(), Expression::literal())
            .assign("orders", bound, value)
    );
}

#[test]
fn the_expected_type_binds_through_a_supertype_of_the_return_type() {
    let methods = vec![pick(&["T"], vec![], generic(ARRAY_LIST, &[variable("T")]))];

    let mapped = mapped_in(&util(methods), "List<Order> orders = Util.pick();");

    let bound = generic(ARRAY_LIST, &[order()]);
    let value = Expression::return_of(util_pick(vec![]).returning(bound));
    assert_eq!(mapped, f(3).assign("orders", list_of(order()), value));
}

#[test]
fn the_expected_type_wins_inside_a_type_argument_of_the_return_type() {
    let types = util(vec![listing(vec![variable("T")])]);

    let mapped = mapped_in(&types, "List<Customer> customers = Util.pick(premium);");

    let bound = list_of(class("Customer"));
    let value = Expression::return_of(util_pick(vec![a_premium()]).returning(bound.clone()));
    assert_eq!(mapped, f(3).assign("customers", bound, value));
}

#[test]
fn an_expected_type_the_argument_does_not_fit_leaves_the_call_unresolved() {
    let types = util(vec![listing(vec![variable("T")])]);

    let mapped = mapped_in(&types, "List<Premium> premiums = Util.pick(customer);");

    let value = Expression::return_of(util_pick(vec![a_customer()]));
    assert_eq!(
        mapped,
        f(3).assign("premiums", list_of(class("Premium")), value)
    );
}

#[test]
fn the_argument_type_is_kept_when_the_type_variable_is_the_whole_return_type() {
    let types = util(vec![pick(&["T"], vec![variable("T")], variable("T"))]);

    let mapped = mapped_in(&types, "Customer chosen = Util.pick(premium);");

    let value = Expression::return_of(util_pick(vec![a_premium()]).returning(class("Premium")));
    assert_eq!(mapped, f(3).assign("chosen", class("Customer"), value));
}
