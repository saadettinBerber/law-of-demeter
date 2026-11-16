mod mapped;

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::{Expression, Lambda, Method};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};
use mapped::{call, class, method_f_in};

fn interface(name: &str, parameters: &[&str]) -> TypeShape {
    parameters.iter().fold(
        TypeShape::new(name, TypeKind::Interface),
        |shape, parameter| shape.with_type_parameter(parameter),
    )
}

fn consumer_of(argument: TypeRef) -> TypeRef {
    TypeRef::generic("Consumer", vec![argument])
}

fn customer() -> TypeRef {
    class("Customer")
}

fn accepting(parameters: Vec<TypeRef>) -> MethodShape {
    MethodShape::new("accept")
        .with_parameters(parameters)
        .as_abstract()
}

fn world() -> Types {
    Types::new().with_shape(
        interface("Consumer", &["T"]).with_method(accepting(vec![TypeRef::variable("T")])),
    )
}

fn f(line: usize) -> Method {
    Method::new("OrderService", "f", line)
}

fn paying(line: usize) -> Expression {
    let wallet = call(Expression::local("c"), "getWallet", line);
    Expression::return_of(call(Expression::return_of(wallet), "pay", line))
}

fn payer() -> Lambda {
    Lambda::new(&["c"]).yielding(paying(3))
}

fn mapped_in(types: &Types, members: &str, body: &str) -> Method {
    let source = format!("class OrderService {{ {members}\n void f() {{\n  {body}\n }}\n}}");

    method_f_in(&source, types)
}

#[test]
fn a_lambda_assigned_to_a_declared_local_resolves_its_parameter_from_the_target() {
    let body = "Consumer<Customer> run = c -> c.getWallet().pay();";

    let mapped = mapped_in(&world(), "", body);

    assert_eq!(
        mapped,
        f(2).assign(
            "run",
            consumer_of(customer()),
            Expression::lambda(payer().resolving("c", customer()))
        )
    );
}

#[test]
fn a_lambda_assigned_to_a_field_resolves_its_parameter_from_the_field_type() {
    let members = "Consumer<Customer> run;";
    let body = "run = c -> c.getWallet().pay();";

    let mapped = mapped_in(&world(), members, body);

    let lambda = payer().resolving("c", customer());
    assert_eq!(
        mapped,
        f(2).evaluate(Expression::lambda(
            lambda.expecting(consumer_of(customer()))
        ))
    );
}

#[test]
fn a_returned_lambda_resolves_its_parameter_from_the_method_return_type() {
    let source = "class OrderService {\n Consumer<Customer> f() {\n  return c -> c.getWallet().pay();\n }\n}";

    let mapped = method_f_in(source, &world());

    let lambda = payer().resolving("c", customer());
    assert_eq!(
        mapped,
        f(2).evaluate(Expression::lambda(
            lambda.expecting(consumer_of(customer()))
        ))
    );
}

#[test]
fn a_cast_lambda_resolves_its_parameter_from_the_cast_type() {
    let body = "Object run = (Consumer<Customer>) c -> c.getWallet().pay();";

    let mapped = mapped_in(&world(), "", body);

    let lambda = payer().resolving("c", customer());
    assert_eq!(
        mapped,
        f(2).assign(
            "run",
            class("Object"),
            Expression::cast(consumer_of(customer()), Expression::lambda(lambda))
        )
    );
}

#[test]
fn a_declared_parameter_type_is_kept_instead_of_the_resolved_one() {
    let body = "Consumer<Customer> run = (Premium c) -> c.getWallet().pay();";

    let mapped = mapped_in(&world(), "", body);

    assert_eq!(
        mapped,
        f(2).assign(
            "run",
            consumer_of(customer()),
            Expression::lambda(payer().declaring("c", class("Premium")))
        )
    );
}

#[test]
fn the_abstract_method_of_a_super_interface_resolves_with_the_substituted_type_arguments() {
    let types = world().with_shape(interface("Reaction", &[]).extending(consumer_of(customer())));
    let body = "Reaction run = c -> c.getWallet().pay();";

    let mapped = mapped_in(&types, "", body);

    assert_eq!(
        mapped,
        f(2).assign(
            "run",
            class("Reaction"),
            Expression::lambda(payer().resolving("c", customer()))
        )
    );
}

#[test]
fn default_static_and_object_methods_do_not_count_as_the_single_abstract_method() {
    let check = interface("Check", &["T"])
        .with_method(accepting(vec![TypeRef::variable("T")]))
        .with_method(MethodShape::new("negate"))
        .with_method(MethodShape::new("of").as_static().as_abstract())
        .with_method(
            MethodShape::new("equals")
                .with_parameters(vec![class("java.lang.Object")])
                .as_abstract(),
        )
        .with_method(MethodShape::new("hashCode").as_abstract())
        .with_method(MethodShape::new("toString").as_abstract());
    let types = world().with_shape(check);
    let body = "Check<Customer> run = c -> c.getWallet().pay();";

    let mapped = mapped_in(&types, "", body);

    assert_eq!(
        mapped,
        f(2).assign(
            "run",
            TypeRef::generic("Check", vec![customer()]),
            Expression::lambda(payer().resolving("c", customer()))
        )
    );
}

#[test]
fn an_interface_with_two_abstract_methods_leaves_the_parameter_unresolved() {
    let two = interface("Two", &[])
        .with_method(accepting(vec![customer()]))
        .with_method(
            MethodShape::new("reject")
                .with_parameters(vec![customer()])
                .as_abstract(),
        );
    let types = world().with_shape(two);
    let body = "Two run = c -> c.getWallet().pay();";

    let mapped = mapped_in(&types, "", body);

    assert_eq!(
        mapped,
        f(2).assign("run", class("Two"), Expression::lambda(payer()))
    );
}

#[test]
fn a_class_target_leaves_the_parameter_unresolved() {
    let abstract_class =
        TypeShape::new("Base", TypeKind::Class).with_method(accepting(vec![customer()]));
    let types = world().with_shape(abstract_class);
    let body = "Base run = c -> c.getWallet().pay();";

    let mapped = mapped_in(&types, "", body);

    assert_eq!(
        mapped,
        f(2).assign("run", class("Base"), Expression::lambda(payer()))
    );
}

#[test]
fn a_target_with_another_parameter_count_leaves_the_parameter_unresolved() {
    let body = "Consumer<Customer> run = (c, d) -> c.getWallet().pay();";

    let mapped = mapped_in(&world(), "", body);

    let lambda = Lambda::new(&["c", "d"]).yielding(paying(3));
    assert_eq!(
        mapped,
        f(2).assign("run", consumer_of(customer()), Expression::lambda(lambda))
    );
}

#[test]
fn a_raw_target_leaves_the_parameter_unresolved() {
    let body = "Consumer run = c -> c.getWallet().pay();";

    let mapped = mapped_in(&world(), "", body);

    assert_eq!(
        mapped,
        f(2).assign("run", class("Consumer"), Expression::lambda(payer()))
    );
}

fn reading_world() -> Types {
    let event = TypeShape::new("Event", TypeKind::Class)
        .with_method(MethodShape::new("getEntry").returning(class("Entry")));
    let entry = TypeShape::new("Entry", TypeKind::Class)
        .with_method(MethodShape::new("getLock").returning(class("Lock")));
    let premium = TypeShape::new("Premium", TypeKind::Class)
        .with_method(MethodShape::new("getEntry").returning(class("Special")));
    world()
        .with_shape(event)
        .with_shape(entry)
        .with_shape(premium)
}

fn event_entry(parameter: &str, line: usize) -> Expression {
    let entry = call(Expression::local(parameter), "getEntry", line).returning(class("Entry"));
    Expression::return_of(entry)
}

fn consumer_of_event() -> TypeRef {
    consumer_of(class("Event"))
}

#[test]
fn a_call_on_a_resolved_lambda_parameter_gets_its_return_type() {
    let body = "Consumer<Event> run = e -> e.getEntry().getLock();";

    let mapped = mapped_in(&reading_world(), "", body);

    let lock = call(event_entry("e", 3), "getLock", 3).returning(class("Lock"));
    let lambda = Lambda::new(&["e"])
        .resolving("e", class("Event"))
        .yielding(Expression::return_of(lock));
    assert_eq!(
        mapped,
        f(2).assign("run", consumer_of_event(), Expression::lambda(lambda))
    );
}

#[test]
fn a_call_in_a_block_body_gets_the_type_of_the_resolved_parameter() {
    let body = "Consumer<Event> run = e -> {\n   e.getEntry();\n  };";

    let mapped = mapped_in(&reading_world(), "", body);

    let entry = call(Expression::local("e"), "getEntry", 4).returning(class("Entry"));
    let lambda = Lambda::new(&["e"])
        .resolving("e", class("Event"))
        .call(entry);
    assert_eq!(
        mapped,
        f(2).assign("run", consumer_of_event(), Expression::lambda(lambda))
    );
}

#[test]
fn a_declared_parameter_type_types_the_calls_in_the_body() {
    let body = "Consumer<Event> run = (Premium e) -> e.getEntry();";

    let mapped = mapped_in(&reading_world(), "", body);

    let special = call(Expression::local("e"), "getEntry", 3).returning(class("Special"));
    let lambda = Lambda::new(&["e"])
        .declaring("e", class("Premium"))
        .yielding(Expression::return_of(special));
    assert_eq!(
        mapped,
        f(2).assign("run", consumer_of_event(), Expression::lambda(lambda))
    );
}

#[test]
fn an_inner_lambda_sees_the_parameter_of_the_outer_one() {
    let body = "Consumer<Event> run = e -> {\n   Consumer<Entry> inner = n -> e.getEntry();\n  };";

    let mapped = mapped_in(&reading_world(), "", body);

    let inner = Lambda::new(&["n"])
        .resolving("n", class("Entry"))
        .yielding(event_entry("e", 4));
    let outer = Lambda::new(&["e"]).resolving("e", class("Event")).assign(
        "inner",
        consumer_of(class("Entry")),
        Expression::lambda(inner),
    );
    assert_eq!(
        mapped,
        f(2).assign("run", consumer_of_event(), Expression::lambda(outer))
    );
}

#[test]
fn the_parameter_types_of_a_lambda_do_not_leak_into_the_next_one() {
    let body = "Consumer<Event> run = e -> e.getEntry();\n  unknown.go(e -> e.getEntry());";

    let mapped = mapped_in(&reading_world(), "", body);

    let second =
        call(Expression::unresolved("unknown"), "go", 4).with(Lambda::new(&["e"]).yielding(
            Expression::return_of(call(Expression::local("e"), "getEntry", 4)),
        ));
    let first = Lambda::new(&["e"])
        .resolving("e", class("Event"))
        .yielding(event_entry("e", 3));
    assert_eq!(
        mapped,
        f(2).assign("run", consumer_of_event(), Expression::lambda(first))
            .call(second)
    );
}
