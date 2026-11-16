use super::{Call, Creation, Expression, Lambda, Method, MethodReference, Reason, Verdict};
use crate::domain::classification::{Classification, Types, sample_types};
use crate::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

const STREAM: &str = "java.util.stream.Stream";
const LIST: &str = "java.util.List";
const BIG_DECIMAL: &str = "java.math.BigDecimal";

fn class(name: &str) -> TypeShape {
    TypeShape::new(name, TypeKind::Class)
}

fn interface(name: &str) -> TypeShape {
    TypeShape::new(name, TypeKind::Interface).with_type_parameter("T")
}

fn generic(name: &str, arguments: &[TypeRef]) -> TypeRef {
    TypeRef::generic(name, arguments.to_vec())
}

fn variable(name: &str) -> TypeRef {
    TypeRef::variable(name)
}

fn stream_of(element: TypeRef) -> TypeRef {
    generic(STREAM, &[element])
}

fn order() -> TypeRef {
    TypeRef::class("Order")
}

fn customer() -> TypeRef {
    TypeRef::class("Customer")
}

fn receipt() -> TypeRef {
    TypeRef::class("Receipt")
}

fn wallet() -> TypeRef {
    TypeRef::class("Wallet")
}

fn shop() -> Types {
    sample_types()
        .with_shape(interface(STREAM))
        .with_shape(interface(LIST))
        .with_shape(
            class("Customers").with_method(
                MethodShape::new("of")
                    .as_static()
                    .with_parameters(vec![order()])
                    .returning(customer()),
            ),
        )
        .with_shape(class("Registry").with_method(MethodShape::new("lookup").returning(customer())))
        .with_shape(
            class("Order")
                .with_method(MethodShape::new("getCustomer").returning(customer()))
                .with_method(MethodShape::new("getTotal").returning(TypeRef::class(BIG_DECIMAL)))
                .with_method(MethodShape::new("isPaid").returning(TypeRef::primitive("boolean")))
                .with_method(MethodShape::new("ship"))
                .with_method(MethodShape::new("newReceipt").returning(receipt())),
        )
        .with_shape(class("Receipt").with_method(MethodShape::new("print")))
        .with_shape(class("Mapper").with_method(MethodShape::new("mapAll")))
        .with_shape(
            class("Customer")
                .with_method(MethodShape::new("rename"))
                .with_method(MethodShape::new("getWallet").returning(wallet())),
        )
        .with_shape(class("Wallet").with_method(MethodShape::new("withdraw")))
}

fn orders_stream() -> Call {
    let orders = Expression::parameter("orders", generic(LIST, &[order()]));
    Call::new(orders, "stream", 2).returning(stream_of(order()))
}

fn getting_the_customer_of(variable: &str, line: usize) -> Call {
    Call::new(Expression::local(variable), "getCustomer", line).returning(customer())
}

fn customer_of(variable: &str, line: usize) -> Call {
    Call::new(
        Expression::return_of(getting_the_customer_of(variable, line)),
        "rename",
        line,
    )
}

fn withdrawing_from_the_wallet_of(variable: &str) -> Lambda {
    Lambda::new(&[variable])
        .fed_by_receiver(variable)
        .call(Call::new(
            Expression::return_of(
                Call::new(Expression::local(variable), "getWallet", 3).returning(wallet()),
            ),
            "withdraw",
            3,
        ))
}

#[test]
fn the_expression_a_lambda_yields_is_judged_like_a_statement() {
    let charge =
        Method::new("Billing", "charge", 1).call(orders_stream().then("forEach", 2).with(
            Lambda::new(&["order"]).yielding(Expression::return_of(customer_of("order", 3))),
        ));

    let verdicts = charge.verdicts(&shop());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 2, Reason::parameter("orders")),
            Verdict::allowed(
                "getCustomer",
                3,
                Reason::lambda_parameter("order", Reason::parameter("orders"))
            ),
            Verdict::violation("rename", 3, Reason::return_of("getCustomer")),
            Verdict::allowed("forEach", 2, Reason::parameter("orders")),
        ]
    );
}

#[test]
fn a_stream_mapped_by_a_lambda_hands_the_returned_objects_to_the_next_lambda() {
    let charge = Method::new("Billing", "charge", 1).call(
        orders_stream()
            .then("map", 2)
            .with(
                Lambda::new(&["order"])
                    .fed_by_elements("order")
                    .yielding(Expression::return_of(getting_the_customer_of("order", 2))),
            )
            .returning(stream_of(customer()))
            .returning_from(&[0])
            .then("forEach", 3)
            .with(withdrawing_from_the_wallet_of("customer")),
    );

    let verdicts = charge.verdicts(&shop());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 2, Reason::parameter("orders")),
            Verdict::allowed(
                "getCustomer",
                2,
                Reason::lambda_parameter("order", Reason::parameter("orders"))
            ),
            Verdict::allowed("map", 2, Reason::parameter("orders")),
            Verdict::violation(
                "getWallet",
                3,
                Reason::lambda_parameter("customer", Reason::return_of("getCustomer"))
            ),
            Verdict::violation("withdraw", 3, Reason::return_of("getWallet")),
            Verdict::allowed(
                "forEach",
                3,
                Reason::classified(Classification::Container, stream_of(customer()))
            ),
        ]
    );
}

fn mapping_orders_with(reference: MethodReference) -> Method {
    Method::new("Billing", "charge", 1).call(
        orders_stream()
            .then("map", 2)
            .with(reference.fed_by_elements())
            .returning(stream_of(customer()))
            .returning_from(&[0])
            .then("forEach", 3)
            .with(withdrawing_from_the_wallet_of("customer")),
    )
}

fn charging_the_customers_a_stranger_returned(call: &str) -> [Verdict; 5] {
    [
        Verdict::allowed("stream", 2, Reason::parameter("orders")),
        Verdict::allowed("map", 2, Reason::parameter("orders")),
        Verdict::violation(
            "getWallet",
            3,
            Reason::lambda_parameter("customer", Reason::return_of(call)),
        ),
        Verdict::violation("withdraw", 3, Reason::return_of("getWallet")),
        Verdict::allowed(
            "forEach",
            3,
            Reason::classified(Classification::Container, stream_of(customer())),
        ),
    ]
}

fn charging_the_customers_a_friend_made(made: Reason) -> [Verdict; 5] {
    [
        Verdict::allowed("stream", 2, Reason::parameter("orders")),
        Verdict::allowed("map", 2, Reason::parameter("orders")),
        Verdict::allowed(
            "getWallet",
            3,
            Reason::lambda_parameter("customer", made.clone()),
        ),
        Verdict::violation("withdraw", 3, Reason::return_of("getWallet")),
        Verdict::allowed("forEach", 3, made),
    ]
}

#[test]
fn a_method_reference_to_an_instance_method_yields_what_the_method_returns() {
    let charge = mapping_orders_with(
        MethodReference::new(Expression::static_type(order()), "getCustomer", 2).taking(1),
    );

    let verdicts = charge.verdicts(&shop());

    let mut expected = charging_the_customers_a_stranger_returned("getCustomer").to_vec();
    expected.insert(
        1,
        Verdict::allowed(
            "getCustomer",
            2,
            Reason::method_reference(order(), "getCustomer", Reason::parameter("orders")),
        ),
    );
    assert_eq!(verdicts, expected);
}

#[test]
fn a_constructor_reference_yields_the_type_it_creates() {
    let charge = mapping_orders_with(
        MethodReference::new(Expression::static_type(customer()), "new", 2).taking(1),
    );

    let verdicts = charge.verdicts(&shop());

    assert_eq!(
        verdicts,
        charging_the_customers_a_friend_made(Reason::created(customer()))
    );
}

#[test]
fn a_method_reference_to_a_static_method_yields_what_the_method_returns() {
    let charge = mapping_orders_with(
        MethodReference::new(
            Expression::static_type(TypeRef::class("Customers")),
            "of",
            2,
        )
        .taking(1)
        .returning(customer()),
    );

    let verdicts = charge.verdicts(&shop());

    let mut expected = charging_the_customers_a_friend_made(Reason::static_result("of")).to_vec();
    expected.insert(
        1,
        Verdict::allowed("of", 2, Reason::static_type(TypeRef::class("Customers"))),
    );
    assert_eq!(verdicts, expected);
}

#[test]
fn a_method_reference_on_an_expression_yields_what_the_method_of_its_type_returns() {
    let charge = mapping_orders_with(
        MethodReference::new(
            Expression::field("registry", TypeRef::class("Registry")),
            "lookup",
            2,
        )
        .taking(1),
    );

    let verdicts = charge.verdicts(&shop());

    let mut expected = charging_the_customers_a_stranger_returned("lookup").to_vec();
    expected.insert(1, Verdict::allowed("lookup", 2, Reason::field("registry")));
    assert_eq!(verdicts, expected);
}

#[test]
fn a_stream_mapped_to_created_objects_keeps_them_friends() {
    let created = Expression::created(Creation::new(receipt()).with(Expression::local("order")));
    let print = Method::new("Billing", "print", 1).call(
        orders_stream()
            .then("map", 2)
            .with(Lambda::new(&["order"]).yielding(created))
            .returning(stream_of(receipt()))
            .returning_from(&[0])
            .then("forEach", 3)
            .with(Lambda::new(&["receipt"]).call(Call::new(
                Expression::local("receipt"),
                "print",
                3,
            ))),
    );

    let verdicts = print.verdicts(&shop());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 2, Reason::parameter("orders")),
            Verdict::allowed("map", 2, Reason::parameter("orders")),
            Verdict::allowed(
                "print",
                3,
                Reason::lambda_parameter("receipt", Reason::created(receipt()))
            ),
            Verdict::allowed("forEach", 3, Reason::created(receipt())),
        ]
    );
}

#[test]
fn a_filtered_stream_keeps_the_standing_of_its_receiver() {
    let is_paid = Expression::return_of(Call::new(Expression::local("order"), "isPaid", 2));
    let ship = Method::new("Billing", "ship", 1).call(
        orders_stream()
            .then("filter", 2)
            .with(Lambda::new(&["order"]).yielding(is_paid))
            .returning(stream_of(order()))
            .then("forEach", 3)
            .with(Lambda::new(&["order"]).call(Call::new(Expression::local("order"), "ship", 3))),
    );

    let verdicts = ship.verdicts(&shop());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 2, Reason::parameter("orders")),
            Verdict::allowed(
                "isPaid",
                2,
                Reason::lambda_parameter("order", Reason::parameter("orders"))
            ),
            Verdict::allowed("filter", 2, Reason::parameter("orders")),
            Verdict::allowed(
                "ship",
                3,
                Reason::lambda_parameter("order", Reason::parameter("orders"))
            ),
            Verdict::allowed("forEach", 3, Reason::parameter("orders")),
        ]
    );
}

#[test]
fn a_value_mapped_from_a_stream_is_allowed_by_its_classification() {
    let total = Expression::return_of(
        Call::new(Expression::local("order"), "getTotal", 2).returning(TypeRef::class(BIG_DECIMAL)),
    );
    let check = Method::new("Billing", "check", 1).call(
        orders_stream()
            .then("map", 2)
            .with(
                Lambda::new(&["order"])
                    .fed_by_elements("order")
                    .yielding(total),
            )
            .returning(stream_of(TypeRef::class(BIG_DECIMAL)))
            .returning_from(&[0])
            .then("forEach", 3)
            .with(
                Lambda::new(&["total"])
                    .fed_by_receiver("total")
                    .call(Call::new(Expression::local("total"), "signum", 3)),
            ),
    );

    let verdicts = check.verdicts(&shop());

    assert_eq!(
        verdicts[3],
        Verdict::allowed(
            "signum",
            3,
            Reason::classified(Classification::Value, TypeRef::class(BIG_DECIMAL))
        )
    );
}

#[test]
fn a_call_returning_a_non_container_keeps_the_standing_of_its_container_receiver() {
    let rename = Method::new("Billing", "rename", 1).call(Call::new(
        Expression::return_of(
            orders_stream()
                .then("fold", 2)
                .with(
                    Lambda::new(&["order"])
                        .yielding(Expression::return_of(getting_the_customer_of("order", 2))),
                )
                .returning(customer())
                .returning_from(&[0]),
        ),
        "rename",
        3,
    ));

    let verdicts = rename.verdicts(&shop());

    assert_eq!(
        verdicts[3],
        Verdict::allowed("rename", 3, Reason::parameter("orders"))
    );
}

#[test]
fn a_container_a_call_on_a_non_container_returns_follows_the_usual_standing_rules() {
    let created = Expression::created(Creation::new(receipt()).with(Expression::local("order")));
    let print = Method::new("Billing", "print", 1).call(
        Call::new(
            Expression::field("mapper", TypeRef::class("Mapper")),
            "mapAll",
            2,
        )
        .with(
            Lambda::new(&["order"])
                .fed_by_receiver("order")
                .yielding(created),
        )
        .returning(stream_of(receipt()))
        .returning_from(&[0])
        .then("forEach", 3)
        .with(
            Lambda::new(&["receipt"])
                .fed_by_receiver("receipt")
                .call(Call::new(Expression::local("receipt"), "print", 3)),
        ),
    );

    let verdicts = print.verdicts(&shop());

    assert_eq!(
        verdicts[1],
        Verdict::violation(
            "print",
            3,
            Reason::lambda_parameter("receipt", Reason::return_of("mapAll"))
        )
    );
}

fn orders_then_print(reference: MethodReference) -> Method {
    Method::new("Billing", "print", 1).call(
        orders_stream()
            .then("map", 2)
            .with(reference.fed_by_elements())
            .returning(stream_of(variable("R")))
            .returning_from(&[0])
            .then("forEach", 3)
            .with(
                Lambda::new(&["receipt"])
                    .fed_by_receiver("receipt")
                    .call(Call::new(Expression::local("receipt"), "print", 3)),
            ),
    )
}

#[test]
fn a_method_reference_to_a_factory_method_of_a_friend_yields_friends() {
    let print = orders_then_print(
        MethodReference::new(Expression::static_type(order()), "newReceipt", 2).taking(1),
    );

    let verdicts = print.verdicts(&shop());

    assert_eq!(
        verdicts[3],
        Verdict::allowed(
            "print",
            3,
            Reason::lambda_parameter("receipt", Reason::factory_result("newReceipt"))
        )
    );
}

#[test]
fn a_method_reference_to_an_unknown_method_yields_strangers() {
    let print = orders_then_print(
        MethodReference::new(Expression::static_type(order()), "missing", 2).taking(1),
    );

    let verdicts = print.verdicts(&shop());

    assert_eq!(
        verdicts[3],
        Verdict::violation(
            "print",
            3,
            Reason::unresolved_type(
                Some(variable("R")),
                Reason::lambda_parameter("receipt", Reason::return_of("missing"))
            )
        )
    );
}
