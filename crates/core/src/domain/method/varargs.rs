use super::{Call, Expression, Method, MethodReference, Reason, Verdict};
use crate::domain::classification::Types;
use crate::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

const STREAM: &str = "java.util.stream.Stream";
const OPTIONAL: &str = "java.util.Optional";
const LIST: &str = "java.util.List";
const FUNCTION: &str = "java.util.function.Function";
const CONSUMER: &str = "java.util.function.Consumer";

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
            Verdict::allowed("flatMap", 2, Reason::static_result("of")),
            Verdict::allowed("toList", 2, Reason::static_result("of")),
            Verdict::allowed("get", 2, Reason::static_result("of")),
            Verdict::allowed("getCustomer", 2, Reason::static_result("of")),
            Verdict::violation("rename", 2, Reason::return_of("getCustomer")),
        ]
    );
}
