use super::{Call, Creation, Expression, Lambda, Method, Reason, Verdict};
use crate::domain::classification::Types;
use crate::domain::type_shape::TypeRef;

fn customer_of_order() -> Expression {
    Expression::return_of(Call::new(
        Expression::parameter("order", TypeRef::class("Order")),
        "getCustomer",
        2,
    ))
}

#[test]
fn evaluating_an_expression_judges_the_calls_inside_it() {
    let notify = Method::new("OrderService", "notify", 1).evaluate(Expression::created(
        Creation::new(TypeRef::class("Receipt")).with(customer_of_order()),
    ));

    let verdicts = notify.verdicts(&Types::new());

    assert_eq!(
        verdicts,
        [Verdict::allowed(
            "getCustomer",
            2,
            Reason::parameter("order")
        )]
    );
}

#[test]
fn a_lambda_body_can_evaluate_an_expression_and_loop() {
    let lines = TypeRef::generic("java.util.List", vec![TypeRef::class("OrderLine")]);
    let discount = Method::new("OrderService", "discount", 1).call(
        Call::new(Expression::parameter("batches", lines), "forEach", 2).with(
            Lambda::new(&["batch"])
                .evaluate(Expression::created(
                    Creation::new(TypeRef::class("Receipt")).with(customer_of_order()),
                ))
                .for_each(
                    "line",
                    Expression::return_of(Call::new(Expression::local("batch"), "lines", 3)),
                ),
        ),
    );

    let verdicts = discount.verdicts(&Types::new());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 2, Reason::parameter("order")),
            Verdict::allowed(
                "lines",
                3,
                Reason::lambda_parameter("batch", Reason::parameter("batches"))
            ),
            Verdict::allowed("forEach", 2, Reason::parameter("batches")),
        ]
    );
}
