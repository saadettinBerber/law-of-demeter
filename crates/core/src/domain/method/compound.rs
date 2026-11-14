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

#[test]
fn an_operation_judges_the_calls_among_its_operands_in_order() {
    let balance = |name: &str, line: usize| {
        Expression::return_of(Call::new(
            Expression::parameter("account", TypeRef::class("Account")),
            name,
            line,
        ))
    };
    let pay = Method::new("PaymentService", "pay", 1).call(
        Call::new(
            Expression::parameter("account", TypeRef::class("Account")),
            "setBalance",
            2,
        )
        .with(Expression::operation(vec![
            balance("getBalance", 2),
            Expression::parameter("amount", TypeRef::primitive("int")),
            balance("getLimit", 3),
        ])),
    );

    let verdicts = pay.verdicts(&Types::new());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getBalance", 2, Reason::parameter("account")),
            Verdict::allowed("getLimit", 3, Reason::parameter("account")),
            Verdict::allowed("setBalance", 2, Reason::parameter("account")),
        ]
    );
}

#[test]
fn the_result_of_an_operation_is_a_value_a_method_can_be_called_on() {
    let length = Method::new("Reports", "length", 1).call(Call::new(
        Expression::operation(vec![customer_of_order(), Expression::literal()]),
        "length",
        3,
    ));

    let verdicts = length.verdicts(&Types::new());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 2, Reason::parameter("order")),
            Verdict::allowed("length", 3, Reason::Literal),
        ]
    );
}
