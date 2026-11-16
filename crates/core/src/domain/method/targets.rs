use super::{Call, Expression, Lambda, Method, MethodReference, Reason, Verdict};
use crate::domain::classification::{Types, sample_types};
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
    sample_types()
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

fn typed_handler() -> Lambda {
    let entry = Expression::return_of(
        Call::new(Expression::local("e"), "getEntry", 2).returning(TypeRef::class("Entry")),
    );
    let lock =
        Expression::return_of(Call::new(entry, "getLock", 2).returning(TypeRef::class("Lock")));
    Lambda::new(&["e"])
        .declaring("e", event())
        .yielding(Expression::return_of(Call::new(lock, "release", 2)))
}

#[test]
fn an_explicitly_typed_parameter_of_a_lambda_in_value_position_keeps_its_type() {
    let register = Method::new("Dispatcher", "register", 1)
        .assign_var("handler", Expression::lambda(typed_handler()));

    let verdicts = register.verdicts(&events());

    assert_eq!(verdicts, releasing_through_a_typed_parameter());
}

#[test]
fn an_explicitly_typed_parameter_of_a_lambda_argument_keeps_its_type() {
    let registry = Expression::parameter("registry", TypeRef::class("Registry"));
    let register = Method::new("Dispatcher", "register", 1)
        .call(Call::new(registry, "register", 2).with(typed_handler()));

    let verdicts = register.verdicts(&events());

    let fed = |name: &str| Reason::lambda_parameter(name, Reason::parameter("registry"));
    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getEntry", 2, fed("e")),
            Verdict::violation("getLock", 2, Reason::return_of("getEntry")),
            Verdict::violation("release", 2, Reason::return_of("getLock")),
            Verdict::allowed("register", 2, Reason::parameter("registry")),
        ]
    );
}

#[test]
fn a_lambda_parameter_without_a_declared_type_stays_untyped_in_an_argument() {
    let registry = Expression::parameter("registry", TypeRef::class("Registry"));
    let register = Method::new("Dispatcher", "register", 1).call(
        Call::new(registry, "register", 2)
            .with(Lambda::new(&["e"]).yielding(releasing_the_lock_of_the_entry_of("e"))),
    );

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

fn handler_types() -> Types {
    events()
        .with_shape(
            class("Entries").with_method(
                MethodShape::new("of")
                    .as_static()
                    .with_parameters(vec![event()])
                    .returning(TypeRef::class("Entry")),
            ),
        )
        .with_shape(
            class("Naming").with_method(
                MethodShape::new("withName")
                    .with_parameters(vec![event()])
                    .returning(TypeRef::class("Naming")),
            ),
        )
        .with_shape(
            class("Registry")
                .with_method(MethodShape::new("naming").returning(TypeRef::class("Naming"))),
        )
}

fn registering(reference: MethodReference) -> Method {
    Method::new("Dispatcher", "register", 1).assign(
        "handler",
        consumer_of(event()),
        Expression::method_reference(reference),
    )
}

#[test]
fn an_unbound_reference_value_is_allowed_like_the_call_on_the_parameter_of_a_lambda() {
    let register = registering(MethodReference::new(
        Expression::static_type(event()),
        "getEntry",
        2,
    ));

    let verdicts = register.verdicts(&handler_types());

    assert_eq!(
        verdicts,
        [Verdict::allowed(
            "getEntry",
            2,
            Reason::method_reference(event(), "getEntry", Reason::FunctionArgument)
        )]
    );
}

#[test]
fn a_static_reference_value_is_allowed_as_the_static_call_it_stands_for() {
    let register = registering(MethodReference::new(
        Expression::static_type(TypeRef::class("Entries")),
        "of",
        2,
    ));

    let verdicts = register.verdicts(&handler_types());

    assert_eq!(
        verdicts,
        [Verdict::allowed(
            "of",
            2,
            Reason::static_type(TypeRef::class("Entries"))
        )]
    );
}

#[test]
fn a_reference_value_on_what_a_stranger_returned_is_a_violation_like_the_lambda_call() {
    let stranger = Expression::return_of(
        Call::new(Expression::parameter("event", event()), "getEntry", 2)
            .returning(TypeRef::class("Entry")),
    );
    let register = registering(MethodReference::new(stranger, "getLock", 2));

    let verdicts = register.verdicts(&handler_types());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getEntry", 2, Reason::parameter("event")),
            Verdict::violation("getLock", 2, Reason::return_of("getEntry")),
        ]
    );
}

#[test]
fn a_reference_value_on_a_field_is_allowed() {
    let register = registering(MethodReference::new(
        Expression::field("entry", TypeRef::class("Entry")),
        "getLock",
        2,
    ));

    let verdicts = register.verdicts(&handler_types());

    assert_eq!(
        verdicts,
        [Verdict::allowed("getLock", 2, Reason::field("entry"))]
    );
}

#[test]
fn a_constructor_reference_value_judges_nothing() {
    let register = registering(MethodReference::new(
        Expression::static_type(TypeRef::class("Entry")),
        "new",
        2,
    ));

    let verdicts = register.verdicts(&handler_types());

    assert_eq!(verdicts, []);
}

fn naming_reference() -> MethodReference {
    let naming = Expression::return_of(
        Call::new(
            Expression::field("registry", TypeRef::class("Registry")),
            "naming",
            2,
        )
        .returning(TypeRef::class("Naming")),
    );
    MethodReference::new(naming, "withName", 2)
}

fn the_naming_of_the_registry() -> Verdict {
    Verdict::allowed("naming", 2, Reason::field("registry"))
}

#[test]
fn a_reference_value_is_fluent_when_the_arity_it_takes_matches_the_method() {
    let register = registering(naming_reference().taking(1));

    let verdicts = register.verdicts(&handler_types());

    assert_eq!(
        verdicts,
        [
            the_naming_of_the_registry(),
            Verdict::allowed("withName", 2, Reason::fluent(TypeRef::class("Naming"))),
        ]
    );
}

#[test]
fn a_reference_value_without_a_known_arity_is_judged_with_no_arguments() {
    let register = Method::new("Dispatcher", "register", 1)
        .assign_var("handler", Expression::method_reference(naming_reference()));

    let verdicts = register.verdicts(&handler_types());

    assert_eq!(
        verdicts[1],
        Verdict::violation("withName", 2, Reason::return_of("naming"))
    );
}

#[test]
fn a_local_holding_a_reference_is_an_object_created_in_the_method_with_the_declared_type() {
    let register = registering(MethodReference::new(
        Expression::static_type(TypeRef::class("Entry")),
        "new",
        2,
    ))
    .call(Call::new(Expression::local("handler"), "accept", 3));

    let verdicts = register.verdicts(&handler_types());

    assert_eq!(
        verdicts,
        [Verdict::allowed(
            "accept",
            3,
            Reason::local("handler", Reason::created(consumer_of(event())))
        )]
    );
}
