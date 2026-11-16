mod judging;

use judging::verdicts_in_c;
use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::{Reason, Verdict};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

const STREAM: &str = "java.util.stream.Stream";
const COLLECTOR: &str = "java.util.stream.Collector";
const COLLECTORS: &str = "java.util.stream.Collectors";
const MAP: &str = "java.util.Map";
const LIST: &str = "java.util.List";
const FUNCTION: &str = "java.util.function.Function";
const CONSUMER: &str = "java.util.function.Consumer";

const LIST_IMPORT: &str = "import java.util.List;";

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

fn customer() -> TypeRef {
    TypeRef::class("Customer")
}

fn wallets() -> TypeRef {
    TypeRef::class("Wallets")
}

fn stream_of(element: TypeRef) -> TypeRef {
    generic(STREAM, &[element])
}

fn streams() -> Types {
    Types::new()
        .with_shape(
            interface(STREAM, &["T"])
                .with_method(
                    MethodShape::new("map")
                        .with_type_parameter("R")
                        .with_parameters(vec![generic(FUNCTION, &[variable("T"), variable("R")])])
                        .returning(stream_of(variable("R"))),
                )
                .with_method(
                    MethodShape::new("forEach")
                        .with_parameters(vec![generic(CONSUMER, &[variable("T")])]),
                )
                .with_method(
                    MethodShape::new("collect")
                        .with_type_parameter("R")
                        .with_type_parameter("A")
                        .with_parameters(vec![generic(
                            COLLECTOR,
                            &[variable("T"), variable("A"), variable("R")],
                        )])
                        .returning(variable("R")),
                ),
        )
        .with_shape(interface(COLLECTOR, &["T", "A", "R"]))
        .with_shape(
            class(COLLECTORS).with_method(
                MethodShape::new("groupingBy")
                    .as_static()
                    .with_type_parameter("T")
                    .with_type_parameter("K")
                    .with_parameters(vec![generic(FUNCTION, &[variable("T"), variable("K")])])
                    .returning(generic(
                        COLLECTOR,
                        &[
                            variable("T"),
                            TypeRef::class("java.lang.Object"),
                            generic(MAP, &[variable("K"), generic(LIST, &[variable("T")])]),
                        ],
                    )),
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
            interface(CONSUMER, &["T"]).with_method(
                MethodShape::new("accept")
                    .with_parameters(vec![variable("T")])
                    .as_abstract(),
            ),
        )
        .with_shape(
            interface(LIST, &["E"]).with_method(
                MethodShape::new("stream").returning(generic(STREAM, &[variable("E")])),
            ),
        )
        .with_shape(
            class("Order").with_method(MethodShape::new("getCustomer").returning(customer())),
        )
        .with_shape(
            class("Customer")
                .with_method(MethodShape::new("getWallet").returning(TypeRef::class("Wallet"))),
        )
        .with_shape(class("Wallet").with_method(MethodShape::new("withdraw")))
        .with_shape(
            class("Customers").with_method(
                MethodShape::new("of")
                    .as_static()
                    .with_parameters(vec![order()])
                    .returning(customer()),
            ),
        )
        .with_shape(class("PremiumCustomers").with_superclass(TypeRef::class("Customers")))
        .with_shape(
            class("Wallets").with_method(
                MethodShape::new("of")
                    .as_static()
                    .with_parameters(vec![customer()])
                    .returning(TypeRef::class("Wallet")),
            ),
        )
}

#[test]
fn a_reference_to_a_static_method_is_allowed_as_the_static_call_it_stands_for() {
    let snippet = "void charge(List<Order> orders) {\n    orders.stream().map(Customers::of);\n}";

    let verdicts = verdicts_in_c(LIST_IMPORT, "", snippet, "charge", &streams());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 3, Reason::parameter("orders")),
            Verdict::allowed("of", 3, Reason::static_type(TypeRef::class("Customers"))),
            Verdict::allowed("map", 3, Reason::parameter("orders")),
        ]
    );
}

#[test]
fn a_reference_to_a_static_method_is_allowed_whoever_feeds_it() {
    let snippet = "void charge(List<Order> orders) {\n    orders.stream().map(order -> order.getCustomer()).map(Wallets::of);\n}";

    let verdicts = verdicts_in_c(LIST_IMPORT, "", snippet, "charge", &streams());

    assert_eq!(
        verdicts[3],
        Verdict::allowed("of", 3, Reason::static_type(wallets()))
    );
}

#[test]
fn a_reference_through_a_subtype_to_an_inherited_static_method_is_static() {
    let snippet =
        "void charge(List<Order> orders) {\n    orders.stream().map(PremiumCustomers::of);\n}";

    let verdicts = verdicts_in_c(LIST_IMPORT, "", snippet, "charge", &streams());

    assert_eq!(
        verdicts[1],
        Verdict::allowed(
            "of",
            3,
            Reason::static_type(TypeRef::class("PremiumCustomers"))
        )
    );
}

fn the_customers_orders_were_taken_from() -> Reason {
    Reason::method_reference(order(), "getCustomer", Reason::parameter("orders"))
}

#[test]
fn an_unbound_reference_is_judged_as_the_call_on_the_element_that_feeds_it() {
    let snippet =
        "void charge(List<Order> orders) {\n    orders.stream().map(Order::getCustomer);\n}";

    let verdicts = verdicts_in_c(LIST_IMPORT, "", snippet, "charge", &streams());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 3, Reason::parameter("orders")),
            Verdict::allowed("getCustomer", 3, the_customers_orders_were_taken_from()),
            Verdict::allowed("map", 3, Reason::parameter("orders")),
        ]
    );
}

#[test]
fn an_unbound_reference_that_cannot_be_typed_is_judged_like_an_untyped_lambda_parameter() {
    let snippet =
        "void charge() {\n    Handlers handlers;\n    handlers.register(Order::getCustomer);\n}";

    let verdicts = verdicts_in_c("", "", snippet, "charge", &streams());

    assert_eq!(
        verdicts,
        [
            Verdict::violation(
                "getCustomer",
                4,
                Reason::unresolved_type(
                    None,
                    Reason::method_reference(
                        order(),
                        "getCustomer",
                        Reason::unassigned_local("handlers")
                    )
                )
            ),
            Verdict::violation(
                "register",
                4,
                Reason::unresolved_type(None, Reason::unassigned_local("handlers"))
            ),
        ]
    );
}

#[test]
fn a_constructor_reference_is_not_a_call() {
    let snippet = "void charge(List<Order> orders) {\n    orders.stream().map(Customer::new);\n}";

    let verdicts = verdicts_in_c(LIST_IMPORT, "", snippet, "charge", &streams());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 3, Reason::parameter("orders")),
            Verdict::allowed("map", 3, Reason::parameter("orders")),
        ]
    );
}

#[test]
fn a_bound_reference_on_a_field_is_allowed_as_the_call_on_the_field_would_be() {
    let snippet =
        "void print(List<Order> orders) {\n    orders.stream().forEach(printer::print);\n}";

    let verdicts = verdicts_in_c(
        LIST_IMPORT,
        "Printer printer;",
        snippet,
        "print",
        &streams(),
    );

    assert_eq!(
        verdicts[1],
        Verdict::allowed("print", 3, Reason::field("printer"))
    );
}

#[test]
fn a_bound_reference_on_a_static_field_is_allowed_as_the_call_on_it_would_be() {
    let snippet =
        "void print(List<Order> orders) {\n    orders.stream().forEach(System.out::println);\n}";
    let types = streams().with_shape(class("java.lang.System"));

    let verdicts = verdicts_in_c(LIST_IMPORT, "", snippet, "print", &types);

    assert_eq!(
        verdicts[1],
        Verdict::allowed("println", 3, Reason::static_result("out"))
    );
}
