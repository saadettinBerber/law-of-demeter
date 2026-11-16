use super::nested::{
    collecting, collectors_call, collectors_static, customer, order, orders_collected_by,
    orders_stream, stranger_customers, stream_of,
};
use super::{Call, Expression, Lambda, Method, MethodReference, Reason, Verdict};

fn wallet_of_each() -> Expression {
    Expression::return_of(Call::new(Expression::local("each"), "getWallet", 3))
}

fn orders_grouped_by(argument: impl Into<super::Argument>) -> Method {
    Method::new("Billing", "group", 1).call(orders_collected_by(
        collectors_call("groupingBy").with(argument),
    ))
}

fn orders_finished_by(argument: impl Into<super::Argument>) -> Method {
    let wrapping = collectors_call("collectingAndThen")
        .with(Expression::return_of(collectors_call("toList")))
        .with(argument);
    Method::new("Billing", "finish", 1).call(orders_collected_by(wrapping))
}

#[test]
fn a_lambda_parameter_linked_to_the_elements_takes_their_standing_though_its_type_differs() {
    let each = Lambda::new(&["each"])
        .resolving("each", customer())
        .fed_by_elements("each")
        .yielding(wallet_of_each());

    let verdicts = orders_grouped_by(each).verdicts(&collecting());

    assert!(verdicts.contains(&Verdict::allowed(
        "getWallet",
        3,
        Reason::lambda_parameter("each", Reason::parameter("orders"))
    )));
}

#[test]
fn a_lambda_parameter_linked_to_the_receiver_keeps_its_own_feed_though_its_type_matches_the_elements()
 {
    let each = Lambda::new(&["each"])
        .resolving("each", order())
        .fed_by_receiver("each")
        .yielding(wallet_of_each());

    let verdicts = orders_grouped_by(each).verdicts(&collecting());

    assert!(verdicts.contains(&Verdict::allowed(
        "getWallet",
        3,
        Reason::lambda_parameter("each", collectors_static())
    )));
}

#[test]
fn a_method_reference_linked_to_the_elements_takes_their_standing_though_its_type_differs() {
    let wallet =
        MethodReference::new(Expression::static_type(customer()), "getWallet", 3).fed_by_elements();

    let verdicts = orders_finished_by(wallet).verdicts(&collecting());

    assert!(verdicts.contains(&Verdict::allowed(
        "getWallet",
        3,
        Reason::method_reference(customer(), "getWallet", Reason::parameter("orders"))
    )));
}

#[test]
fn a_method_reference_linked_to_the_receiver_keeps_its_own_feed_though_its_type_matches_the_elements()
 {
    let wallet =
        MethodReference::new(Expression::static_type(order()), "getStatus", 3).fed_by_receiver();

    let verdicts = orders_grouped_by(wallet).verdicts(&collecting());

    assert!(verdicts.contains(&Verdict::allowed(
        "getStatus",
        3,
        Reason::method_reference(order(), "getStatus", collectors_static())
    )));
}

fn customers_of_the_orders() -> Lambda {
    Lambda::new(&["each"])
        .resolving("each", order())
        .yielding(Expression::return_of(Call::new(
            Expression::local("each"),
            "getCustomer",
            2,
        )))
}

fn wallet_of_the_first_customer_in(customers: Call) -> Method {
    Method::new("Billing", "charge", 1).call(
        customers
            .then("collect", 2)
            .with(Expression::return_of(collectors_call("toList")))
            .then("get", 2)
            .with(Expression::literal())
            .then("getWallet", 3),
    )
}

#[test]
fn a_result_the_link_keeps_out_of_the_return_leaves_the_standing_of_the_receiver() {
    let customers = orders_stream()
        .then("map", 2)
        .with(customers_of_the_orders())
        .returning_from(&[]);

    let verdicts = wallet_of_the_first_customer_in(customers).verdicts(&collecting());

    assert_eq!(
        verdicts.last(),
        Some(&Verdict::allowed(
            "getWallet",
            3,
            Reason::parameter("orders")
        ))
    );
}

#[test]
fn a_result_the_link_feeds_into_the_return_gives_it_its_standing() {
    let customers = orders_stream()
        .then("transform", 2)
        .with(customers_of_the_orders())
        .returning(stream_of(customer()))
        .returning_from(&[0]);

    let verdicts = wallet_of_the_first_customer_in(customers).verdicts(&collecting());

    assert_eq!(
        verdicts.last(),
        Some(&Verdict::violation("getWallet", 3, stranger_customers()))
    );
}
