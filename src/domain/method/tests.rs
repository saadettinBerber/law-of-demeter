use super::{Call, Method, Receiver, Violation};

#[test]
fn calling_a_method_on_a_returned_object_is_reported() {
    let pay = Method::new(vec![
        Call::new(Receiver::parameter("order"), "getCustomer")
            .then("getWallet")
            .then("withdraw"),
    ]);

    let violations = pay.violations();

    assert_eq!(
        violations,
        [
            Violation::calling("getWallet"),
            Violation::calling("withdraw")
        ]
    );
}
