use super::{Call, Expression, Method, Reason, Verdict};
use crate::domain::classification::{Classification, Types, sample_types};
use crate::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

const OBJECT: &str = "java.lang.Object";
const BIG_DECIMAL: &str = "java.math.BigDecimal";

fn class(name: &str) -> TypeShape {
    TypeShape::new(name, TypeKind::Class)
}

fn order() -> TypeRef {
    TypeRef::class("Order")
}

fn shop() -> Types {
    sample_types().with_shape(
        class("Order").with_method(MethodShape::new("getTotal").returning(TypeRef::class(OBJECT))),
    )
}

#[test]
fn a_call_takes_its_resolved_return_type_over_the_one_its_shape_declares() {
    let charge = Method::new("Checkout", "charge", 1).call(Call::new(
        Expression::return_of(
            Call::new(Expression::parameter("order", order()), "getTotal", 2)
                .returning(TypeRef::class(BIG_DECIMAL)),
        ),
        "signum",
        2,
    ));

    let verdicts = charge.verdicts(&shop());

    assert_eq!(
        verdicts[1],
        Verdict::allowed(
            "signum",
            2,
            Reason::classified(Classification::Value, TypeRef::class(BIG_DECIMAL))
        )
    );
}

#[test]
fn a_call_takes_its_resolved_return_type_when_its_shape_declares_none() {
    let charge = Method::new("Checkout", "charge", 1).call(Call::new(
        Expression::return_of(
            Call::new(
                Expression::parameter("gateway", TypeRef::class("Gateway")),
                "quote",
                2,
            )
            .returning(TypeRef::class(BIG_DECIMAL)),
        ),
        "signum",
        2,
    ));

    let verdicts = charge.verdicts(&shop());

    assert_eq!(
        verdicts[1],
        Verdict::allowed(
            "signum",
            2,
            Reason::classified(Classification::Value, TypeRef::class(BIG_DECIMAL))
        )
    );
}
