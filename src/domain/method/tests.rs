use super::{Call, Expression, Method, Violation};

#[test]
fn calling_a_method_on_a_returned_object_is_reported() {
    let pay = Method::new().call(
        Call::new(Expression::parameter("order"), "getCustomer")
            .then("getWallet")
            .then("withdraw"),
    );

    let violations = pay.violations();

    assert_eq!(
        violations,
        [
            Violation::calling("getWallet"),
            Violation::calling("withdraw")
        ]
    );
}

#[test]
fn calling_methods_on_this_and_own_fields_is_not_reported() {
    let pay = Method::new()
        .call(Call::new(Expression::field("wallet"), "withdraw"))
        .call(Call::new(Expression::This, "audit"));

    let violations = pay.violations();

    assert_eq!(violations, []);
}
