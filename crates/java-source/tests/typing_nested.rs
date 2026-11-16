mod judging;

use judging::verdicts_in_c;
use law_of_demeter_core::domain::classification::{Classification, Types};
use law_of_demeter_core::domain::method::{Reason, Verdict};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

const STREAM: &str = "java.util.stream.Stream";
const COLLECTOR: &str = "java.util.stream.Collector";
const COLLECTORS: &str = "java.util.stream.Collectors";
const LIST: &str = "java.util.List";
const MAP: &str = "java.util.Map";
const FUNCTION: &str = "java.util.function.Function";
const OBJECT: &str = "java.lang.Object";

const IMPORTS: &str = "import java.util.List; import java.util.stream.Collectors;";

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

fn status() -> TypeRef {
    TypeRef::class("Status")
}

fn wildcard() -> TypeRef {
    TypeRef::class(OBJECT)
}

fn stream_of(element: TypeRef) -> TypeRef {
    generic(STREAM, &[element])
}

fn list_of(element: TypeRef) -> TypeRef {
    generic(LIST, &[element])
}

fn collector(element: TypeRef, accumulation: TypeRef, result: TypeRef) -> TypeRef {
    generic(COLLECTOR, &[element, accumulation, result])
}

fn function(argument: TypeRef, result: TypeRef) -> TypeRef {
    generic(FUNCTION, &[argument, result])
}

fn collecting() -> Types {
    let collectors = class(COLLECTORS)
        .with_method(
            MethodShape::new("toList")
                .as_static()
                .with_type_parameter("T")
                .returning(collector(variable("T"), wildcard(), list_of(variable("T")))),
        )
        .with_method(
            MethodShape::new("collectingAndThen")
                .as_static()
                .with_type_parameter("T")
                .with_type_parameter("A")
                .with_type_parameter("R")
                .with_type_parameter("RR")
                .with_parameters(vec![
                    collector(variable("T"), variable("A"), variable("R")),
                    function(variable("R"), variable("RR")),
                ])
                .returning(collector(variable("T"), variable("A"), variable("RR"))),
        )
        .with_method(
            MethodShape::new("groupingBy")
                .as_static()
                .with_type_parameter("T")
                .with_type_parameter("K")
                .with_type_parameter("A")
                .with_type_parameter("D")
                .with_parameters(vec![
                    function(variable("T"), variable("K")),
                    collector(variable("T"), variable("A"), variable("D")),
                ])
                .returning(collector(
                    variable("T"),
                    wildcard(),
                    generic(MAP, &[variable("K"), variable("D")]),
                )),
        )
        .with_method(
            MethodShape::new("groupingBy")
                .as_static()
                .with_type_parameter("T")
                .with_type_parameter("K")
                .with_parameters(vec![function(variable("T"), variable("K"))])
                .returning(collector(
                    variable("T"),
                    wildcard(),
                    generic(MAP, &[variable("K"), list_of(variable("T"))]),
                )),
        );
    Types::new()
        .with_shape(collectors)
        .with_shape(interface(COLLECTOR, &["T", "A", "R"]))
        .with_shape(
            interface(STREAM, &["T"])
                .with_method(
                    MethodShape::new("collect")
                        .with_type_parameter("R")
                        .with_type_parameter("A")
                        .with_parameters(vec![collector(
                            variable("T"),
                            variable("A"),
                            variable("R"),
                        )])
                        .returning(variable("R")),
                )
                .with_method(
                    MethodShape::new("map")
                        .with_type_parameter("R")
                        .with_parameters(vec![function(variable("T"), variable("R"))])
                        .returning(stream_of(variable("R"))),
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
            interface(LIST, &["E"])
                .with_method(MethodShape::new("stream").returning(stream_of(variable("E"))))
                .with_method(
                    MethodShape::new("get")
                        .with_parameters(vec![TypeRef::primitive("int")])
                        .returning(variable("E")),
                ),
        )
        .with_shape(
            interface(MAP, &["K", "V"]).with_method(
                MethodShape::new("get")
                    .with_parameters(vec![wildcard()])
                    .returning(variable("V")),
            ),
        )
        .with_shape(
            class("Order")
                .with_method(MethodShape::new("getCustomer").returning(customer()))
                .with_method(MethodShape::new("getStatus").returning(status()))
                .with_method(MethodShape::new("ship")),
        )
        .with_shape(
            class("Customer")
                .with_method(MethodShape::new("getWallet").returning(TypeRef::class("Wallet"))),
        )
        .with_shape(class("Wallet").with_method(MethodShape::new("withdraw")))
        .with_shape(class("Status"))
}

fn collectors_static() -> Reason {
    Reason::static_type(TypeRef::class(COLLECTORS))
}

#[test]
fn collecting_a_stream_by_a_method_reference_types_the_groups() {
    let snippet = "void ship(List<Order> orders, Status status) {\n    orders.stream().collect(Collectors.groupingBy(Order::getStatus)).get(status).get(0).getCustomer().rename();\n}";

    let verdicts = verdicts_in_c(IMPORTS, "", snippet, "ship", &collecting());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 3, Reason::parameter("orders")),
            Verdict::allowed(
                "getStatus",
                3,
                Reason::method_reference(order(), "getStatus", Reason::parameter("orders"))
            ),
            Verdict::allowed("groupingBy", 3, collectors_static()),
            Verdict::allowed("collect", 3, Reason::parameter("orders")),
            Verdict::allowed("get", 3, Reason::parameter("orders")),
            Verdict::allowed("get", 3, Reason::parameter("orders")),
            Verdict::allowed("getCustomer", 3, Reason::parameter("orders")),
            Verdict::violation("rename", 3, Reason::return_of("getCustomer")),
        ]
    );
}

#[test]
fn a_collector_wrapping_another_gets_its_result_from_its_finisher() {
    let snippet = "void ship(List<Order> orders) {\n    orders.stream().collect(Collectors.collectingAndThen(Collectors.toList(), list -> list.get(0))).getCustomer().rename();\n}";

    let verdicts = verdicts_in_c(IMPORTS, "", snippet, "ship", &collecting());

    assert_eq!(
        verdicts.last(),
        Some(&Verdict::violation(
            "rename",
            3,
            Reason::return_of("getCustomer")
        ))
    );
}

#[test]
fn a_lambda_in_a_nested_call_takes_the_standing_of_the_stranger_elements_it_receives() {
    let snippet = "void group(List<Order> orders) {\n    orders.stream().map(Order::getCustomer).collect(Collectors.groupingBy(each -> each.getWallet()));\n}";

    let verdicts = verdicts_in_c(IMPORTS, "", snippet, "group", &collecting());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 3, Reason::parameter("orders")),
            Verdict::allowed(
                "getCustomer",
                3,
                Reason::method_reference(order(), "getCustomer", Reason::parameter("orders"))
            ),
            Verdict::allowed("map", 3, Reason::parameter("orders")),
            Verdict::violation(
                "getWallet",
                3,
                Reason::lambda_parameter("each", Reason::return_of("getCustomer"))
            ),
            Verdict::allowed("groupingBy", 3, collectors_static()),
            Verdict::allowed(
                "collect",
                3,
                Reason::classified(Classification::Container, stream_of(customer()))
            ),
        ]
    );
}

#[test]
fn a_lambda_in_a_nested_call_takes_the_standing_of_the_friend_elements_it_receives() {
    let snippet = "void group(List<Order> orders) {\n    orders.stream().collect(Collectors.groupingBy(each -> each.getStatus()));\n}";

    let verdicts = verdicts_in_c(IMPORTS, "", snippet, "group", &collecting());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 3, Reason::parameter("orders")),
            Verdict::allowed(
                "getStatus",
                3,
                Reason::lambda_parameter("each", Reason::parameter("orders"))
            ),
            Verdict::allowed("groupingBy", 3, collectors_static()),
            Verdict::allowed("collect", 3, Reason::parameter("orders")),
        ]
    );
}

#[test]
fn a_lambda_in_a_nested_call_that_receives_something_else_keeps_its_own_feed() {
    let snippet = "void ship(List<Order> orders) {\n    orders.stream().collect(Collectors.collectingAndThen(Collectors.toList(), list -> list.get(0)));\n}";

    let verdicts = verdicts_in_c(IMPORTS, "", snippet, "ship", &collecting());

    assert!(verdicts.contains(&Verdict::allowed(
        "get",
        3,
        Reason::lambda_parameter("list", collectors_static())
    )));
}

#[test]
fn collectors_nested_two_levels_deep_type_the_groups() {
    let snippet = "void ship(List<Order> orders, Status status) {\n    orders.stream().collect(Collectors.groupingBy(Order::getStatus, Collectors.toList())).get(status).get(0).getCustomer().rename();\n}";

    let verdicts = verdicts_in_c(IMPORTS, "", snippet, "ship", &collecting());

    assert_eq!(
        verdicts.last(),
        Some(&Verdict::violation(
            "rename",
            3,
            Reason::return_of("getCustomer")
        ))
    );
}
