use super::{Call, Expression, Lambda, Method, Reason, Verdict};
use crate::domain::classification::Types;
use crate::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

const CONSUMER: &str = "java.util.function.Consumer";

fn class(name: &str) -> TypeShape {
    TypeShape::new(name, TypeKind::Class)
}

fn consumer_of(argument: TypeRef) -> TypeRef {
    TypeRef::generic(CONSUMER, vec![argument])
}

fn event() -> TypeRef {
    TypeRef::class("Event")
}

fn events() -> Types {
    Types::new()
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
            class("Event")
                .with_method(MethodShape::new("getEntry").returning(TypeRef::class("Entry"))),
        )
        .with_shape(
            class("Entry")
                .with_method(MethodShape::new("getLock").returning(TypeRef::class("Lock"))),
        )
        .with_shape(class("Lock").with_method(MethodShape::new("release")))
}

fn releasing_the_lock_of_the_entry_of(parameter: &str) -> Expression {
    let entry = Expression::return_of(Call::new(Expression::local(parameter), "getEntry", 2));
    let lock = Expression::return_of(Call::new(entry, "getLock", 2));
    Expression::return_of(Call::new(lock, "release", 2))
}

fn handler() -> Expression {
    Expression::lambda(Lambda::new(&["e"]).yielding(releasing_the_lock_of_the_entry_of("e")))
}

#[test]
fn a_lambda_without_a_target_judges_its_body_with_untyped_friendly_parameters() {
    let register = Method::new("Dispatcher", "register", 1).assign_var("handler", handler());

    let verdicts = register.verdicts(&events());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getEntry", 2, Reason::parameter("e")),
            Verdict::violation(
                "getLock",
                2,
                Reason::unresolved_type(None, Reason::return_of("getEntry"))
            ),
            Verdict::violation(
                "release",
                2,
                Reason::unresolved_type(None, Reason::return_of("getLock"))
            ),
        ]
    );
}

fn releasing_through_a_typed_parameter() -> [Verdict; 3] {
    [
        Verdict::allowed("getEntry", 2, Reason::parameter("e")),
        Verdict::violation("getLock", 2, Reason::return_of("getEntry")),
        Verdict::violation("release", 2, Reason::return_of("getLock")),
    ]
}

#[test]
fn a_local_declared_with_a_functional_type_types_the_parameters_of_its_lambda() {
    let register =
        Method::new("Dispatcher", "register", 1).assign("handler", consumer_of(event()), handler());

    let verdicts = register.verdicts(&events());

    assert_eq!(verdicts, releasing_through_a_typed_parameter());
}

#[test]
fn a_cast_gives_its_type_to_the_lambda_it_wraps() {
    let register = Method::new("Dispatcher", "register", 1)
        .evaluate(Expression::cast(consumer_of(event()), handler()));

    let verdicts = register.verdicts(&events());

    assert_eq!(verdicts, releasing_through_a_typed_parameter());
}

#[test]
fn a_lambda_whose_target_is_no_functional_interface_keeps_untyped_parameters() {
    let register = Method::new("Dispatcher", "register", 1).assign("handler", event(), handler());

    let verdicts = register.verdicts(&events());

    assert_eq!(
        verdicts[1],
        Verdict::violation(
            "getLock",
            2,
            Reason::unresolved_type(None, Reason::return_of("getEntry"))
        )
    );
}

#[test]
fn a_local_holding_a_lambda_is_an_object_created_in_the_method_with_the_declared_type() {
    let register = Method::new("Dispatcher", "register", 1)
        .assign("handler", consumer_of(event()), handler())
        .call(Call::new(Expression::local("handler"), "accept", 3).with(Expression::literal()));

    let verdicts = register.verdicts(&events());

    assert_eq!(
        verdicts[3],
        Verdict::allowed(
            "accept",
            3,
            Reason::local("handler", Reason::created(consumer_of(event())))
        )
    );
}
