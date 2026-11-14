use super::{Call, Expression, Method, Reason, Verdict};

#[test]
fn calling_a_method_on_a_returned_object_is_a_violation() {
    let pay = Method::new().call(
        Call::new(Expression::parameter("order"), "getCustomer")
            .then("getWallet")
            .then("withdraw"),
    );

    let verdicts = pay.verdicts();

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", Reason::parameter("order")),
            Verdict::violation("getWallet", Reason::return_of("getCustomer")),
            Verdict::violation("withdraw", Reason::return_of("getWallet")),
        ]
    );
}

#[test]
fn calling_methods_on_this_and_own_fields_is_allowed() {
    let pay = Method::new()
        .call(Call::new(Expression::field("wallet"), "withdraw"))
        .call(Call::new(Expression::This, "audit"));

    let verdicts = pay.verdicts();

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("withdraw", Reason::field("wallet")),
            Verdict::allowed("audit", Reason::This),
        ]
    );
}
