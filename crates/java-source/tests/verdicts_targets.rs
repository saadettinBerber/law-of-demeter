mod judging;

use judging::verdicts_in_c;
use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::{Reason, Verdict};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

const CONSUMER: &str = "java.util.function.Consumer";

const CONSUMER_IMPORT: &str = "import java.util.function.Consumer;";

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

fn releasing_through_a_typed_parameter() -> [Verdict; 3] {
    [
        Verdict::allowed("getEntry", 3, Reason::parameter("e")),
        Verdict::violation("getLock", 3, Reason::return_of("getEntry")),
        Verdict::violation("release", 3, Reason::return_of("getLock")),
    ]
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

fn release_in(created: TypeRef) -> [Verdict; 3] {
    let fed = Reason::lambda_parameter("e", Reason::created(created));
    [
        Verdict::allowed("getEntry", 3, fed),
        Verdict::violation("getLock", 3, Reason::return_of("getEntry")),
        Verdict::violation("release", 3, Reason::return_of("getLock")),
    ]
}

fn untyped_getlock() -> Verdict {
    Verdict::violation(
        "getLock",
        3,
        Reason::unresolved_type(None, Reason::return_of("getEntry")),
    )
}

fn worker() -> TypeRef {
    TypeRef::class("Worker")
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
fn a_lambda_whose_target_is_no_functional_interface_keeps_untyped_parameters() {
    let snippet =
        "void register() {\n    Event handler = e -> e.getEntry().getLock().release();\n}";

    let verdicts = verdicts_in_c("", "", snippet, "register", &events());

    assert_eq!(verdicts[1], untyped_getlock());
}

#[test]
fn an_explicitly_typed_parameter_of_a_lambda_in_value_position_keeps_its_type() {
    let snippet =
        "void register() {\n    var handler = (Event e) -> e.getEntry().getLock().release();\n}";

    let verdicts = verdicts_in_c("", "", snippet, "register", &events());

    assert_eq!(verdicts, releasing_through_a_typed_parameter());
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
    let snippet =
        "void start() {\n    new Worker(\"label\", e -> e.getEntry().getLock().release());\n}";

    let verdicts = verdicts_in_c(CONSUMER_IMPORT, "", snippet, "start", &types);

    assert_eq!(verdicts, release_in(worker()));
}

#[test]
fn a_variable_arity_constructor_types_every_lambda_it_takes() {
    let tasks = TypeRef::array(consumer_of(event()));
    let types = worker_taking(vec![constructor(vec![tasks]).as_varargs()]);
    let snippet = "void start() {\n    new Worker(e -> e.getEntry().getLock().release(), e -> e.getEntry().getLock().release());\n}";

    let verdicts = verdicts_in_c(CONSUMER_IMPORT, "", snippet, "start", &types);

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
    let snippet = "void start() {\n    new Helper(e -> e.getEntry().getLock().release());\n}";

    let verdicts = verdicts_in_c(CONSUMER_IMPORT, "", snippet, "start", &types);

    assert_eq!(verdicts[1], untyped_getlock());
}

#[test]
fn constructors_of_the_same_arity_leave_the_lambda_untyped() {
    let types = worker_taking(vec![
        constructor(vec![consumer_of(event())]),
        constructor(vec![consumer_of(TypeRef::class("Entry"))]),
    ]);
    let snippet = "void start() {\n    new Worker(e -> e.getEntry().getLock().release());\n}";

    let verdicts = verdicts_in_c(CONSUMER_IMPORT, "", snippet, "start", &types);

    assert_eq!(verdicts[1], untyped_getlock());
}

#[test]
fn the_type_arguments_written_after_the_class_type_the_lambda_argument() {
    let snippet =
        "void start() {\n    new Handler<Event>(e -> e.getEntry().getLock().release());\n}";

    let verdicts = verdicts_in_c(
        CONSUMER_IMPORT,
        "",
        snippet,
        "start",
        &handlers_of_any_event(),
    );

    assert_eq!(verdicts, release_in(handler_of(event())));
}

#[test]
fn a_diamond_takes_its_type_arguments_from_the_declared_type_of_the_local() {
    let snippet = "void start() {\n    Handler<Event> handler = new Handler<>(e -> e.getEntry().getLock().release());\n}";

    let verdicts = verdicts_in_c(
        CONSUMER_IMPORT,
        "",
        snippet,
        "start",
        &handlers_of_any_event(),
    );

    assert_eq!(verdicts, release_in(TypeRef::class("Handler")));
}

#[test]
fn a_diamond_without_a_target_leaves_the_lambda_untyped() {
    let snippet = "void start() {\n    new Handler<>(e -> e.getEntry().getLock().release());\n}";

    let verdicts = verdicts_in_c(
        CONSUMER_IMPORT,
        "",
        snippet,
        "start",
        &handlers_of_any_event(),
    );

    assert_eq!(verdicts[1], untyped_getlock());
}
