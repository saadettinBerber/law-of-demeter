use super::nested::{
    collecting, collector, collectors_call, collectors_static, customer, generic, interface,
    list_of, order, orders_collected_by, orders_stream, stranger_customers, stream_of, variable,
    wildcard,
};
use super::{Call, Expression, Lambda, Method, MethodReference, Reason, Verdict};
use crate::domain::classification::Types;
use crate::domain::type_shape::{MethodShape, TypeRef};

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

#[test]
fn a_nested_call_with_a_resolved_return_is_judged_once_among_the_elements_of_the_enclosing_call() {
    let each = Lambda::new(&["each"])
        .resolving("each", order())
        .fed_by_elements("each")
        .yielding(wallet_of_each());
    let grouping = collectors_call("groupingBy")
        .returning(collector(order(), wildcard(), list_of(order())))
        .with(each);
    let group = Method::new("Billing", "group", 1).call(orders_collected_by(grouping));

    let verdicts = group.verdicts(&collecting());

    assert!(verdicts.contains(&Verdict::allowed(
        "getWallet",
        3,
        Reason::lambda_parameter("each", Reason::parameter("orders"))
    )));
}

fn keyed() -> Types {
    let keys = generic("java.util.Set", &[variable("K")]);
    collecting()
        .with_shape(
            interface("java.util.Map", &["K", "V"])
                .with_method(MethodShape::new("keySet").returning(keys)),
        )
        .with_shape(interface("java.util.Set", &["E"]))
}

fn orders_by_customer() -> TypeRef {
    generic("java.util.Map", &[customer(), list_of(order())])
}

fn grouping_by_customer() -> Call {
    collectors_call("groupingBy")
        .with(MethodReference::new(
            Expression::static_type(order()),
            "getCustomer",
            2,
        ))
        .returning(orders_by_customer())
        .returning_from(&[])
}

fn withdrawing_from_each_of(customers: Call) -> Method {
    Method::new("Billing", "charge", 1)
        .for_each("customer", Expression::return_of(customers))
        .call(Call::new(Expression::local("customer"), "getWallet", 3))
}

fn wallet_of_a_stranger_loop_variable() -> Verdict {
    Verdict::violation(
        "getWallet",
        3,
        Reason::loop_variable("customer", Reason::return_of("getCustomer")),
    )
}

#[test]
fn a_call_reading_the_keys_takes_the_standing_the_link_gave_them() {
    let keys = grouping_by_customer()
        .keyed_by(0)
        .then("keySet", 2)
        .yielding_keys();

    let verdicts = withdrawing_from_each_of(keys).verdicts(&keyed());

    assert_eq!(verdicts.last(), Some(&wallet_of_a_stranger_loop_variable()));
}

#[test]
fn a_call_that_does_not_read_the_keys_keeps_the_standing_of_its_receiver() {
    let keys = grouping_by_customer().keyed_by(0).then("keySet", 2);

    let verdicts = withdrawing_from_each_of(keys).verdicts(&keyed());

    assert_eq!(
        verdicts.last(),
        Some(&Verdict::allowed(
            "getWallet",
            3,
            Reason::loop_variable("customer", Reason::static_result("groupingBy"))
        ))
    );
}

#[test]
fn a_call_passes_on_the_keys_of_the_argument_the_link_names() {
    let grouped = orders_collected_by(grouping_by_customer().keyed_by(0))
        .returning(orders_by_customer())
        .keyed_by(0);

    let verdicts =
        withdrawing_from_each_of(grouped.then("keySet", 2).yielding_keys()).verdicts(&keyed());

    assert_eq!(verdicts.last(), Some(&wallet_of_a_stranger_loop_variable()));
}
