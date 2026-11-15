use super::{Call, Expression, Lambda, Method, MethodReference, Reason, Verdict};
use crate::domain::classification::Types;
use crate::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

const ARRAYS: &str = "java.util.Arrays";
const STREAM: &str = "java.util.stream.Stream";
const CONSUMER: &str = "java.util.function.Consumer";
const COLLECTION: &str = "java.util.Collection";
const ARRAY_LIST: &str = "java.util.ArrayList";
const STREAMS: &str = "shop.Streams";
const LIST: &str = "java.util.List";
const FUNCTION: &str = "java.util.function.Function";
const MAP: &str = "java.util.Map";
const BI_CONSUMER: &str = "java.util.function.BiConsumer";

fn class(name: &str) -> TypeShape {
    TypeShape::new(name, TypeKind::Class)
}

fn interface(name: &str, parameters: &[&str]) -> TypeShape {
    parameters.iter().fold(
        TypeShape::new(name, TypeKind::Interface),
        |shape, parameter| shape.with_type_parameter(parameter),
    )
}

fn generic(name: &str, arguments: &[TypeRef]) -> TypeRef {
    TypeRef::generic(name, arguments.to_vec())
}

fn variable(name: &str) -> TypeRef {
    TypeRef::variable(name)
}

fn order() -> TypeRef {
    TypeRef::class("Order")
}

fn shop() -> Types {
    Types::new()
        .with_shape(
            class(ARRAYS).with_method(
                MethodShape::new("stream")
                    .as_static()
                    .with_type_parameter("T")
                    .with_parameters(vec![TypeRef::array(variable("T"))])
                    .returning(generic(STREAM, &[variable("T")])),
            ),
        )
        .with_shape(
            interface(STREAM, &["T"])
                .with_method(
                    MethodShape::new("forEach")
                        .with_parameters(vec![generic(CONSUMER, &[variable("T")])]),
                )
                .with_method(
                    MethodShape::new("map")
                        .with_type_parameter("R")
                        .with_parameters(vec![generic(FUNCTION, &[variable("T"), variable("R")])])
                        .returning(generic(STREAM, &[variable("R")])),
                ),
        )
        .with_shape(
            interface(FUNCTION, &["T", "R"]).with_method(
                MethodShape::new("apply")
                    .with_parameters(vec![variable("T")])
                    .returning(variable("R"))
                    .as_abstract(),
            ),
        )
        .with_shape(
            interface(LIST, &["E"]).with_method(
                MethodShape::new("stream").returning(generic(STREAM, &[variable("E")])),
            ),
        )
        .with_shape(
            interface(CONSUMER, &["T"]).with_method(
                MethodShape::new("accept")
                    .with_parameters(vec![variable("T")])
                    .as_abstract(),
            ),
        )
        .with_shape(
            interface(BI_CONSUMER, &["T", "U"]).with_method(
                MethodShape::new("accept")
                    .with_parameters(vec![variable("T"), variable("U")])
                    .as_abstract(),
            ),
        )
        .with_shape(
            interface(MAP, &["K", "V"])
                .with_method(
                    MethodShape::new("of")
                        .as_static()
                        .with_type_parameter("K")
                        .with_type_parameter("V")
                        .with_parameters(vec![variable("K"), variable("V")])
                        .returning(generic(MAP, &[variable("K"), variable("V")])),
                )
                .with_method(
                    MethodShape::new("put")
                        .with_parameters(vec![variable("K"), variable("V")])
                        .returning(variable("V")),
                )
                .with_method(
                    MethodShape::new("forEach").with_parameters(vec![generic(
                        BI_CONSUMER,
                        &[variable("K"), variable("V")],
                    )]),
                ),
        )
        .with_shape(
            class("Event")
                .with_method(MethodShape::new("getReceipt").returning(TypeRef::class("Receipt"))),
        )
        .with_shape(class("Receipt").with_method(MethodShape::new("close")))
        .with_shape(interface(COLLECTION, &["E"]))
        .with_shape(
            class(ARRAY_LIST)
                .with_type_parameter("E")
                .extending(generic(COLLECTION, &[variable("E")])),
        )
        .with_shape(
            class(STREAMS).with_method(
                MethodShape::new("from")
                    .as_static()
                    .with_type_parameter("T")
                    .with_parameters(vec![generic(COLLECTION, &[variable("T")])])
                    .returning(generic(STREAM, &[variable("T")])),
            ),
        )
        .with_shape(
            class("Customers").with_method(
                MethodShape::new("of")
                    .as_static()
                    .with_parameters(vec![order()])
                    .returning(TypeRef::class("Customer")),
            ),
        )
        .with_shape(
            class("Registry").with_method(
                MethodShape::new("lookup")
                    .with_parameters(vec![order()])
                    .returning(TypeRef::class("Customer")),
            ),
        )
        .with_shape(
            class("Order")
                .with_method(MethodShape::new("getCustomer").returning(TypeRef::class("Customer"))),
        )
        .with_shape(
            class("Customer")
                .with_method(MethodShape::new("rename"))
                .with_method(MethodShape::new("getWallet").returning(TypeRef::class("Wallet"))),
        )
        .with_shape(class("Wallet").with_method(MethodShape::new("withdraw")))
}

fn customer_of(variable: &str, line: usize) -> Call {
    Call::new(
        Expression::return_of(Call::new(Expression::local(variable), "getCustomer", line)),
        "rename",
        line,
    )
}

#[test]
fn the_type_argument_of_a_generic_method_comes_from_an_array_argument() {
    let notify = Method::new("Shipping", "notify", 1).call(
        Call::new(
            Expression::return_of(
                Call::new(Expression::static_type(TypeRef::class(ARRAYS)), "stream", 2)
                    .with(Expression::parameter("orders", TypeRef::array(order()))),
            ),
            "forEach",
            2,
        )
        .with(Lambda::new(&["order"]).call(customer_of("order", 3))),
    );

    let verdicts = notify.verdicts(&shop());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 2, Reason::static_type(TypeRef::class(ARRAYS))),
            Verdict::allowed(
                "getCustomer",
                3,
                Reason::lambda_parameter("order", Reason::static_result("stream"))
            ),
            Verdict::violation("rename", 3, Reason::return_of("getCustomer")),
            Verdict::allowed("forEach", 2, Reason::static_result("stream")),
        ]
    );
}

#[test]
fn the_type_argument_of_a_generic_method_comes_through_the_supertype_of_the_argument() {
    let notify = Method::new("Shipping", "notify", 1).call(
        Call::new(
            Expression::return_of(
                Call::new(Expression::static_type(TypeRef::class(STREAMS)), "from", 2).with(
                    Expression::parameter("orders", generic(ARRAY_LIST, &[order()])),
                ),
            ),
            "forEach",
            2,
        )
        .with(Lambda::new(&["order"]).call(customer_of("order", 3))),
    );

    let verdicts = notify.verdicts(&shop());

    assert_eq!(
        verdicts[2],
        Verdict::violation("rename", 3, Reason::return_of("getCustomer"))
    );
}

#[test]
fn a_lambda_parameter_takes_its_type_from_the_functional_interface_the_parameter_declares() {
    let handlers = generic(
        MAP,
        &[
            TypeRef::class("java.lang.String"),
            generic(CONSUMER, &[TypeRef::class("Event")]),
        ],
    );
    let register = Method::new("Dispatcher", "register", 1).call(
        Call::new(Expression::field("handlers", handlers), "put", 2)
            .with(Expression::literal())
            .with(Lambda::new(&["event"]).call(Call::new(
                Expression::return_of(Call::new(Expression::local("event"), "getReceipt", 3)),
                "close",
                3,
            ))),
    );

    let verdicts = register.verdicts(&shop());

    assert_eq!(
        verdicts[1],
        Verdict::violation("close", 3, Reason::return_of("getReceipt"))
    );
}

#[test]
fn each_lambda_parameter_takes_the_type_at_its_position_in_the_functional_interface() {
    let ledger = generic(MAP, &[TypeRef::class("Customer"), order()]);
    let audit = Method::new("Auditor", "audit", 1).call(
        Call::new(Expression::field("ledger", ledger), "forEach", 2).with(
            Lambda::new(&["customer", "order"]).call(Call::new(
                Expression::return_of(Call::new(Expression::local("customer"), "getWallet", 3)),
                "withdraw",
                3,
            )),
        ),
    );

    let verdicts = audit.verdicts(&shop());

    assert_eq!(
        verdicts[1],
        Verdict::violation("withdraw", 3, Reason::return_of("getWallet"))
    );
}

fn handlers_type() -> TypeRef {
    generic(
        MAP,
        &[
            TypeRef::class("java.lang.String"),
            generic(CONSUMER, &[TypeRef::class("Event")]),
        ],
    )
}

fn registering_map_of() -> Call {
    Call::new(Expression::static_type(TypeRef::class(MAP)), "of", 2)
        .with(Expression::literal())
        .with(Lambda::new(&["event"]).call(Call::new(
            Expression::return_of(Call::new(Expression::local("event"), "getReceipt", 3)),
            "close",
            3,
        )))
}

fn closing_the_receipt() -> Verdict {
    Verdict::violation("close", 3, Reason::return_of("getReceipt"))
}

#[test]
fn a_call_expecting_a_type_binds_its_type_variables_from_it() {
    let register = Method::new("Dispatcher", "register", 1)
        .call(registering_map_of().expecting(handlers_type()));

    let verdicts = register.verdicts(&shop());

    assert_eq!(verdicts[1], closing_the_receipt());
}

#[test]
fn a_local_declared_with_a_type_gives_its_type_to_the_call_that_fills_it() {
    let register = Method::new("Dispatcher", "register", 1).assign(
        "handlers",
        handlers_type(),
        Expression::return_of(registering_map_of()),
    );

    let verdicts = register.verdicts(&shop());

    assert_eq!(verdicts[1], closing_the_receipt());
}

#[test]
fn a_cast_gives_its_type_to_the_call_it_wraps() {
    let register = Method::new("Dispatcher", "register", 1).evaluate(Expression::cast(
        handlers_type(),
        Expression::return_of(registering_map_of()),
    ));

    let verdicts = register.verdicts(&shop());

    assert_eq!(verdicts[1], closing_the_receipt());
}

#[test]
fn the_expression_a_lambda_yields_is_judged_like_a_statement() {
    let orders = Expression::parameter("orders", generic(LIST, &[order()]));
    let charge = Method::new("Billing", "charge", 1).call(
        Call::new(orders, "stream", 2)
            .then("forEach", 2)
            .with(Lambda::new(&["order"]).yielding(Expression::return_of(customer_of("order", 3)))),
    );

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

fn withdrawing_from_the_wallet_of(variable: &str) -> Lambda {
    Lambda::new(&[variable]).call(Call::new(
        Expression::return_of(Call::new(Expression::local(variable), "getWallet", 3)),
        "withdraw",
        3,
    ))
}

#[test]
fn the_type_argument_of_a_generic_method_comes_from_the_result_of_a_lambda() {
    let orders = Expression::parameter("orders", generic(LIST, &[order()]));
    let charge = Method::new("Billing", "charge", 1).call(
        Call::new(orders, "stream", 2)
            .then("map", 2)
            .with(
                Lambda::new(&["order"]).yielding(Expression::return_of(Call::new(
                    Expression::local("order"),
                    "getCustomer",
                    2,
                ))),
            )
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
            Verdict::allowed(
                "getWallet",
                3,
                Reason::lambda_parameter("customer", Reason::parameter("orders"))
            ),
            Verdict::violation("withdraw", 3, Reason::return_of("getWallet")),
            Verdict::allowed("forEach", 3, Reason::parameter("orders")),
        ]
    );
}

fn mapping_orders_with(reference: MethodReference) -> Method {
    let orders = Expression::parameter("orders", generic(LIST, &[order()]));
    Method::new("Billing", "charge", 1).call(
        Call::new(orders, "stream", 2)
            .then("map", 2)
            .with(reference)
            .then("forEach", 3)
            .with(withdrawing_from_the_wallet_of("customer")),
    )
}

fn charging_the_mapped_customers() -> [Verdict; 5] {
    [
        Verdict::allowed("stream", 2, Reason::parameter("orders")),
        Verdict::allowed("map", 2, Reason::parameter("orders")),
        Verdict::allowed(
            "getWallet",
            3,
            Reason::lambda_parameter("customer", Reason::parameter("orders")),
        ),
        Verdict::violation("withdraw", 3, Reason::return_of("getWallet")),
        Verdict::allowed("forEach", 3, Reason::parameter("orders")),
    ]
}

#[test]
fn a_method_reference_to_an_instance_method_yields_what_the_method_returns() {
    let charge = mapping_orders_with(MethodReference::new(
        Expression::static_type(order()),
        "getCustomer",
    ));

    let verdicts = charge.verdicts(&shop());

    assert_eq!(verdicts, charging_the_mapped_customers());
}

#[test]
fn a_constructor_reference_yields_the_type_it_creates() {
    let charge = mapping_orders_with(MethodReference::new(
        Expression::static_type(TypeRef::class("Customer")),
        "new",
    ));

    let verdicts = charge.verdicts(&shop());

    assert_eq!(verdicts, charging_the_mapped_customers());
}

#[test]
fn a_method_reference_to_a_static_method_yields_what_the_method_returns() {
    let charge = mapping_orders_with(MethodReference::new(
        Expression::static_type(TypeRef::class("Customers")),
        "of",
    ));

    let verdicts = charge.verdicts(&shop());

    assert_eq!(verdicts, charging_the_mapped_customers());
}

#[test]
fn a_method_reference_on_an_expression_yields_what_the_method_of_its_type_returns() {
    let charge = mapping_orders_with(MethodReference::new(
        Expression::field("registry", TypeRef::class("Registry")),
        "lookup",
    ));

    let verdicts = charge.verdicts(&shop());

    assert_eq!(verdicts, charging_the_mapped_customers());
}

fn with_primitive_array_overload(types: Types) -> Types {
    let arrays = class(ARRAYS)
        .with_method(
            MethodShape::new("stream")
                .as_static()
                .with_parameters(vec![TypeRef::array(TypeRef::primitive("int"))])
                .returning(TypeRef::class("java.util.stream.IntStream")),
        )
        .with_method(
            MethodShape::new("stream")
                .as_static()
                .with_type_parameter("T")
                .with_parameters(vec![TypeRef::array(variable("T"))])
                .returning(generic(STREAM, &[variable("T")])),
        );
    types.with_shape(arrays)
}

#[test]
fn the_overload_whose_parameters_fit_the_arguments_decides_the_type_variables() {
    let notify = Method::new("Shipping", "notify", 1).call(
        Call::new(
            Expression::return_of(
                Call::new(Expression::static_type(TypeRef::class(ARRAYS)), "stream", 2)
                    .with(Expression::parameter("orders", TypeRef::array(order()))),
            ),
            "forEach",
            2,
        )
        .with(Lambda::new(&["order"]).call(customer_of("order", 3))),
    );

    let verdicts = notify.verdicts(&with_primitive_array_overload(shop()));

    assert_eq!(
        verdicts[2],
        Verdict::violation("rename", 3, Reason::return_of("getCustomer"))
    );
}

fn with_quotes(overloads: &[(TypeRef, &str)]) -> Types {
    let pricing = overloads
        .iter()
        .fold(class("Pricing"), |shape, (parameter, returns)| {
            shape.with_method(
                MethodShape::new("quote")
                    .with_parameters(vec![parameter.clone()])
                    .returning(TypeRef::class(returns)),
            )
        });
    shop().with_shape(pricing)
}

fn quoting_with(argument: TypeRef, then: &str) -> Method {
    Method::new("Billing", "charge", 1).call(Call::new(
        Expression::return_of(
            Call::new(
                Expression::parameter("pricing", TypeRef::class("Pricing")),
                "quote",
                2,
            )
            .with(Expression::parameter("argument", argument)),
        ),
        then,
        2,
    ))
}

fn int() -> TypeRef {
    TypeRef::primitive("int")
}

fn wallet() -> TypeRef {
    TypeRef::class("Wallet")
}

#[test]
fn the_overload_whose_primitive_parameter_matches_the_argument_decides_the_quote() {
    let types = with_quotes(&[(int(), "Wallet"), (TypeRef::primitive("long"), "Customer")]);
    let charge = quoting_with(int(), "withdraw");

    let verdicts = charge.verdicts(&types);

    assert_eq!(
        verdicts[1],
        Verdict::violation("withdraw", 2, Reason::return_of("quote"))
    );
}

#[test]
fn the_overload_whose_class_parameter_is_a_supertype_of_the_argument_decides_the_quote() {
    let types = with_quotes(&[(order(), "Customer"), (wallet(), "Wallet")]);
    let charge = quoting_with(wallet(), "withdraw");

    let verdicts = charge.verdicts(&types);

    assert_eq!(
        verdicts[1],
        Verdict::violation("withdraw", 2, Reason::return_of("quote"))
    );
}

#[test]
fn an_array_argument_fits_only_object_and_array_parameters() {
    let object = TypeRef::class("java.lang.Object");
    let types = with_quotes(&[(object, "Wallet"), (wallet(), "Customer")]);
    let charge = quoting_with(TypeRef::array(order()), "withdraw");

    let verdicts = charge.verdicts(&types);

    assert_eq!(
        verdicts[1],
        Verdict::violation("withdraw", 2, Reason::return_of("quote"))
    );
}

#[test]
fn an_argument_of_a_type_the_catalog_lacks_fits_any_class_parameter() {
    let types = with_quotes(&[(order(), "Customer"), (int(), "Wallet")]);
    let charge = quoting_with(TypeRef::class("Unmapped"), "rename");

    let verdicts = charge.verdicts(&types);

    assert_eq!(
        verdicts[1],
        Verdict::violation("rename", 2, Reason::return_of("quote"))
    );
}

#[test]
fn overloads_that_all_miss_the_argument_still_agree_on_a_common_return() {
    let types = with_quotes(&[(order(), "Customer"), (wallet(), "Customer")]);
    let charge = quoting_with(TypeRef::class("Customer"), "rename");

    let verdicts = charge.verdicts(&types);

    assert_eq!(
        verdicts[1],
        Verdict::violation("rename", 2, Reason::return_of("quote"))
    );
}

#[test]
fn a_class_argument_fits_an_object_parameter() {
    let object = TypeRef::class("java.lang.Object");
    let types = with_quotes(&[(object, "Customer"), (int(), "Wallet")]);
    let charge = quoting_with(wallet(), "rename");

    let verdicts = charge.verdicts(&types);

    assert_eq!(
        verdicts[1],
        Verdict::violation("rename", 2, Reason::return_of("quote"))
    );
}

#[test]
fn a_call_returning_a_subtype_of_the_expected_type_binds_through_the_supertype() {
    let lists = class("Lists").with_method(
        MethodShape::new("listOf")
            .as_static()
            .with_type_parameter("T")
            .with_parameters(vec![variable("T")])
            .returning(generic(ARRAY_LIST, &[variable("T")])),
    );
    let handlers = generic(COLLECTION, &[generic(CONSUMER, &[TypeRef::class("Event")])]);
    let register = Method::new("Dispatcher", "register", 1).assign(
        "handlers",
        handlers,
        Expression::return_of(
            Call::new(
                Expression::static_type(TypeRef::class("Lists")),
                "listOf",
                2,
            )
            .with(Lambda::new(&["event"]).call(Call::new(
                Expression::return_of(Call::new(Expression::local("event"), "getReceipt", 3)),
                "close",
                3,
            ))),
        ),
    );

    let verdicts = register.verdicts(&shop().with_shape(lists));

    assert_eq!(verdicts[1], closing_the_receipt());
}
