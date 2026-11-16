mod judging;

use judging::verdicts_in_c;
use law_of_demeter_core::domain::classification::{Classification, Types};
use law_of_demeter_core::domain::method::{Reason, Verdict};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

const ARRAYS: &str = "java.util.Arrays";
const STREAM: &str = "java.util.stream.Stream";
const CONSUMER: &str = "java.util.function.Consumer";
const COLLECTION: &str = "java.util.Collection";
const ARRAY_LIST: &str = "java.util.ArrayList";
const STREAMS: &str = "shop.Streams";
const LIST: &str = "java.util.List";
const FUNCTION: &str = "java.util.function.Function";
const BIG_DECIMAL: &str = "java.math.BigDecimal";
const PREDICATE: &str = "java.util.function.Predicate";
const MAP: &str = "java.util.Map";
const BI_CONSUMER: &str = "java.util.function.BiConsumer";

const LIST_IMPORT: &str = "import java.util.List;";
const MAP_IMPORTS: &str = "import java.util.Map; import java.util.function.Consumer;";
const HANDLERS: &str = "Map<String, Consumer<Event>>";
const REGISTERING_MAP_OF: &str = "Map.of(\"created\", event -> { event.getReceipt().close(); })";

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
        .with_shape(
            interface(STREAM, &["T"])
                .with_method(
                    MethodShape::new("forEach")
                        .with_parameters(vec![generic(CONSUMER, &[variable("T")])]),
                )
                .with_method(
                    MethodShape::new("filter")
                        .with_parameters(vec![generic(PREDICATE, &[variable("T")])])
                        .returning(generic(STREAM, &[variable("T")])),
                )
                .with_method(
                    MethodShape::new("fold")
                        .with_type_parameter("R")
                        .with_parameters(vec![generic(FUNCTION, &[variable("T"), variable("R")])])
                        .returning(variable("R")),
                )
                .with_method(
                    MethodShape::new("map")
                        .with_type_parameter("R")
                        .with_parameters(vec![generic(FUNCTION, &[variable("T"), variable("R")])])
                        .returning(generic(STREAM, &[variable("R")])),
                ),
        )
        .with_shape(
            interface(PREDICATE, &["T"]).with_method(
                MethodShape::new("test")
                    .with_parameters(vec![variable("T")])
                    .returning(TypeRef::primitive("boolean"))
                    .as_abstract(),
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
            interface(LIST, &["E"]).with_method(
                MethodShape::new("stream").returning(generic(STREAM, &[variable("E")])),
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
            class("Customers").with_method(
                MethodShape::new("of")
                    .as_static()
                    .with_parameters(vec![order()])
                    .returning(TypeRef::class("Customer")),
            ),
        )
        .with_shape(
            class("Registry").with_method(
                MethodShape::new("lookup")
                    .with_parameters(vec![order()])
                    .returning(TypeRef::class("Customer")),
            ),
        )
        .with_shape(
            class("Order")
                .with_method(MethodShape::new("getCustomer").returning(TypeRef::class("Customer")))
                .with_method(MethodShape::new("getTotal").returning(TypeRef::class(BIG_DECIMAL)))
                .with_method(MethodShape::new("isPaid").returning(TypeRef::primitive("boolean")))
                .with_method(MethodShape::new("ship"))
                .with_method(MethodShape::new("newReceipt").returning(TypeRef::class("Receipt"))),
        )
        .with_shape(class("Receipt").with_method(MethodShape::new("print")))
        .with_shape(
            class("Mapper").with_method(
                MethodShape::new("mapAll")
                    .with_type_parameter("R")
                    .with_parameters(vec![generic(FUNCTION, &[order(), variable("R")])])
                    .returning(generic(STREAM, &[variable("R")])),
            ),
        )
        .with_shape(
            class("Customer")
                .with_method(MethodShape::new("rename"))
                .with_method(MethodShape::new("getWallet").returning(TypeRef::class("Wallet"))),
        )
        .with_shape(class("Wallet").with_method(MethodShape::new("withdraw")))
}

#[test]
fn the_type_argument_of_a_generic_method_comes_from_an_array_argument() {
    let snippet = "void notify(Order[] orders) {\n    Arrays.stream(orders).forEach(order -> { order.getCustomer().rename(); });\n}";

    let verdicts = verdicts_in_c("import java.util.Arrays;", "", snippet, "notify", &shop());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 3, Reason::static_type(TypeRef::class(ARRAYS))),
            Verdict::allowed(
                "getCustomer",
                3,
                Reason::lambda_parameter("order", Reason::static_result("stream"))
            ),
            Verdict::violation("rename", 3, Reason::return_of("getCustomer")),
            Verdict::allowed("forEach", 3, Reason::static_result("stream")),
        ]
    );
}

#[test]
fn the_type_argument_of_a_generic_method_comes_through_the_supertype_of_the_argument() {
    let snippet = "void notify(ArrayList<Order> orders) {\n    Streams.from(orders).forEach(order -> { order.getCustomer().rename(); });\n}";

    let verdicts = verdicts_in_c(
        "import java.util.ArrayList; import shop.Streams;",
        "",
        snippet,
        "notify",
        &shop(),
    );

    assert_eq!(
        verdicts[2],
        Verdict::violation("rename", 3, Reason::return_of("getCustomer"))
    );
}

fn closing_the_receipt() -> Verdict {
    Verdict::violation("close", 3, Reason::return_of("getReceipt"))
}

#[test]
fn a_lambda_parameter_takes_its_type_from_the_functional_interface_the_parameter_declares() {
    let snippet = "void register() {\n    handlers.put(\"created\", event -> { event.getReceipt().close(); });\n}";

    let verdicts = verdicts_in_c(
        MAP_IMPORTS,
        &format!("{HANDLERS} handlers;"),
        snippet,
        "register",
        &shop(),
    );

    assert_eq!(verdicts[1], closing_the_receipt());
}

#[test]
fn each_lambda_parameter_takes_the_type_at_its_position_in_the_functional_interface() {
    let snippet = "void audit() {\n    ledger.forEach((customer, order) -> { customer.getWallet().withdraw(); });\n}";

    let verdicts = verdicts_in_c(
        "import java.util.Map;",
        "Map<Customer, Order> ledger;",
        snippet,
        "audit",
        &shop(),
    );

    assert_eq!(
        verdicts[1],
        Verdict::violation("withdraw", 3, Reason::return_of("getWallet"))
    );
}

#[test]
fn a_call_expecting_a_type_binds_its_type_variables_from_it() {
    let snippet = format!("{HANDLERS} register() {{\n    return {REGISTERING_MAP_OF};\n}}");

    let verdicts = verdicts_in_c(MAP_IMPORTS, "", &snippet, "register", &shop());

    assert_eq!(verdicts[1], closing_the_receipt());
}

#[test]
fn a_local_declared_with_a_type_gives_its_type_to_the_call_that_fills_it() {
    let snippet =
        format!("void register() {{\n    {HANDLERS} handlers = {REGISTERING_MAP_OF};\n}}");

    let verdicts = verdicts_in_c(MAP_IMPORTS, "", &snippet, "register", &shop());

    assert_eq!(verdicts[1], closing_the_receipt());
}

#[test]
fn a_cast_gives_its_type_to_the_call_it_wraps() {
    let snippet =
        format!("Object register() {{\n    return ({HANDLERS}) {REGISTERING_MAP_OF};\n}}");

    let verdicts = verdicts_in_c(MAP_IMPORTS, "", &snippet, "register", &shop());

    assert_eq!(verdicts[1], closing_the_receipt());
}

#[test]
fn the_expression_a_lambda_yields_is_judged_like_a_statement() {
    let snippet = "void charge(List<Order> orders) {\n    orders.stream().forEach(order -> order.getCustomer().rename());\n}";

    let verdicts = verdicts_in_c(LIST_IMPORT, "", snippet, "charge", &shop());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 3, Reason::parameter("orders")),
            Verdict::allowed(
                "getCustomer",
                3,
                Reason::lambda_parameter("order", Reason::parameter("orders"))
            ),
            Verdict::violation("rename", 3, Reason::return_of("getCustomer")),
            Verdict::allowed("forEach", 3, Reason::parameter("orders")),
        ]
    );
}

fn mapping_orders_with(reference: &str) -> String {
    format!(
        "void charge(List<Order> orders) {{\n    orders.stream().map({reference}).forEach(customer -> {{ customer.getWallet().withdraw(); }});\n}}"
    )
}

fn charging_the_customers_a_stranger_returned(call: &str) -> [Verdict; 5] {
    [
        Verdict::allowed("stream", 3, Reason::parameter("orders")),
        Verdict::allowed("map", 3, Reason::parameter("orders")),
        Verdict::violation(
            "getWallet",
            3,
            Reason::lambda_parameter("customer", Reason::return_of(call)),
        ),
        Verdict::violation("withdraw", 3, Reason::return_of("getWallet")),
        Verdict::allowed(
            "forEach",
            3,
            Reason::classified(Classification::Container, generic(STREAM, &[customer()])),
        ),
    ]
}

fn charging_the_customers_a_friend_made(made: Reason) -> [Verdict; 5] {
    [
        Verdict::allowed("stream", 3, Reason::parameter("orders")),
        Verdict::allowed("map", 3, Reason::parameter("orders")),
        Verdict::allowed(
            "getWallet",
            3,
            Reason::lambda_parameter("customer", made.clone()),
        ),
        Verdict::violation("withdraw", 3, Reason::return_of("getWallet")),
        Verdict::allowed("forEach", 3, made),
    ]
}

#[test]
fn a_constructor_reference_yields_the_type_it_creates() {
    let snippet = mapping_orders_with("Customer::new");

    let verdicts = verdicts_in_c(LIST_IMPORT, "", &snippet, "charge", &shop());

    assert_eq!(
        verdicts,
        charging_the_customers_a_friend_made(Reason::created(customer()))
    );
}

#[test]
fn a_method_reference_to_a_static_method_yields_what_the_method_returns() {
    let snippet = mapping_orders_with("Customers::of");

    let verdicts = verdicts_in_c(LIST_IMPORT, "", &snippet, "charge", &shop());

    let mut expected = charging_the_customers_a_friend_made(Reason::static_result("of")).to_vec();
    expected.insert(
        1,
        Verdict::allowed("of", 3, Reason::static_type(TypeRef::class("Customers"))),
    );
    assert_eq!(verdicts, expected);
}

#[test]
fn a_method_reference_on_an_expression_yields_what_the_method_of_its_type_returns() {
    let snippet = mapping_orders_with("registry::lookup");

    let verdicts = verdicts_in_c(
        LIST_IMPORT,
        "Registry registry;",
        &snippet,
        "charge",
        &shop(),
    );

    let mut expected = charging_the_customers_a_stranger_returned("lookup").to_vec();
    expected.insert(1, Verdict::allowed("lookup", 3, Reason::field("registry")));
    assert_eq!(verdicts, expected);
}

fn with_quotes(overloads: &[(TypeRef, &str)]) -> Types {
    let pricing = overloads
        .iter()
        .fold(class("Pricing"), |shape, (parameter, returns)| {
            shape.with_method(
                MethodShape::new("quote")
                    .with_parameters(vec![parameter.clone()])
                    .returning(TypeRef::class(returns)),
            )
        });
    shop().with_shape(pricing)
}

fn quoting_with(argument: &str, then: &str, types: &Types) -> Vec<Verdict> {
    let snippet = format!(
        "void charge(Pricing pricing, {argument} argument) {{\n    pricing.quote(argument).{then}();\n}}"
    );
    verdicts_in_c("", "", &snippet, "charge", types)
}

fn int() -> TypeRef {
    TypeRef::primitive("int")
}

fn wallet() -> TypeRef {
    TypeRef::class("Wallet")
}

#[test]
fn the_overload_whose_primitive_parameter_matches_the_argument_decides_the_quote() {
    let types = with_quotes(&[(int(), "Wallet"), (TypeRef::primitive("long"), "Customer")]);

    let verdicts = quoting_with("int", "withdraw", &types);

    assert_eq!(
        verdicts[1],
        Verdict::violation("withdraw", 3, Reason::return_of("quote"))
    );
}

#[test]
fn the_overload_whose_class_parameter_is_a_supertype_of_the_argument_decides_the_quote() {
    let types = with_quotes(&[(order(), "Customer"), (wallet(), "Wallet")]);

    let verdicts = quoting_with("Wallet", "withdraw", &types);

    assert_eq!(
        verdicts[1],
        Verdict::violation("withdraw", 3, Reason::return_of("quote"))
    );
}

#[test]
fn an_array_argument_fits_only_object_and_array_parameters() {
    let object = TypeRef::class("java.lang.Object");
    let types = with_quotes(&[(object, "Wallet"), (wallet(), "Customer")]);

    let verdicts = quoting_with("Order[]", "withdraw", &types);

    assert_eq!(
        verdicts[1],
        Verdict::violation("withdraw", 3, Reason::return_of("quote"))
    );
}

#[test]
fn an_argument_of_a_type_the_catalog_lacks_fits_any_class_parameter() {
    let types = with_quotes(&[(order(), "Customer"), (int(), "Wallet")]);

    let verdicts = quoting_with("Unmapped", "rename", &types);

    assert_eq!(
        verdicts[1],
        Verdict::violation("rename", 3, Reason::return_of("quote"))
    );
}

#[test]
fn overloads_that_all_miss_the_argument_still_agree_on_a_common_return() {
    let types = with_quotes(&[(order(), "Customer"), (wallet(), "Customer")]);

    let verdicts = quoting_with("Customer", "rename", &types);

    assert_eq!(
        verdicts[1],
        Verdict::violation("rename", 3, Reason::return_of("quote"))
    );
}

#[test]
fn a_class_argument_fits_an_object_parameter() {
    let object = TypeRef::class("java.lang.Object");
    let types = with_quotes(&[(object, "Customer"), (int(), "Wallet")]);

    let verdicts = quoting_with("Wallet", "rename", &types);

    assert_eq!(
        verdicts[1],
        Verdict::violation("rename", 3, Reason::return_of("quote"))
    );
}

#[test]
fn a_call_returning_a_subtype_of_the_expected_type_binds_through_the_supertype() {
    let lists = class("Lists").with_method(
        MethodShape::new("listOf")
            .as_static()
            .with_type_parameter("T")
            .with_parameters(vec![variable("T")])
            .returning(generic(ARRAY_LIST, &[variable("T")])),
    );
    let snippet = "void register() {\n    Collection<Consumer<Event>> handlers = Lists.listOf(event -> { event.getReceipt().close(); });\n}";

    let verdicts = verdicts_in_c(
        "import java.util.Collection; import java.util.function.Consumer;",
        "",
        snippet,
        "register",
        &shop().with_shape(lists),
    );

    assert_eq!(verdicts[1], closing_the_receipt());
}

fn registering_into(handler: &str, extra: TypeShape) -> Verdict {
    let snippet = "void register() {\n    handlers.put(\"created\", event -> { event.getReceipt().close(); });\n}";
    let verdicts = verdicts_in_c(
        &format!("import java.util.Map; import shop.{handler};"),
        &format!("Map<String, {handler}<Event>> handlers;"),
        snippet,
        "register",
        &shop().with_shape(extra),
    );
    verdicts[1].clone()
}

fn undecided_close() -> Verdict {
    Verdict::violation(
        "close",
        3,
        Reason::unresolved_type(None, Reason::return_of("getReceipt")),
    )
}

#[test]
fn an_abstract_method_inherited_from_a_super_interface_is_the_functional_method() {
    let handler = interface("shop.Handler", &["T"]).extending(generic(CONSUMER, &[variable("T")]));

    let verdict = registering_into("Handler", handler);

    assert_eq!(verdict, closing_the_receipt());
}

#[test]
fn an_interface_with_two_abstract_methods_is_not_functional() {
    let listener = interface("shop.Listener", &["T"])
        .with_method(
            MethodShape::new("accept")
                .with_parameters(vec![variable("T")])
                .as_abstract(),
        )
        .with_method(
            MethodShape::new("reject")
                .with_parameters(vec![variable("T")])
                .as_abstract(),
        );

    let verdict = registering_into("Listener", listener);

    assert_eq!(verdict, undecided_close());
}

#[test]
fn abstract_methods_of_object_do_not_count_against_a_functional_interface() {
    let matcher = interface("shop.Matcher", &["T"])
        .with_method(
            MethodShape::new("accept")
                .with_parameters(vec![variable("T")])
                .as_abstract(),
        )
        .with_method(
            MethodShape::new("equals")
                .with_parameters(vec![TypeRef::class("java.lang.Object")])
                .as_abstract(),
        );

    let verdict = registering_into("Matcher", matcher);

    assert_eq!(verdict, closing_the_receipt());
}

#[test]
fn a_default_method_overriding_an_inherited_abstract_one_leaves_the_other_abstract_method() {
    let relaxed = interface("shop.Relaxed", &["T"])
        .extending(generic(CONSUMER, &[variable("T")]))
        .with_method(MethodShape::new("accept").with_parameters(vec![variable("T")]))
        .with_method(
            MethodShape::new("handle")
                .with_parameters(vec![variable("T")])
                .as_abstract(),
        );

    let verdict = registering_into("Relaxed", relaxed);

    assert_eq!(verdict, closing_the_receipt());
}

#[test]
fn a_class_with_one_abstract_method_is_not_functional() {
    let task = class("shop.Task").with_type_parameter("T").with_method(
        MethodShape::new("run")
            .with_parameters(vec![variable("T")])
            .as_abstract(),
    );

    let verdict = registering_into("Task", task);

    assert_eq!(verdict, undecided_close());
}

#[test]
fn a_stream_mapped_to_created_objects_keeps_them_friends() {
    let snippet = "void print(List<Order> orders) {\n    orders.stream().map(order -> new Receipt(order)).forEach(receipt -> { receipt.print(); });\n}";

    let verdicts = verdicts_in_c(LIST_IMPORT, "", snippet, "print", &shop());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 3, Reason::parameter("orders")),
            Verdict::allowed("map", 3, Reason::parameter("orders")),
            Verdict::allowed(
                "print",
                3,
                Reason::lambda_parameter("receipt", Reason::created(TypeRef::class("Receipt")))
            ),
            Verdict::allowed("forEach", 3, Reason::created(TypeRef::class("Receipt"))),
        ]
    );
}

#[test]
fn a_filtered_stream_keeps_the_standing_of_its_receiver() {
    let snippet = "void ship(List<Order> orders) {\n    orders.stream().filter(order -> order.isPaid()).forEach(order -> { order.ship(); });\n}";

    let verdicts = verdicts_in_c(LIST_IMPORT, "", snippet, "ship", &shop());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 3, Reason::parameter("orders")),
            Verdict::allowed(
                "isPaid",
                3,
                Reason::lambda_parameter("order", Reason::parameter("orders"))
            ),
            Verdict::allowed("filter", 3, Reason::parameter("orders")),
            Verdict::allowed(
                "ship",
                3,
                Reason::lambda_parameter("order", Reason::parameter("orders"))
            ),
            Verdict::allowed("forEach", 3, Reason::parameter("orders")),
        ]
    );
}

#[test]
fn a_value_mapped_from_a_stream_is_allowed_by_its_classification() {
    let snippet = "void check(List<Order> orders) {\n    orders.stream().map(order -> order.getTotal()).forEach(total -> { total.signum(); });\n}";

    let verdicts = verdicts_in_c(LIST_IMPORT, "", snippet, "check", &shop());

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
fn a_call_returning_a_non_container_keeps_the_standing_of_its_container_receiver() {
    let snippet = "void rename(List<Order> orders) {\n    orders.stream().fold(order -> order.getCustomer()).rename();\n}";

    let verdicts = verdicts_in_c(LIST_IMPORT, "", snippet, "rename", &shop());

    assert_eq!(
        verdicts[3],
        Verdict::allowed("rename", 3, Reason::parameter("orders"))
    );
}

#[test]
fn a_container_a_call_on_a_non_container_returns_follows_the_usual_standing_rules() {
    let snippet = "void print() {\n    mapper.mapAll(order -> new Receipt(order)).forEach(receipt -> { receipt.print(); });\n}";

    let verdicts = verdicts_in_c("", "Mapper mapper;", snippet, "print", &shop());

    assert_eq!(
        verdicts[1],
        Verdict::violation(
            "print",
            3,
            Reason::lambda_parameter("receipt", Reason::return_of("mapAll"))
        )
    );
}

fn orders_then_print(reference: &str) -> Vec<Verdict> {
    let snippet = format!(
        "void print(List<Order> orders) {{\n    orders.stream().map({reference}).forEach(receipt -> {{ receipt.print(); }});\n}}"
    );
    verdicts_in_c(LIST_IMPORT, "", &snippet, "print", &shop())
}

#[test]
fn a_method_reference_to_a_factory_method_of_a_friend_yields_friends() {
    let verdicts = orders_then_print("Order::newReceipt");

    assert_eq!(
        verdicts[3],
        Verdict::allowed(
            "print",
            3,
            Reason::lambda_parameter("receipt", Reason::factory_result("newReceipt"))
        )
    );
}

#[test]
fn a_method_reference_to_an_unknown_method_yields_strangers() {
    let verdicts = orders_then_print("Order::missing");

    assert_eq!(
        verdicts[3],
        Verdict::violation(
            "print",
            3,
            Reason::unresolved_type(
                Some(variable("R")),
                Reason::lambda_parameter("receipt", Reason::return_of("missing"))
            )
        )
    );
}
