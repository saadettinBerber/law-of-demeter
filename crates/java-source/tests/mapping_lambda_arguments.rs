mod mapped;

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::{Creation, Expression, Lambda, Method};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};
use mapped::{call, class, method_f_in, parameter};

fn shape(name: &str) -> TypeShape {
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

fn consumer_of(argument: TypeRef) -> TypeRef {
    generic("Consumer", &[argument])
}

fn event() -> TypeRef {
    class("Event")
}

fn entry() -> TypeRef {
    class("Entry")
}

fn order() -> TypeRef {
    class("Order")
}

fn world() -> Types {
    Types::new()
        .with_shape(
            interface("Consumer", &["T"]).with_method(abstract_method("accept", &[variable("T")])),
        )
        .with_shape(
            interface("BiConsumer", &["T", "U"])
                .with_method(abstract_method("accept", &[variable("T"), variable("U")])),
        )
        .with_shape(interface("Predicate", &["T"]).with_method(
            abstract_method("test", &[variable("T")]).returning(TypeRef::primitive("boolean")),
        ))
        .with_shape(
            interface("Function", &["T", "R"])
                .with_method(abstract_method("apply", &[variable("T")]).returning(variable("R"))),
        )
        .with_shape(interface("List", &["E"]))
        .with_shape(shape("Order"))
}

fn abstract_method(name: &str, parameters: &[TypeRef]) -> MethodShape {
    MethodShape::new(name)
        .with_parameters(parameters.to_vec())
        .as_abstract()
}

fn method(name: &str, parameters: Vec<TypeRef>) -> MethodShape {
    MethodShape::new(name).with_parameters(parameters)
}

fn worker_with(methods: Vec<MethodShape>) -> Types {
    let worker = methods
        .into_iter()
        .fold(shape("Worker"), TypeShape::with_method);

    world().with_shape(worker)
}

fn f(line: usize) -> Method {
    Method::new("OrderService", "f", line)
}

fn mapped_in(types: &Types, parameters: &str, body: &str) -> Method {
    let source = format!("class OrderService {{\n void f({parameters}) {{\n  {body}\n }}\n}}");

    method_f_in(&source, types)
}

fn reading(parameter: &str) -> Expression {
    Expression::return_of(call(Expression::local(parameter), "getEntry", 3))
}

fn listener(parameter: &str) -> Lambda {
    Lambda::new(&[parameter]).yielding(reading(parameter))
}

fn fed(lambda: Lambda, parameter: &str) -> Lambda {
    lambda.fed_by_receiver(parameter)
}

fn worker() -> Expression {
    parameter("worker", class("Worker"))
}

fn running(lambda: Lambda) -> Method {
    f(2).call(call(worker(), "run", 3).with(lambda))
}

fn running_linked(lambda: Lambda) -> Method {
    f(2).call(call(worker(), "run", 3).with(lambda).returning_from(&[]))
}

#[test]
fn a_lambda_argument_resolves_its_parameter_from_the_method_parameter_type() {
    let types = worker_with(vec![method("run", vec![consumer_of(event())])]);

    let mapped = mapped_in(&types, "Worker worker", "worker.run(e -> e.getEntry());");

    assert_eq!(
        mapped,
        running_linked(fed(listener("e").resolving("e", event()), "e"))
    );
}

#[test]
fn the_overload_with_the_same_parameter_count_types_the_lambda() {
    let types = worker_with(vec![
        method("run", vec![consumer_of(event())]),
        method("run", vec![generic("BiConsumer", &[event(), entry()])]),
    ]);

    let mapped = mapped_in(
        &types,
        "Worker worker",
        "worker.run((e, n) -> e.getEntry());",
    );

    let lambda = Lambda::new(&["e", "n"])
        .resolving("e", event())
        .resolving("n", entry())
        .fed_by_receiver("e")
        .fed_by_receiver("n")
        .yielding(reading("e"));
    assert_eq!(mapped, running_linked(lambda));
}

#[test]
fn overloads_that_fit_the_lambda_equally_leave_it_unresolved() {
    let types = worker_with(vec![
        method("run", vec![consumer_of(event())]),
        method("run", vec![consumer_of(entry())]),
    ]);

    let mapped = mapped_in(&types, "Worker worker", "worker.run(e -> e.getEntry());");

    assert_eq!(mapped, running(listener("e")));
}

#[test]
fn a_non_functional_parameter_does_not_fit_a_lambda() {
    let types = worker_with(vec![
        method("run", vec![event()]),
        method("run", vec![consumer_of(entry())]),
    ]);

    let mapped = mapped_in(&types, "Worker worker", "worker.run(e -> e.getEntry());");

    assert_eq!(
        mapped,
        running_linked(fed(listener("e").resolving("e", entry()), "e"))
    );
}

#[test]
fn a_type_variable_parameter_also_fits_the_lambda_and_keeps_the_overload_ambiguous() {
    let generic_run = method("run", vec![variable("T")]).with_type_parameter("T");
    let types = worker_with(vec![generic_run, method("run", vec![consumer_of(event())])]);

    let mapped = mapped_in(&types, "Worker worker", "worker.run(e -> e.getEntry());");

    assert_eq!(mapped, running(listener("e")));
}

#[test]
fn the_other_arguments_choose_the_overload_that_types_the_lambda() {
    let types = worker_with(vec![
        method("run", vec![class("java.lang.String"), consumer_of(event())]),
        method("run", vec![TypeRef::primitive("int"), consumer_of(entry())]),
    ]);

    let mapped = mapped_in(
        &types,
        "Worker worker",
        "worker.run(\"fast\", e -> e.getEntry());",
    );

    let run = call(worker(), "run", 3)
        .with(Expression::literal())
        .with(fed(listener("e").resolving("e", event()), "e"))
        .returning_from(&[]);
    assert_eq!(mapped, f(2).call(run));
}

#[test]
fn an_argument_of_unknown_type_still_lets_the_only_overload_type_its_lambda() {
    let types = worker_with(vec![method("run", vec![event(), consumer_of(event())])]);

    let mapped = mapped_in(
        &types,
        "Worker worker, Mystery mystery",
        "worker.run(mystery, e -> e.getEntry());",
    );

    let typed = fed(listener("e").resolving("e", event()), "e");
    let run = call(worker(), "run", 3)
        .with(parameter("mystery", class("Mystery")))
        .with(typed)
        .returning_from(&[]);
    assert_eq!(mapped, f(2).call(run));
}

#[test]
fn a_type_variable_bound_by_another_argument_types_the_lambda() {
    let each = MethodShape::new("each")
        .as_static()
        .with_type_parameter("T")
        .with_parameters(vec![
            generic("List", &[variable("T")]),
            consumer_of(variable("T")),
        ]);
    let types = world().with_shape(shape("Util").with_method(each));

    let mapped = mapped_in(
        &types,
        "List<Order> orders",
        "Util.each(orders, o -> o.getEntry());",
    );

    let each = call(Expression::static_type(class("Util")), "each", 3)
        .with(parameter("orders", generic("List", &[order()])))
        .with(fed(listener("o").resolving("o", order()), "o"))
        .returning_from(&[]);
    assert_eq!(mapped, f(2).call(each));
}

#[test]
fn a_type_variable_left_open_by_the_lambda_does_not_stop_its_parameters_from_resolving() {
    let apply = MethodShape::new("apply")
        .as_static()
        .with_type_parameter("T")
        .with_type_parameter("R")
        .with_parameters(vec![
            generic("List", &[variable("T")]),
            generic("Function", &[variable("T"), variable("R")]),
        ]);
    let types = world().with_shape(shape("Util").with_method(apply));

    let mapped = mapped_in(
        &types,
        "List<Order> orders",
        "Util.apply(orders, o -> o.getEntry());",
    );

    let apply = call(Expression::static_type(class("Util")), "apply", 3)
        .with(parameter("orders", generic("List", &[order()])))
        .with(fed(listener("o").resolving("o", order()), "o"))
        .returning_from(&[]);
    assert_eq!(mapped, f(2).call(apply));
}

#[test]
fn a_lambda_argument_no_longer_leaves_the_return_type_undecided() {
    let first = MethodShape::new("first")
        .as_static()
        .with_type_parameter("T")
        .with_parameters(vec![
            generic("List", &[variable("T")]),
            generic("Predicate", &[variable("T")]),
        ])
        .returning(variable("T"));
    let types = world().with_shape(shape("Util").with_method(first));

    let mapped = mapped_in(
        &types,
        "List<Order> orders",
        "Util.first(orders, o -> true).go();",
    );

    let first = call(Expression::static_type(class("Util")), "first", 3)
        .with(parameter("orders", generic("List", &[order()])))
        .with(
            Lambda::new(&["o"])
                .resolving("o", order())
                .fed_by_receiver("o")
                .yielding(Expression::literal()),
        )
        .returning(order())
        .returning_from(&[]);
    assert_eq!(mapped, f(2).call(first.then("go", 3)));
}

#[test]
fn a_variable_arity_method_types_every_lambda_it_takes() {
    let types = worker_with(vec![
        method("run", vec![TypeRef::array(consumer_of(event()))]).as_varargs(),
    ]);

    let mapped = mapped_in(
        &types,
        "Worker worker",
        "worker.run(e -> e.getEntry(), d -> d.getEntry());",
    );

    let run = call(worker(), "run", 3)
        .with(fed(listener("e").resolving("e", event()), "e"))
        .with(fed(listener("d").resolving("d", event()), "d"))
        .returning_from(&[]);
    assert_eq!(mapped, f(2).call(run));
}

fn constructor(parameters: Vec<TypeRef>) -> MethodShape {
    method("<init>", parameters)
}

fn created(types: &Types, source: &str) -> Method {
    mapped_in(types, "", source)
}

fn creating(created: TypeRef, lambda: Lambda) -> Method {
    f(2).evaluate(Expression::created(Creation::new(created).with(lambda)))
}

#[test]
fn a_constructor_argument_resolves_its_parameter_from_the_constructor() {
    let types =
        world().with_shape(shape("Worker").with_method(constructor(vec![consumer_of(event())])));

    let mapped = created(&types, "new Worker(e -> e.getEntry());");

    assert_eq!(
        mapped,
        creating(
            class("Worker"),
            fed(listener("e").resolving("e", event()), "e")
        )
    );
}

#[test]
fn constructors_that_fit_the_lambda_equally_leave_it_unresolved() {
    let worker = shape("Worker")
        .with_method(constructor(vec![consumer_of(event())]))
        .with_method(constructor(vec![consumer_of(entry())]));
    let types = world().with_shape(worker);

    let mapped = created(&types, "new Worker(e -> e.getEntry());");

    assert_eq!(mapped, creating(class("Worker"), listener("e")));
}

#[test]
fn the_class_type_arguments_substitute_into_the_constructor_parameter() {
    let handler = shape("Handler")
        .with_type_parameter("T")
        .with_method(constructor(vec![consumer_of(variable("T"))]));
    let types = world().with_shape(handler);

    let mapped = created(&types, "new Handler<Event>(e -> e.getEntry());");

    assert_eq!(
        mapped,
        creating(
            generic("Handler", &[event()]),
            fed(listener("e").resolving("e", event()), "e")
        )
    );
}

#[test]
fn a_diamond_leaves_the_constructor_lambda_unresolved() {
    let handler = shape("Handler")
        .with_type_parameter("T")
        .with_method(constructor(vec![consumer_of(variable("T"))]));
    let types = world().with_shape(handler);

    let mapped = created(&types, "new Handler<>(e -> e.getEntry());");

    assert_eq!(mapped, creating(class("Handler"), fed(listener("e"), "e")));
}

#[test]
fn a_diamond_takes_its_type_arguments_from_the_declared_local_for_its_lambda() {
    let handler = shape("Handler")
        .with_type_parameter("T")
        .with_method(constructor(vec![consumer_of(variable("T"))]));
    let types = world().with_shape(handler);

    let mapped = created(
        &types,
        "Handler<Event> handler = new Handler<>(e -> e.getEntry());",
    );

    let lambda = fed(listener("e").resolving("e", event()), "e");
    assert_eq!(
        mapped,
        f(2).assign(
            "handler",
            generic("Handler", &[event()]),
            Expression::created(Creation::new(class("Handler")).with(lambda))
        )
    );
}

#[test]
fn the_constructors_of_a_supertype_do_not_type_the_lambda_of_a_subtype() {
    let types = world()
        .with_shape(shape("Worker").with_method(constructor(vec![consumer_of(event())])))
        .with_shape(shape("Helper").with_superclass(class("Worker")));

    let mapped = created(&types, "new Helper(e -> e.getEntry());");

    assert_eq!(mapped, creating(class("Helper"), listener("e")));
}

fn keyed_world() -> Types {
    let of = MethodShape::new("of")
        .as_static()
        .with_type_parameter("K")
        .with_type_parameter("V")
        .with_parameters(vec![variable("K"), variable("V")])
        .returning(generic("Map", &[variable("K"), variable("V")]));
    let wrap = MethodShape::new("wrap")
        .as_static()
        .with_type_parameter("T")
        .with_parameters(vec![variable("T")])
        .returning(variable("T"));
    world()
        .with_shape(interface("Map", &["K", "V"]))
        .with_shape(shape("Key"))
        .with_shape(shape("Util").with_method(of).with_method(wrap))
}

fn handlers() -> TypeRef {
    generic("Map", &[class("Key"), consumer_of(event())])
}

#[test]
fn the_expected_type_binds_the_variable_that_types_the_lambda() {
    let body = "Map<Key, Consumer<Event>> handlers = Util.of(key, e -> e.getEntry());";

    let mapped = mapped_in(&keyed_world(), "Key key", body);

    let of = call(Expression::static_type(class("Util")), "of", 3)
        .with(parameter("key", class("Key")))
        .with(listener("e").resolving("e", event()))
        .returning(handlers())
        .returning_from(&[]);
    assert_eq!(
        mapped,
        f(2).assign("handlers", handlers(), Expression::return_of(of))
    );
}

#[test]
fn a_bare_variable_return_takes_the_expected_type_for_its_lambda() {
    let body = "Consumer<Event> handler = Util.wrap(e -> e.getEntry());";

    let mapped = mapped_in(&keyed_world(), "", body);

    let wrap = call(Expression::static_type(class("Util")), "wrap", 3)
        .with(listener("e").resolving("e", event()))
        .returning(consumer_of(event()))
        .returning_from(&[]);
    assert_eq!(
        mapped,
        f(2).assign("handler", consumer_of(event()), Expression::return_of(wrap))
    );
}

#[test]
fn a_returned_call_takes_the_return_type_of_the_method_for_its_lambda() {
    let source = "class OrderService {\n Consumer<Event> f() {\n  return Util.wrap(e -> e.getEntry());\n }\n}";

    let mapped = method_f_in(source, &keyed_world());

    let wrap = call(Expression::static_type(class("Util")), "wrap", 3)
        .with(listener("e").resolving("e", event()))
        .returning(consumer_of(event()))
        .returning_from(&[]);
    assert_eq!(mapped, f(2).call(wrap));
}
