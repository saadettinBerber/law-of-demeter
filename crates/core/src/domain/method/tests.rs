use super::{Call, Expression, Lambda, Method, Reason, Verdict};

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

#[test]
fn passing_a_returned_object_as_an_argument_is_allowed() {
    let notify_customer =
        Method::new().call(Call::new(Expression::field("notifier"), "send").with(
            Expression::return_of(Call::new(Expression::parameter("order"), "getCustomer")),
        ));

    let verdicts = notify_customer.verdicts();

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", Reason::parameter("order")),
            Verdict::allowed("send", Reason::field("notifier")),
        ]
    );
}

#[test]
fn calling_a_setter_on_a_returned_object_is_a_violation() {
    let move_customer = Method::new().call(
        Call::new(Expression::parameter("order"), "getCustomer")
            .then("setAddress")
            .with(Expression::parameter("address")),
    );

    let verdicts = move_customer.verdicts();

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", Reason::parameter("order")),
            Verdict::violation("setAddress", Reason::return_of("getCustomer")),
        ]
    );
}

#[test]
fn calling_a_mutator_on_a_returned_collection_is_a_violation() {
    let add_line = Method::new().call(
        Call::new(Expression::parameter("order"), "getLines")
            .then("add")
            .with(Expression::parameter("line")),
    );

    let verdicts = add_line.verdicts();

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getLines", Reason::parameter("order")),
            Verdict::violation("add", Reason::return_of("getLines")),
        ]
    );
}

#[test]
fn calling_a_method_on_an_unassigned_local_is_a_violation() {
    let pay = Method::new().call(Call::new(Expression::local("customer"), "getWallet"));

    let verdicts = pay.verdicts();

    assert_eq!(
        verdicts,
        [Verdict::violation(
            "getWallet",
            Reason::unassigned_local("customer")
        )]
    );
}

#[test]
fn calling_a_method_on_a_local_holding_a_returned_object_is_a_violation() {
    let pay = Method::new()
        .assign(
            "customer",
            Expression::return_of(Call::new(Expression::parameter("order"), "getCustomer")),
        )
        .call(Call::new(Expression::local("customer"), "getWallet").then("withdraw"));

    let verdicts = pay.verdicts();

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", Reason::parameter("order")),
            Verdict::violation(
                "getWallet",
                Reason::local("customer", Reason::return_of("getCustomer"))
            ),
            Verdict::violation("withdraw", Reason::return_of("getWallet")),
        ]
    );
}

#[test]
fn calls_inside_an_assigned_value_are_judged_once_however_often_the_local_is_used() {
    let inspect = Method::new()
        .assign(
            "customer",
            Expression::return_of(Call::new(Expression::parameter("order"), "getCustomer")),
        )
        .call(Call::new(Expression::local("customer"), "getWallet"))
        .call(Call::new(Expression::local("customer"), "getAddress"));

    let verdicts = inspect.verdicts();

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", Reason::parameter("order")),
            Verdict::violation(
                "getWallet",
                Reason::local("customer", Reason::return_of("getCustomer"))
            ),
            Verdict::violation(
                "getAddress",
                Reason::local("customer", Reason::return_of("getCustomer"))
            ),
        ]
    );
}

#[test]
fn calling_a_method_on_a_local_holding_a_parameter_is_allowed() {
    let pay = Method::new()
        .assign("current", Expression::parameter("order"))
        .call(
            Call::new(Expression::local("current"), "charge").with(Expression::parameter("amount")),
        );

    let verdicts = pay.verdicts();

    assert_eq!(
        verdicts,
        [Verdict::allowed(
            "charge",
            Reason::local("current", Reason::parameter("order"))
        )]
    );
}

#[test]
fn calling_a_method_on_a_lambda_parameter_fed_by_a_returned_collection_is_a_violation() {
    let discount = Method::new().call(
        Call::new(Expression::parameter("order"), "getLines")
            .then("forEach")
            .with(
                Lambda::new(&["line"]).call(
                    Call::new(Expression::local("line"), "applyDiscount")
                        .with(Expression::parameter("rate")),
                ),
            ),
    );

    let verdicts = discount.verdicts();

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getLines", Reason::parameter("order")),
            Verdict::violation(
                "applyDiscount",
                Reason::lambda_parameter("line", Reason::return_of("getLines"))
            ),
            Verdict::violation("forEach", Reason::return_of("getLines")),
        ]
    );
}
