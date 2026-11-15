use super::{Call, Expression, Lambda, Method, MethodReference, Reason, Verdict};
use crate::domain::classification::Types;
use crate::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

const STREAM: &str = "java.util.stream.Stream";
const OPTIONAL: &str = "java.util.Optional";
const LIST: &str = "java.util.List";
const FUNCTION: &str = "java.util.function.Function";
const CONSUMER: &str = "java.util.function.Consumer";
const STRING: &str = "java.lang.String";
const OBJECT: &str = "java.lang.Object";

fn class(name: &str) -> TypeShape {
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

fn order() -> TypeRef {
    TypeRef::class("Order")
}

fn stream_of(element: TypeRef) -> TypeRef {
    generic(STREAM, &[element])
}

fn streams(varargs_returns: TypeRef) -> Types {
    let stream = interface(STREAM, &["T"])
        .with_method(
            MethodShape::new("of")
                .as_static()
                .with_type_parameter("T")
                .with_parameters(vec![variable("T")])
                .returning(stream_of(variable("T"))),
        )
        .with_method(
            MethodShape::new("of")
                .as_static()
                .as_varargs()
                .with_type_parameter("T")
                .with_parameters(vec![TypeRef::array(variable("T"))])
                .returning(varargs_returns),
        )
        .with_method(
            MethodShape::new("flatMap")
                .with_type_parameter("R")
                .with_parameters(vec![generic(
                    FUNCTION,
                    &[variable("T"), stream_of(variable("R"))],
                )])
                .returning(stream_of(variable("R"))),
        )
        .with_method(MethodShape::new("toList").returning(generic(LIST, &[variable("T")])))
        .with_method(
            MethodShape::new("forEach").with_parameters(vec![generic(CONSUMER, &[variable("T")])]),
        );
    Types::new()
        .with_shape(stream)
        .with_shape(
            interface(FUNCTION, &["T", "R"]).with_method(
                MethodShape::new("apply")
                    .with_parameters(vec![variable("T")])
                    .returning(variable("R"))
                    .as_abstract(),
            ),
        )
        .with_shape(
            interface(CONSUMER, &["T"]).with_method(
                MethodShape::new("accept")
                    .with_parameters(vec![variable("T")])
                    .as_abstract(),
            ),
        )
        .with_shape(
            interface(OPTIONAL, &["T"])
                .with_method(MethodShape::new("stream").returning(stream_of(variable("T")))),
        )
        .with_shape(
            interface(LIST, &["E"]).with_method(
                MethodShape::new("get")
                    .with_parameters(vec![TypeRef::primitive("int")])
                    .returning(variable("E")),
            ),
        )
        .with_shape(
            class("Order")
                .with_method(MethodShape::new("getCustomer").returning(TypeRef::class("Customer"))),
        )
        .with_shape(class("Customer").with_method(MethodShape::new("rename")))
}

fn optional_order(name: &str) -> Expression {
    Expression::parameter(name, generic(OPTIONAL, &[order()]))
}

#[test]
fn a_variable_arity_method_matches_more_arguments_than_it_has_parameters() {
    let notify = Method::new("Shipping", "notify", 1).call(
        Call::new(Expression::static_type(TypeRef::class(STREAM)), "of", 2)
            .with(optional_order("first"))
            .with(optional_order("second"))
            .with(optional_order("third"))
            .then("flatMap", 2)
            .with(MethodReference::new(
                Expression::static_type(TypeRef::class(OPTIONAL)),
                "stream",
                2,
            ))
            .then("toList", 2)
            .then("get", 2)
            .with(Expression::literal())
            .then("getCustomer", 2)
            .then("rename", 2),
    );

    let verdicts = notify.verdicts(&streams(stream_of(variable("T"))));

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("of", 2, Reason::static_type(TypeRef::class(STREAM))),
            Verdict::allowed(
                "stream",
                2,
                Reason::method_reference(
                    TypeRef::class(OPTIONAL),
                    "stream",
                    Reason::static_result("of")
                )
            ),
            Verdict::allowed("flatMap", 2, Reason::static_result("of")),
            Verdict::allowed("toList", 2, Reason::static_result("of")),
            Verdict::allowed("get", 2, Reason::static_result("of")),
            Verdict::allowed("getCustomer", 2, Reason::static_result("of")),
            Verdict::violation("rename", 2, Reason::return_of("getCustomer")),
        ]
    );
}

fn varargs_method(name: &str, parameters: Vec<TypeRef>, returns: &str) -> MethodShape {
    MethodShape::new(name)
        .as_varargs()
        .with_parameters(parameters)
        .returning(TypeRef::class(returns))
}

fn array_of(name: &str) -> TypeRef {
    TypeRef::array(TypeRef::class(name))
}

fn with_helpers(types: Types) -> Types {
    types
        .with_shape(
            class(STRING)
                .with_method(
                    varargs_method(
                        "format",
                        vec![TypeRef::class(STRING), array_of(OBJECT)],
                        STRING,
                    )
                    .as_static(),
                )
                .with_method(MethodShape::new("trim").returning(TypeRef::class(STRING)))
                .with_method(MethodShape::new("isEmpty").returning(TypeRef::primitive("boolean"))),
        )
        .with_shape(
            class("Picker")
                .with_method(varargs_method("pick", vec![array_of("Order")], "Order").as_static()),
        )
        .with_shape(
            class("Actions").with_method(
                MethodShape::new("run")
                    .as_static()
                    .as_varargs()
                    .with_parameters(vec![TypeRef::array(generic(CONSUMER, &[order()]))]),
            ),
        )
        .with_shape(class("Builder").with_method(varargs_method(
            "add",
            vec![array_of(STRING)],
            "Builder",
        )))
        .with_shape(
            class("Order")
                .with_method(MethodShape::new("getCustomer").returning(TypeRef::class("Customer")))
                .with_method(MethodShape::new("getBuilder").returning(TypeRef::class("Builder"))),
        )
        .with_shape(
            class("Printer")
                .with_method(varargs_method("print", vec![array_of(OBJECT)], "Wallet"))
                .with_method(varargs_method(
                    "print",
                    vec![TypeRef::class(STRING), array_of(OBJECT)],
                    "Customer",
                )),
        )
        .with_shape(
            class("Joiner")
                .with_method(varargs_method("join", vec![array_of(STRING)], "Wallet"))
                .with_method(varargs_method(
                    "join",
                    vec![TypeRef::array(TypeRef::primitive("int"))],
                    "Customer",
                )),
        )
        .with_shape(class("Wallet").with_method(MethodShape::new("withdraw")))
        .with_shape(class("Customer").with_method(MethodShape::new("rename")))
}

fn string_parameter(name: &str) -> Expression {
    Expression::parameter(name, TypeRef::class(STRING))
}

#[test]
fn a_variable_arity_method_takes_the_arguments_after_its_fixed_parameters() {
    let describe = Method::new("Shipping", "describe", 1).call(
        Call::new(Expression::static_type(TypeRef::class(STRING)), "format", 2)
            .with(Expression::literal())
            .with(Expression::parameter("first", order()))
            .with(Expression::parameter("second", order()))
            .then("trim", 2)
            .then("isEmpty", 2),
    );

    let verdicts = describe.verdicts(&with_helpers(streams(stream_of(variable("T")))));

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("format", 2, Reason::static_type(TypeRef::class(STRING))),
            Verdict::allowed("trim", 2, Reason::static_result("format")),
            Verdict::allowed("isEmpty", 2, Reason::static_result("format")),
        ]
    );
}

#[test]
fn a_variable_arity_method_matches_without_any_argument_for_its_variable_parameter() {
    let notify = Method::new("Shipping", "notify", 1).call(
        Call::new(Expression::static_type(TypeRef::class("Picker")), "pick", 2)
            .then("getCustomer", 2)
            .then("rename", 2),
    );

    let verdicts = notify.verdicts(&with_helpers(streams(stream_of(variable("T")))));

    assert_eq!(
        verdicts[2],
        Verdict::violation("rename", 2, Reason::return_of("getCustomer"))
    );
}

#[test]
fn a_lambda_at_a_variable_arity_position_takes_the_element_type_of_the_array_parameter() {
    let renaming = |name: &str| {
        Lambda::new(&[name]).call(Call::new(
            Expression::return_of(Call::new(Expression::local(name), "getCustomer", 2)),
            "rename",
            2,
        ))
    };
    let notify = Method::new("Shipping", "notify", 1).call(
        Call::new(Expression::static_type(TypeRef::class("Actions")), "run", 2)
            .with(renaming("first"))
            .with(renaming("second")),
    );

    let verdicts = notify.verdicts(&with_helpers(streams(stream_of(variable("T")))));

    assert_eq!(
        [&verdicts[1], &verdicts[3]],
        [
            &Verdict::violation("rename", 2, Reason::return_of("getCustomer")),
            &Verdict::violation("rename", 2, Reason::return_of("getCustomer")),
        ]
    );
}

#[test]
fn a_variable_arity_builder_method_stays_fluent_with_any_number_of_arguments() {
    let build = Method::new("Shipping", "build", 1).call(
        Call::new(Expression::parameter("order", order()), "getBuilder", 2)
            .then("add", 2)
            .with(string_parameter("first"))
            .with(string_parameter("second"))
            .then("add", 2)
            .with(string_parameter("third")),
    );

    let verdicts = build.verdicts(&with_helpers(streams(stream_of(variable("T")))));

    let builder = TypeRef::class("Builder");
    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getBuilder", 2, Reason::parameter("order")),
            Verdict::allowed("add", 2, Reason::fluent(builder.clone())),
            Verdict::allowed("add", 2, Reason::fluent(builder)),
        ]
    );
}

#[test]
fn several_variable_arity_overloads_that_all_fit_leave_the_return_unresolved() {
    let print = Method::new("Shipping", "print", 1).call(
        Call::new(
            Expression::field("printer", TypeRef::class("Printer")),
            "print",
            2,
        )
        .with(string_parameter("title"))
        .with(Expression::parameter("first", order()))
        .with(Expression::parameter("second", order()))
        .then("withdraw", 2),
    );

    let verdicts = print.verdicts(&with_helpers(streams(stream_of(variable("T")))));

    assert_eq!(
        verdicts[1],
        Verdict::violation(
            "withdraw",
            2,
            Reason::unresolved_type(None, Reason::return_of("print"))
        )
    );
}

#[test]
fn the_variable_arity_overload_whose_element_type_fits_the_arguments_decides_the_return() {
    let join = Method::new("Shipping", "join", 1).call(
        Call::new(
            Expression::field("joiner", TypeRef::class("Joiner")),
            "join",
            2,
        )
        .with(string_parameter("first"))
        .with(string_parameter("second"))
        .then("withdraw", 2),
    );

    let verdicts = join.verdicts(&with_helpers(streams(stream_of(variable("T")))));

    assert_eq!(
        verdicts[1],
        Verdict::violation("withdraw", 2, Reason::return_of("join"))
    );
}

#[test]
fn one_argument_still_picks_the_fixed_arity_overload() {
    let notify = Method::new("Shipping", "notify", 1).call(
        Call::new(Expression::static_type(TypeRef::class(STREAM)), "of", 2)
            .with(Expression::parameter("order", order()))
            .then("forEach", 2)
            .with(Lambda::new(&["each"]).call(Call::new(
                Expression::return_of(Call::new(Expression::local("each"), "getCustomer", 3)),
                "rename",
                3,
            ))),
    );

    let verdicts = notify.verdicts(&streams(stream_of(TypeRef::class("Wallet"))));

    assert_eq!(
        verdicts[2],
        Verdict::violation("rename", 3, Reason::return_of("getCustomer"))
    );
}
