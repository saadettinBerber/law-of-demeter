use super::{Call, Creation, Expression, Lambda, Method, MethodReference, Reason, Verdict};
use crate::domain::classification::{Platform, Types, sample_types};
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

#[test]
fn a_lambda_expecting_a_type_types_its_parameters_without_a_declaration() {
    let lambda = Lambda::new(&["e"])
        .yielding(releasing_the_lock_of_the_entry_of("e"))
        .expecting(consumer_of(event()));
    let register = Method::new("Dispatcher", "register", 1).evaluate(Expression::lambda(lambda));

    let verdicts = register.verdicts(&events());

    assert_eq!(verdicts, releasing_through_a_typed_parameter());
}

fn typed_handler() -> Lambda {
    Lambda::new(&["e"])
        .declaring("e", event())
        .yielding(releasing_the_lock_of_the_entry_of("e"))
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

fn constructor(parameters: Vec<TypeRef>) -> MethodShape {
    MethodShape::new("<init>").with_parameters(parameters)
}

fn worker_taking(constructors: Vec<MethodShape>) -> Types {
    let worker = constructors
        .into_iter()
        .fold(class("Worker"), |worker, constructor| {
            worker.with_method(constructor)
        });
    events().with_shape(worker)
}

fn creating(type_ref: TypeRef, lambdas: Vec<Lambda>) -> Expression {
    let creation = lambdas
        .into_iter()
        .fold(Creation::new(type_ref), Creation::with);
    Expression::created(creation)
}

fn release_in(created: TypeRef) -> [Verdict; 3] {
    let fed = Reason::lambda_parameter("e", Reason::created(created));
    [
        Verdict::allowed("getEntry", 2, fed),
        Verdict::violation("getLock", 2, Reason::return_of("getEntry")),
        Verdict::violation("release", 2, Reason::return_of("getLock")),
    ]
}

fn untyped_getlock() -> Verdict {
    Verdict::violation(
        "getLock",
        2,
        Reason::unresolved_type(None, Reason::return_of("getEntry")),
    )
}

fn plain_handler() -> Lambda {
    Lambda::new(&["e"]).yielding(releasing_the_lock_of_the_entry_of("e"))
}

fn worker() -> TypeRef {
    TypeRef::class("Worker")
}

#[test]
fn a_type_is_constructed_through_the_method_the_platform_names_as_its_constructor() {
    let make = MethodShape::new("make").with_parameters(vec![consumer_of(event())]);
    let types =
        worker_taking(vec![make]).with_platform(Platform::new().with_constructor_name("make"));
    let run =
        Method::new("Dispatcher", "start", 1).evaluate(creating(worker(), vec![plain_handler()]));

    let verdicts = run.verdicts(&types);

    assert_eq!(verdicts, release_in(worker()));
}

#[test]
fn a_constructor_parameter_of_a_functional_type_types_the_lambda_argument() {
    let types = worker_taking(vec![constructor(vec![consumer_of(event())])]);
    let run =
        Method::new("Dispatcher", "start", 1).evaluate(creating(worker(), vec![plain_handler()]));

    let verdicts = run.verdicts(&types);

    assert_eq!(verdicts, release_in(worker()));
}

#[test]
fn the_constructor_whose_parameter_count_fits_the_arguments_types_the_lambda() {
    let types = worker_taking(vec![
        constructor(vec![consumer_of(TypeRef::class("Entry"))]),
        constructor(vec![
            TypeRef::class("java.lang.String"),
            consumer_of(event()),
        ]),
    ]);
    let creation = Creation::new(worker())
        .with(Expression::literal())
        .with(plain_handler());
    let run = Method::new("Dispatcher", "start", 1).evaluate(Expression::created(creation));

    let verdicts = run.verdicts(&types);

    assert_eq!(verdicts, release_in(worker()));
}

#[test]
fn a_variable_arity_constructor_types_every_lambda_it_takes() {
    let tasks = TypeRef::array(consumer_of(event()));
    let types = worker_taking(vec![constructor(vec![tasks]).as_varargs()]);
    let run = Method::new("Dispatcher", "start", 1)
        .evaluate(creating(worker(), vec![plain_handler(), plain_handler()]));

    let verdicts = run.verdicts(&types);

    let both: Vec<_> = release_in(worker())
        .into_iter()
        .chain(release_in(worker()))
        .collect();
    assert_eq!(verdicts, both);
}

#[test]
fn the_constructors_of_a_supertype_do_not_type_the_lambda_of_a_subtype() {
    let types = worker_taking(vec![constructor(vec![consumer_of(event())])])
        .with_shape(class("Helper").extending(worker()));
    let run = Method::new("Dispatcher", "start", 1)
        .evaluate(creating(TypeRef::class("Helper"), vec![plain_handler()]));

    let verdicts = run.verdicts(&types);

    assert_eq!(verdicts[1], untyped_getlock());
}

#[test]
fn constructors_of_the_same_arity_leave_the_lambda_untyped() {
    let types = worker_taking(vec![
        constructor(vec![consumer_of(event())]),
        constructor(vec![consumer_of(TypeRef::class("Entry"))]),
    ]);
    let run =
        Method::new("Dispatcher", "start", 1).evaluate(creating(worker(), vec![plain_handler()]));

    let verdicts = run.verdicts(&types);

    assert_eq!(verdicts[1], untyped_getlock());
}

fn handlers_of_any_event() -> Types {
    let handler = class("Handler")
        .with_type_parameter("T")
        .with_method(constructor(vec![consumer_of(TypeRef::variable("T"))]));
    events().with_shape(handler)
}

fn handler_of(argument: TypeRef) -> TypeRef {
    TypeRef::generic("Handler", vec![argument])
}

#[test]
fn the_type_arguments_written_after_the_class_type_the_lambda_argument() {
    let run = Method::new("Dispatcher", "start", 1)
        .evaluate(creating(handler_of(event()), vec![plain_handler()]));

    let verdicts = run.verdicts(&handlers_of_any_event());

    assert_eq!(verdicts, release_in(handler_of(event())));
}

#[test]
fn a_diamond_takes_its_type_arguments_from_the_declared_type_of_the_local() {
    let run = Method::new("Dispatcher", "start", 1).assign(
        "handler",
        handler_of(event()),
        creating(TypeRef::class("Handler"), vec![plain_handler()]),
    );

    let verdicts = run.verdicts(&handlers_of_any_event());

    assert_eq!(verdicts, release_in(TypeRef::class("Handler")));
}

#[test]
fn a_diamond_without_a_target_leaves_the_lambda_untyped() {
    let run = Method::new("Dispatcher", "start", 1)
        .evaluate(creating(TypeRef::class("Handler"), vec![plain_handler()]));

    let verdicts = run.verdicts(&handlers_of_any_event());

    assert_eq!(verdicts[1], untyped_getlock());
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
    let stranger = Expression::return_of(Call::new(
        Expression::parameter("event", event()),
        "getEntry",
        2,
    ));
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
    let naming = Expression::return_of(Call::new(
        Expression::field("registry", TypeRef::class("Registry")),
        "naming",
        2,
    ));
    MethodReference::new(naming, "withName", 2)
}

fn the_naming_of_the_registry() -> Verdict {
    Verdict::allowed("naming", 2, Reason::field("registry"))
}

#[test]
fn the_arity_of_the_declared_target_decides_whether_a_reference_value_is_fluent() {
    let register = registering(naming_reference());

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
fn a_reference_value_without_a_target_is_judged_with_no_arguments() {
    let register = Method::new("Dispatcher", "register", 1)
        .assign_var("handler", Expression::method_reference(naming_reference()));

    let verdicts = register.verdicts(&handler_types());

    assert_eq!(
        verdicts[1],
        Verdict::violation("withName", 2, Reason::return_of("naming"))
    );
}

#[test]
fn a_reference_value_takes_the_expected_type_it_was_given() {
    let register = Method::new("Dispatcher", "register", 1).evaluate(Expression::method_reference(
        naming_reference().expecting(consumer_of(event())),
    ));

    let verdicts = register.verdicts(&handler_types());

    assert_eq!(
        verdicts[1],
        Verdict::allowed("withName", 2, Reason::fluent(TypeRef::class("Naming")))
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
