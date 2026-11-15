use super::{Call, Expression, Lambda, Method, Reason, Verdict};
use crate::domain::classification::Types;
use crate::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

const ARRAYS: &str = "java.util.Arrays";
const STREAM: &str = "java.util.stream.Stream";
const CONSUMER: &str = "java.util.function.Consumer";
const COLLECTION: &str = "java.util.Collection";
const ARRAY_LIST: &str = "java.util.ArrayList";
const STREAMS: &str = "shop.Streams";
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
        .with_shape(interface(STREAM, &["T"]).with_method(
            MethodShape::new("forEach").with_parameters(vec![generic(CONSUMER, &[variable("T")])]),
        ))
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
