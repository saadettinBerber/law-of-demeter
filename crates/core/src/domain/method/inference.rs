use super::{Call, Expression, Lambda, Method, Reason, Verdict};
use crate::domain::classification::Types;
use crate::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

const ARRAYS: &str = "java.util.Arrays";
const STREAM: &str = "java.util.stream.Stream";
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

fn shop() -> Types {
    Types::new()
        .with_shape(
            class(ARRAYS).with_method(
                MethodShape::new("stream")
                    .as_static()
                    .with_type_parameter("T")
                    .with_parameters(vec![TypeRef::array(variable("T"))])
                    .returning(generic(STREAM, &[variable("T")])),
            ),
        )
        .with_shape(interface(STREAM, &["T"]).with_method(
            MethodShape::new("forEach").with_parameters(vec![generic(CONSUMER, &[variable("T")])]),
        ))
        .with_shape(interface(CONSUMER, &["T"]))
        .with_shape(
            class("Order")
                .with_method(MethodShape::new("getCustomer").returning(TypeRef::class("Customer"))),
        )
        .with_shape(class("Customer").with_method(MethodShape::new("rename")))
}

fn customer_of(variable: &str, line: usize) -> Call {
    Call::new(
        Expression::return_of(Call::new(Expression::local(variable), "getCustomer", line)),
        "rename",
        line,
    )
}

#[test]
fn the_type_argument_of_a_generic_method_comes_from_an_array_argument() {
    let notify = Method::new("Shipping", "notify", 1).call(
        Call::new(
            Expression::return_of(
                Call::new(Expression::static_type(TypeRef::class(ARRAYS)), "stream", 2)
                    .with(Expression::parameter("orders", TypeRef::array(order()))),
            ),
            "forEach",
            2,
        )
        .with(Lambda::new(&["order"]).call(customer_of("order", 3))),
    );

    let verdicts = notify.verdicts(&shop());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 2, Reason::static_type(TypeRef::class(ARRAYS))),
            Verdict::allowed(
                "getCustomer",
                3,
                Reason::lambda_parameter("order", Reason::static_result("stream"))
            ),
            Verdict::violation("rename", 3, Reason::return_of("getCustomer")),
            Verdict::allowed("forEach", 2, Reason::static_result("stream")),
        ]
    );
}
