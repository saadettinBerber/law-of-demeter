use super::{Call, Expression, Method, Reason, Verdict};
use crate::domain::classification::{Types, sample_types};
use crate::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

const STRING: &str = "java.lang.String";

fn builder() -> TypeRef {
    TypeRef::class("Builder")
}

fn builders() -> Types {
    let add = MethodShape::new("add")
        .as_varargs()
        .with_parameters(vec![TypeRef::array(TypeRef::class(STRING))])
        .returning(builder());
    sample_types().with_shape(TypeShape::new("Builder", TypeKind::Class).with_method(add))
}

fn string_parameter(name: &str) -> Expression {
    Expression::parameter(name, TypeRef::class(STRING))
}

#[test]
fn variable_arity_fluent_calls_on_a_stranger_remain_violations() {
    let build = Method::new("Shipping", "build", 1).call(
        Call::new(
            Expression::parameter("order", TypeRef::class("Order")),
            "getBuilder",
            2,
        )
        .returning(builder())
        .then("add", 2)
        .with(string_parameter("first"))
        .with(string_parameter("second"))
        .returning(builder())
        .returning_receiver()
        .then("add", 2)
        .with(string_parameter("third")),
    );

    let verdicts = build.verdicts(&builders());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getBuilder", 2, Reason::parameter("order")),
            Verdict::violation("add", 2, Reason::return_of("getBuilder")),
            Verdict::violation("add", 2, Reason::return_of("getBuilder")),
        ]
    );
}
