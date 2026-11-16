use super::{Call, Expression, Lambda, Method, MethodReference, Reason, Verdict};
use crate::domain::classification::{Classification, Types, sample_types};
use crate::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

const CONSUMER: &str = "java.util.function.Consumer";
const FUNCTION: &str = "java.util.function.Function";
const STREAM: &str = "java.util.stream.Stream";
const LIST: &str = "java.util.List";
const OBJECT: &str = "java.lang.Object";
const BIG_DECIMAL: &str = "java.math.BigDecimal";

fn class(name: &str) -> TypeShape {
    TypeShape::new(name, TypeKind::Class)
}

fn order() -> TypeRef {
    TypeRef::class("Order")
}

fn shop() -> Types {
    sample_types()
        .with_shape(
            class("Order")
                .with_method(MethodShape::new("getTotal").returning(TypeRef::class(OBJECT)))
                .with_method(MethodShape::new("getCustomer").returning(TypeRef::class("Customer"))),
        )
        .with_shape(class("Customer").with_method(MethodShape::new("rename")))
}

#[test]
fn a_call_takes_its_resolved_return_type_over_the_one_its_shape_declares() {
    let charge = Method::new("Checkout", "charge", 1).call(Call::new(
        Expression::return_of(
            Call::new(Expression::parameter("order", order()), "getTotal", 2)
                .returning(TypeRef::class(BIG_DECIMAL)),
        ),
        "signum",
        2,
    ));

    let verdicts = charge.verdicts(&shop());

    assert_eq!(
        verdicts[1],
        Verdict::allowed(
            "signum",
            2,
            Reason::classified(Classification::Value, TypeRef::class(BIG_DECIMAL))
        )
    );
}

#[test]
fn a_call_takes_its_resolved_return_type_when_its_shape_declares_none() {
    let charge = Method::new("Checkout", "charge", 1).call(Call::new(
        Expression::return_of(
            Call::new(
                Expression::parameter("gateway", TypeRef::class("Gateway")),
                "quote",
                2,
            )
            .returning(TypeRef::class(BIG_DECIMAL)),
        ),
        "signum",
        2,
    ));

    let verdicts = charge.verdicts(&shop());

    assert_eq!(
        verdicts[1],
        Verdict::allowed(
            "signum",
            2,
            Reason::classified(Classification::Value, TypeRef::class(BIG_DECIMAL))
        )
    );
}

fn renaming_the_customer_of(order: &str) -> Call {
    Call::new(
        Expression::return_of(Call::new(Expression::local(order), "getCustomer", 3)),
        "rename",
        3,
    )
}

#[test]
fn a_lambda_argument_takes_its_resolved_parameter_type() {
    let register = Method::new("Dispatcher", "register", 1).call(
        Call::new(
            Expression::field("dispatcher", TypeRef::class("Dispatcher")),
            "onEach",
            2,
        )
        .with(
            Lambda::new(&["order"])
                .resolving("order", order())
                .call(renaming_the_customer_of("order")),
        ),
    );

    let verdicts = register.verdicts(&shop());

    assert_eq!(
        verdicts[1],
        Verdict::violation("rename", 3, Reason::return_of("getCustomer"))
    );
}

#[test]
fn a_lambda_value_takes_its_resolved_parameter_type() {
    let register = Method::new("Dispatcher", "register", 1).evaluate(Expression::lambda(
        Lambda::new(&["order"])
            .resolving("order", order())
            .call(renaming_the_customer_of("order")),
    ));

    let verdicts = register.verdicts(&shop());

    assert_eq!(
        verdicts[1],
        Verdict::violation("rename", 3, Reason::return_of("getCustomer"))
    );
}

#[test]
fn a_lambda_parameter_keeps_its_source_declared_type_over_the_resolved_one() {
    let register = Method::new("Dispatcher", "register", 1).evaluate(Expression::lambda(
        Lambda::new(&["order"])
            .declaring("order", order())
            .resolving("order", TypeRef::class("Customer"))
            .call(renaming_the_customer_of("order")),
    ));

    let verdicts = register.verdicts(&shop());

    assert_eq!(
        verdicts[1],
        Verdict::violation("rename", 3, Reason::return_of("getCustomer"))
    );
}

#[test]
fn a_lambda_argument_takes_its_resolved_parameter_type_over_the_functional_interface() {
    let consumer = TypeRef::generic(CONSUMER, vec![TypeRef::class("Customer")]);
    let types = shop()
        .with_shape(
            TypeShape::new(CONSUMER, TypeKind::Interface)
                .with_type_parameter("T")
                .with_method(
                    MethodShape::new("accept")
                        .with_parameters(vec![TypeRef::variable("T")])
                        .as_abstract(),
                ),
        )
        .with_shape(
            class("Notifier")
                .with_method(MethodShape::new("onEach").with_parameters(vec![consumer])),
        );
    let register = Method::new("Dispatcher", "register", 1).call(
        Call::new(
            Expression::field("notifier", TypeRef::class("Notifier")),
            "onEach",
            2,
        )
        .with(
            Lambda::new(&["order"])
                .resolving("order", order())
                .call(renaming_the_customer_of("order")),
        ),
    );

    let verdicts = register.verdicts(&types);

    assert_eq!(
        verdicts[1],
        Verdict::violation("rename", 3, Reason::return_of("getCustomer"))
    );
}

fn generic(name: &str, arguments: &[TypeRef]) -> TypeRef {
    TypeRef::generic(name, arguments.to_vec())
}

fn streaming() -> Types {
    let element = TypeRef::variable("T");
    let result = TypeRef::variable("R");
    shop()
        .with_shape(
            TypeShape::new(STREAM, TypeKind::Interface)
                .with_type_parameter("T")
                .with_method(
                    MethodShape::new("map")
                        .with_type_parameter("R")
                        .with_parameters(vec![generic(
                            FUNCTION,
                            &[element.clone(), result.clone()],
                        )])
                        .returning(generic(STREAM, std::slice::from_ref(&result))),
                )
                .with_method(
                    MethodShape::new("forEach")
                        .with_parameters(vec![generic(CONSUMER, std::slice::from_ref(&element))]),
                ),
        )
        .with_shape(
            TypeShape::new(FUNCTION, TypeKind::Interface)
                .with_type_parameter("T")
                .with_type_parameter("R")
                .with_method(
                    MethodShape::new("apply")
                        .with_parameters(vec![element.clone()])
                        .returning(result)
                        .as_abstract(),
                ),
        )
        .with_shape(
            TypeShape::new(CONSUMER, TypeKind::Interface)
                .with_type_parameter("T")
                .with_method(
                    MethodShape::new("accept")
                        .with_parameters(vec![element])
                        .as_abstract(),
                ),
        )
        .with_shape(
            TypeShape::new(LIST, TypeKind::Interface)
                .with_type_parameter("E")
                .with_method(
                    MethodShape::new("stream")
                        .returning(generic(STREAM, &[TypeRef::variable("E")])),
                ),
        )
}

#[test]
fn a_method_reference_takes_its_resolved_result_type_over_the_one_its_shape_declares() {
    let total = MethodReference::new(Expression::static_type(order()), "getTotal", 2)
        .returning(TypeRef::class(BIG_DECIMAL));
    let orders = Expression::parameter("orders", generic(LIST, &[order()]));
    let sign = Lambda::new(&["total"]).call(Call::new(Expression::local("total"), "signum", 3));
    let charge = Method::new("Billing", "charge", 1).call(
        Call::new(orders, "stream", 2)
            .then("map", 2)
            .with(total)
            .then("forEach", 3)
            .with(sign),
    );

    let verdicts = charge.verdicts(&streaming());

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
fn a_method_reference_with_a_resolved_arity_yields_its_result_without_the_functional_interface() {
    let customers = MethodReference::new(Expression::static_type(order()), "getCustomer", 2)
        .taking(1)
        .returning(TypeRef::class("Customer"))
        .fed_by_elements();
    let rename = Lambda::new(&["customer"])
        .resolving("customer", TypeRef::class("Customer"))
        .fed_by_receiver("customer")
        .call(Call::new(Expression::local("customer"), "rename", 3));
    let orders = Expression::parameter("orders", generic(LIST, &[order()]));
    let charge = Method::new("Billing", "charge", 1).call(
        Call::new(orders, "stream", 2)
            .returning(generic(STREAM, &[order()]))
            .then("map", 2)
            .returning(generic(STREAM, &[TypeRef::class("Customer")]))
            .returning_from(&[0])
            .with(customers)
            .then("forEach", 3)
            .with(rename),
    );

    let verdicts = charge.verdicts(&shop());

    assert_eq!(
        verdicts[3],
        Verdict::violation(
            "rename",
            3,
            Reason::lambda_parameter("customer", Reason::return_of("getCustomer"))
        )
    );
}
