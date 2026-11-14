use super::{Call, Creation, Expression, Lambda, Method, Reason, Verdict};

#[test]
fn calling_a_method_on_a_returned_object_is_a_violation() {
    let pay = Method::new("OrderService", "pay", 1).call(
        Call::new(Expression::parameter("order"), "getCustomer", 1)
            .then("getWallet", 1)
            .then("withdraw", 1),
    );

    let verdicts = pay.verdicts();

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 1, Reason::parameter("order")),
            Verdict::violation("getWallet", 1, Reason::return_of("getCustomer")),
            Verdict::violation("withdraw", 1, Reason::return_of("getWallet")),
        ]
    );
}

#[test]
fn calling_methods_on_this_and_own_fields_is_allowed() {
    let pay = Method::new("OrderService", "pay", 1)
        .call(Call::new(Expression::field("wallet"), "withdraw", 1))
        .call(Call::new(Expression::This, "audit", 1));

    let verdicts = pay.verdicts();

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("withdraw", 1, Reason::field("wallet")),
            Verdict::allowed("audit", 1, Reason::This),
        ]
    );
}

#[test]
fn passing_a_returned_object_as_an_argument_is_allowed() {
    let notify_customer =
        Method::new("OrderService", "pay", 1).call(
            Call::new(Expression::field("notifier"), "send", 1).with(Expression::return_of(
                Call::new(Expression::parameter("order"), "getCustomer", 1),
            )),
        );

    let verdicts = notify_customer.verdicts();

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 1, Reason::parameter("order")),
            Verdict::allowed("send", 1, Reason::field("notifier")),
        ]
    );
}

#[test]
fn calling_a_setter_on_a_returned_object_is_a_violation() {
    let move_customer = Method::new("OrderService", "pay", 1).call(
        Call::new(Expression::parameter("order"), "getCustomer", 1)
            .then("setAddress", 1)
            .with(Expression::parameter("address")),
    );

    let verdicts = move_customer.verdicts();

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 1, Reason::parameter("order")),
            Verdict::violation("setAddress", 1, Reason::return_of("getCustomer")),
        ]
    );
}

#[test]
fn calling_a_mutator_on_a_returned_collection_is_a_violation() {
    let add_line = Method::new("OrderService", "pay", 1).call(
        Call::new(Expression::parameter("order"), "getLines", 1)
            .then("add", 1)
            .with(Expression::parameter("line")),
    );

    let verdicts = add_line.verdicts();

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getLines", 1, Reason::parameter("order")),
            Verdict::violation("add", 1, Reason::return_of("getLines")),
        ]
    );
}

#[test]
fn calling_a_method_on_an_unassigned_local_is_a_violation() {
    let pay = Method::new("OrderService", "pay", 1).call(Call::new(
        Expression::local("customer"),
        "getWallet",
        1,
    ));

    let verdicts = pay.verdicts();

    assert_eq!(
        verdicts,
        [Verdict::violation(
            "getWallet",
            1,
            Reason::unassigned_local("customer")
        )]
    );
}

#[test]
fn calling_a_method_on_a_local_holding_a_returned_object_is_a_violation() {
    let pay = Method::new("OrderService", "pay", 1)
        .assign(
            "customer",
            Expression::return_of(Call::new(Expression::parameter("order"), "getCustomer", 1)),
        )
        .call(Call::new(Expression::local("customer"), "getWallet", 1).then("withdraw", 1));

    let verdicts = pay.verdicts();

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 1, Reason::parameter("order")),
            Verdict::violation(
                "getWallet",
                1,
                Reason::local("customer", Reason::return_of("getCustomer"))
            ),
            Verdict::violation("withdraw", 1, Reason::return_of("getWallet")),
        ]
    );
}

#[test]
fn calls_inside_an_assigned_value_are_judged_once_however_often_the_local_is_used() {
    let inspect = Method::new("OrderService", "pay", 1)
        .assign(
            "customer",
            Expression::return_of(Call::new(Expression::parameter("order"), "getCustomer", 1)),
        )
        .call(Call::new(Expression::local("customer"), "getWallet", 1))
        .call(Call::new(Expression::local("customer"), "getAddress", 1));

    let verdicts = inspect.verdicts();

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 1, Reason::parameter("order")),
            Verdict::violation(
                "getWallet",
                1,
                Reason::local("customer", Reason::return_of("getCustomer"))
            ),
            Verdict::violation(
                "getAddress",
                1,
                Reason::local("customer", Reason::return_of("getCustomer"))
            ),
        ]
    );
}

#[test]
fn calling_a_method_on_a_local_holding_a_parameter_is_allowed() {
    let pay = Method::new("OrderService", "pay", 1)
        .assign("current", Expression::parameter("order"))
        .call(
            Call::new(Expression::local("current"), "charge", 1)
                .with(Expression::parameter("amount")),
        );

    let verdicts = pay.verdicts();

    assert_eq!(
        verdicts,
        [Verdict::allowed(
            "charge",
            1,
            Reason::local("current", Reason::parameter("order"))
        )]
    );
}

#[test]
fn calling_a_method_on_a_lambda_parameter_fed_by_a_returned_collection_is_a_violation() {
    let discount = Method::new("OrderService", "pay", 1).call(
        Call::new(Expression::parameter("order"), "getLines", 1)
            .then("forEach", 1)
            .with(
                Lambda::new(&["line"]).call(
                    Call::new(Expression::local("line"), "applyDiscount", 1)
                        .with(Expression::parameter("rate")),
                ),
            ),
    );

    let verdicts = discount.verdicts();

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getLines", 1, Reason::parameter("order")),
            Verdict::violation(
                "applyDiscount",
                1,
                Reason::lambda_parameter("line", Reason::return_of("getLines"))
            ),
            Verdict::violation("forEach", 1, Reason::return_of("getLines")),
        ]
    );
}

#[test]
fn calling_a_method_on_a_lambda_parameter_fed_by_an_own_field_is_allowed() {
    let ship_all = Method::new("OrderService", "pay", 1).call(
        Call::new(Expression::field("orders"), "forEach", 1)
            .with(Lambda::new(&["order"]).call(Call::new(Expression::local("order"), "ship", 1))),
    );

    let verdicts = ship_all.verdicts();

    assert_eq!(
        verdicts,
        [
            Verdict::allowed(
                "ship",
                1,
                Reason::lambda_parameter("order", Reason::field("orders"))
            ),
            Verdict::allowed("forEach", 1, Reason::field("orders")),
        ]
    );
}

#[test]
fn calling_a_method_on_an_object_created_in_the_method_is_allowed() {
    let greeting = Method::new("OrderService", "pay", 1).call(Call::new(
        Expression::created(Creation::new("StringBuilder")),
        "append",
        1,
    ));

    let verdicts = greeting.verdicts();

    assert_eq!(
        verdicts,
        [Verdict::allowed(
            "append",
            1,
            Reason::created("StringBuilder")
        )]
    );
}

#[test]
fn calls_inside_constructor_arguments_are_judged() {
    let print = Method::new("OrderService", "pay", 1).call(Call::new(
        Expression::created(Creation::new("Receipt").with(Expression::return_of(
            Call::new(Expression::parameter("order"), "getCustomer", 1).then("getName", 1),
        ))),
        "print",
        1,
    ));

    let verdicts = print.verdicts();

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 1, Reason::parameter("order")),
            Verdict::violation("getName", 1, Reason::return_of("getCustomer")),
            Verdict::allowed("print", 1, Reason::created("Receipt")),
        ]
    );
}

#[test]
fn calling_a_static_method_on_a_type_is_allowed() {
    let create = Method::new("OrderService", "pay", 1).call(Call::new(
        Expression::static_type("Order"),
        "builder",
        1,
    ));

    let verdicts = create.verdicts();

    assert_eq!(
        verdicts,
        [Verdict::allowed("builder", 1, Reason::static_type("Order"))]
    );
}

#[test]
fn a_lambda_body_can_assign_a_local_and_call_it() {
    let inspect_all = Method::new("OrderService", "pay", 1).call(
        Call::new(Expression::field("orders"), "forEach", 1).with(
            Lambda::new(&["order"])
                .assign(
                    "customer",
                    Expression::return_of(Call::new(Expression::local("order"), "getCustomer", 1)),
                )
                .call(Call::new(Expression::local("customer"), "getWallet", 1)),
        ),
    );

    let verdicts = inspect_all.verdicts();

    assert_eq!(
        verdicts,
        [
            Verdict::allowed(
                "getCustomer",
                1,
                Reason::lambda_parameter("order", Reason::field("orders"))
            ),
            Verdict::violation(
                "getWallet",
                1,
                Reason::local("customer", Reason::return_of("getCustomer"))
            ),
            Verdict::allowed("forEach", 1, Reason::field("orders")),
        ]
    );
}

#[test]
fn a_loop_variable_takes_the_standing_of_the_iterable_and_judges_it_once() {
    let discount = Method::new("OrderService", "pay", 1)
        .for_each(
            "line",
            Expression::return_of(Call::new(Expression::parameter("order"), "getLines", 1)),
        )
        .call(
            Call::new(Expression::local("line"), "applyDiscount", 1)
                .with(Expression::parameter("rate")),
        );

    let verdicts = discount.verdicts();

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getLines", 1, Reason::parameter("order")),
            Verdict::violation(
                "applyDiscount",
                1,
                Reason::loop_variable("line", Reason::return_of("getLines"))
            ),
        ]
    );
}

#[test]
fn every_verdict_carries_the_line_of_its_call() {
    let pay = Method::new("OrderService", "pay", 10)
        .call(
            Call::new(Expression::parameter("order"), "getCustomer", 11)
                .then("getWallet", 12)
                .then("withdraw", 13),
        )
        .call(Call::new(Expression::This, "audit", 14));

    let verdicts = pay.verdicts();

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 11, Reason::parameter("order")),
            Verdict::violation("getWallet", 12, Reason::return_of("getCustomer")),
            Verdict::violation("withdraw", 13, Reason::return_of("getWallet")),
            Verdict::allowed("audit", 14, Reason::This),
        ]
    );
}

#[test]
fn a_method_knows_its_class_its_name_and_its_line() {
    let pay = Method::new("com.shop.OrderService$Payments", "pay", 42);

    assert_eq!(
        (pay.class_name(), pay.name(), pay.line()),
        ("com.shop.OrderService$Payments", "pay", 42)
    );
}
