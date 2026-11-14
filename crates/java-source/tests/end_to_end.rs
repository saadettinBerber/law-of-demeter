use std::path::Path;

use law_of_demeter_core::domain::classification::{Classification, Types};
use law_of_demeter_core::domain::method::{Reason, Verdict};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};
use law_of_demeter_java_source::map_source;

fn record(name: &str) -> TypeShape {
    TypeShape::new(name, TypeKind::Record)
}

fn class(name: &str) -> TypeShape {
    TypeShape::new(name, TypeKind::Class)
}

fn getter(name: &str, field: &str, returns: &str) -> MethodShape {
    MethodShape::new(name)
        .reading(field)
        .returning(TypeRef::class(returns))
}

fn verdicts_of(source: &str, class_name: &str, method: &str, types: &Types) -> Vec<Verdict> {
    let mapped = map_source(Path::new("OrderService.java"), source, &|name| {
        name.starts_with("java.") || types.shape(name).is_some()
    })
    .unwrap();
    let found: Vec<_> = mapped
        .methods
        .iter()
        .filter(|candidate| candidate.class_name() == class_name && candidate.name() == method)
        .collect();
    assert!(!found.is_empty(), "{class_name}.{method} was not mapped");
    found
        .into_iter()
        .flat_map(|candidate| candidate.verdicts(types))
        .collect()
}

fn in_c(imports: &str, members: &str, snippet: &str) -> String {
    format!("{imports} class C {{ {members}\n{snippet}\n}}")
}

fn verdicts_in_c(
    imports: &str,
    members: &str,
    snippet: &str,
    method: &str,
    types: &Types,
) -> Vec<Verdict> {
    verdicts_of(&in_c(imports, members, snippet), "C", method, types)
}

fn shop_with_objects() -> Types {
    Types::new()
        .with_shape(
            class("Order")
                .with_method(getter("getCustomer", "customer", "Customer"))
                .with_method(getter("getLines", "lines", "java.util.List")),
        )
        .with_shape(
            class("Customer")
                .with_method(getter("getWallet", "wallet", "Wallet"))
                .with_method(
                    MethodShape::new("setAddress")
                        .writing("address")
                        .with_parameters(vec![TypeRef::class("Address")]),
                )
                .with_method(MethodShape::new("rename")),
        )
        .with_shape(class("Wallet").with_method(MethodShape::new("withdraw")))
}

fn order_with_lines(lines: &TypeRef) -> TypeShape {
    class("Order").with_method(
        MethodShape::new("getLines")
            .reading("lines")
            .returning(lines.clone()),
    )
}

fn order_lines() -> TypeRef {
    TypeRef::generic("java.util.List", vec![TypeRef::class("OrderLine")])
}

fn repository_catalog() -> Types {
    Types::new()
        .with_shape(
            class("OrderRepository").with_method(
                MethodShape::new("findById")
                    .with_parameters(vec![TypeRef::class("OrderId")])
                    .returning(TypeRef::class("Order")),
            ),
        )
        .with_shape(class("Order").with_method(MethodShape::new("ship")))
}

fn data_structure_catalog() -> Types {
    Types::new()
        .with_shape(record("Config").with_method(getter("server", "server", "Server")))
        .with_shape(
            record("Server").with_method(
                MethodShape::new("port")
                    .reading("port")
                    .returning(TypeRef::primitive("int")),
            ),
        )
        .with_shape(class("ShippingRequest").with_method(getter(
            "getAddress",
            "address",
            "Address",
        )))
        .with_shape(class("Address").with_method(getter("getCity", "city", "java.lang.String")))
        .with_shape(class("Customer").with_method(getter("getId", "id", "CustomerId")))
        .with_shape(class("CustomerId").with_method(getter(
            "getValue",
            "value",
            "java.lang.String",
        )))
}

fn classified_data_structure(name: &str) -> Reason {
    Reason::classified(Classification::DataStructure, TypeRef::class(name))
}

#[test]
fn violation_1_reaching_through_a_getters_object() {
    let snippet = "void pay(Order order, int amount) {\n    order.getCustomer().getWallet().withdraw(amount);\n}";

    let verdicts = verdicts_in_c("", "", snippet, "pay", &shop_with_objects());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 3, Reason::parameter("order")),
            Verdict::violation("getWallet", 3, Reason::return_of("getCustomer")),
            Verdict::violation("withdraw", 3, Reason::return_of("getWallet")),
        ]
    );
}

#[test]
fn violation_2_splitting_the_chain_into_a_local() {
    let snippet = "void pay(Order order, int amount) {\n    Customer customer = order.getCustomer();\n    customer.getWallet().withdraw(amount);\n}";

    let verdicts = verdicts_in_c("", "", snippet, "pay", &shop_with_objects());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 3, Reason::parameter("order")),
            Verdict::violation(
                "getWallet",
                4,
                Reason::local("customer", Reason::return_of("getCustomer"))
            ),
            Verdict::violation("withdraw", 4, Reason::return_of("getWallet")),
        ]
    );
}

#[test]
fn violation_3_changing_a_getters_object_with_a_setter() {
    let snippet = "void move(Order order, Address address) {\n    order.getCustomer().setAddress(address);\n}";

    let verdicts = verdicts_in_c("", "", snippet, "move", &shop_with_objects());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 3, Reason::parameter("order")),
            Verdict::violation("setAddress", 3, Reason::return_of("getCustomer")),
        ]
    );
}

#[test]
fn violation_4_changing_a_collection_a_getter_returned() {
    let lines = order_lines();
    let types = Types::new().with_shape(order_with_lines(&lines));
    let snippet = "void addLine(Order order, OrderLine line) {\n    order.getLines().add(line);\n}";

    let verdicts = verdicts_in_c("", "", snippet, "addLine", &types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getLines", 3, Reason::parameter("order")),
            Verdict::violation(
                "add",
                3,
                Reason::container_mutation(lines, Reason::return_of("getLines"))
            ),
        ]
    );
}

#[test]
fn violation_5_a_hybrid_class_maps_with_its_calls_judged_and_its_name_listed() {
    let source = "@Getter\n@Setter\nclass Account {\n    private int balance;\n\n    void withdraw(int amount) {\n        if (amount > balance) {\n            throw new IllegalStateException(\"Insufficient balance\");\n        }\n        balance -= amount;\n    }\n}";
    let mapped = map_source(Path::new("Account.java"), source, &|_| false).unwrap();

    let verdicts = verdicts_of(source, "Account", "withdraw", &Types::new());

    assert_eq!(mapped.classes, ["Account"]);
    assert_eq!(verdicts, []);
}

#[test]
fn violation_6_changing_the_elements_of_an_inner_collection() {
    let lines = order_lines();
    let types = Types::new()
        .with_shape(order_with_lines(&lines))
        .with_shape(class("OrderLine").with_method(MethodShape::new("applyDiscount")));
    let snippet = "void discount(Order order, int rate) {\n    order.getLines().forEach(line -> line.applyDiscount(rate));\n}";

    let verdicts = verdicts_in_c("", "", snippet, "discount", &types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getLines", 3, Reason::parameter("order")),
            Verdict::violation(
                "applyDiscount",
                3,
                Reason::lambda_parameter("line", Reason::return_of("getLines"))
            ),
            Verdict::allowed(
                "forEach",
                3,
                Reason::classified(Classification::Container, lines)
            ),
        ]
    );
}

#[test]
fn violation_7_reaching_through_the_object_a_data_structure_carries() {
    let types = Types::new()
        .with_shape(class("OrderShippedEvent").with_method(getter("getOrder", "order", "Order")))
        .with_shape(
            class("Order")
                .with_method(getter("getTotal", "total", "int"))
                .with_method(MethodShape::new("ship")),
        );
    let snippet =
        "void onShipped(OrderShippedEvent event) {\n    log(event.getOrder().getTotal());\n}";

    let verdicts = verdicts_in_c("", "", snippet, "onShipped", &types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getOrder", 3, Reason::parameter("event")),
            Verdict::violation("getTotal", 3, Reason::return_of("getOrder")),
            Verdict::allowed("log", 3, Reason::This),
        ]
    );
}

#[test]
fn violation_8_talking_to_the_object_a_holder_getter_returned() {
    let types = Types::new()
        .with_shape(class("Session").with_method(getter("getCart", "cart", "Cart")))
        .with_shape(class("Cart").with_method(MethodShape::new("clear")));
    let snippet = "void checkout(Session session) {\n    session.getCart().clear();\n}";

    let verdicts = verdicts_in_c("", "", snippet, "checkout", &types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCart", 3, Reason::parameter("session")),
            Verdict::violation("clear", 3, Reason::return_of("getCart")),
        ]
    );
}

#[test]
fn allowed_1_reading_a_value_from_a_parameter() {
    let snippet = "boolean canShip(Order order) {\n    return order.getStatus() == Status.PAID;\n}";

    let verdicts = verdicts_in_c("", "", snippet, "canShip", &Types::new());

    assert_eq!(
        verdicts,
        [Verdict::allowed("getStatus", 3, Reason::parameter("order"))]
    );
}

#[test]
fn allowed_2_operating_on_a_value() {
    let instant = TypeRef::class("java.time.Instant");
    let types = Types::new().with_shape(
        class("java.time.Instant").with_method(
            MethodShape::new("plus")
                .with_parameters(vec![TypeRef::class("java.time.Duration")])
                .returning(instant),
        ),
    );
    let imports = "import java.time.Duration; import java.time.Instant; import java.time.temporal.ChronoUnit;";
    let snippet = "Instant expiry(Instant issuedAt, Duration ttl) {\n    return issuedAt.plus(ttl).truncatedTo(ChronoUnit.SECONDS);\n}";

    let verdicts = verdicts_in_c(imports, "", snippet, "expiry", &types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("plus", 3, Reason::parameter("issuedAt")),
            Verdict::allowed("truncatedTo", 3, Reason::parameter("issuedAt")),
        ]
    );
}

#[test]
fn allowed_3_reading_a_chain_through_data_structures() {
    let snippet = "int portOf(Config config) {\n    return config.server().port();\n}\n\nString cityOf(ShippingRequest request) {\n    return request.getAddress().getCity();\n}\n\nString keyOf(Customer customer) {\n    return customer.getId().getValue();\n}";
    let types = data_structure_catalog();

    let verdicts =
        ["portOf", "cityOf", "keyOf"].map(|method| verdicts_in_c("", "", snippet, method, &types));

    assert_eq!(
        verdicts,
        [
            vec![
                Verdict::allowed("server", 3, Reason::parameter("config")),
                Verdict::allowed("port", 3, classified_data_structure("Server")),
            ],
            vec![
                Verdict::allowed("getAddress", 7, Reason::parameter("request")),
                Verdict::allowed("getCity", 7, classified_data_structure("Address")),
            ],
            vec![
                Verdict::allowed("getId", 11, Reason::parameter("customer")),
                Verdict::allowed("getValue", 11, classified_data_structure("CustomerId")),
            ],
        ]
    );
}

fn builder_catalog() -> Types {
    let builder = TypeRef::class("Order$Builder");
    Types::new()
        .with_shape(
            class("Order").with_method(
                MethodShape::new("builder")
                    .as_static()
                    .returning(builder.clone()),
            ),
        )
        .with_shape(
            class("Order$Builder")
                .with_method(
                    MethodShape::new("id")
                        .with_parameters(vec![TypeRef::class("java.lang.String")])
                        .returning(builder.clone()),
                )
                .with_method(
                    MethodShape::new("total")
                        .with_parameters(vec![TypeRef::primitive("int")])
                        .returning(builder.clone()),
                )
                .with_method(MethodShape::new("build").returning(TypeRef::class("Order"))),
        )
}

#[test]
fn allowed_4_builder_and_fluent_chains() {
    let snippet = "Order create(String id, int total) {\n    return Order.builder().id(id).total(total).build();\n}";

    let verdicts = verdicts_in_c("", "", snippet, "create", &builder_catalog());

    let order = TypeRef::class("Order");
    assert_eq!(
        verdicts,
        [
            Verdict::allowed("builder", 3, Reason::static_type(order)),
            Verdict::allowed("id", 3, Reason::static_result("builder")),
            Verdict::allowed("total", 3, Reason::static_result("builder")),
            Verdict::allowed("build", 3, Reason::static_result("builder")),
        ]
    );
}

#[test]
fn allowed_4_a_constructor_reading_the_fields_of_its_builder() {
    let source = "class Order {\n    String id;\n    int total;\n\n    private Order(Builder builder) {\n        this.id = builder.id;\n        this.total = builder.total;\n    }\n\n    static class Builder {\n        String id;\n        int total;\n    }\n}";

    let verdicts = verdicts_of(source, "Order", "<init>", &builder_catalog());

    assert_eq!(verdicts, []);
}

#[test]
fn allowed_5_java_collections_and_streams() {
    let order = TypeRef::class("Order");
    let orders_stream = TypeRef::generic("java.util.stream.Stream", vec![order.clone()]);
    let types = Types::new()
        .with_shape(
            TypeShape::new("java.util.List", TypeKind::Interface)
                .with_method(MethodShape::new("stream").returning(orders_stream.clone())),
        )
        .with_shape(
            TypeShape::new("java.util.stream.Stream", TypeKind::Interface)
                .with_method(
                    MethodShape::new("filter")
                        .with_parameters(vec![TypeRef::class("java.util.function.Predicate")])
                        .returning(orders_stream.clone()),
                )
                .with_method(MethodShape::new("count").returning(TypeRef::primitive("long"))),
        );
    let snippet = "long paidCount(List<Order> orders) {\n    return orders.stream().filter(Order::isPaid).count();\n}";

    let verdicts = verdicts_in_c("import java.util.List;", "", snippet, "paidCount", &types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 3, Reason::parameter("orders")),
            Verdict::allowed("filter", 3, Reason::parameter("orders")),
            Verdict::allowed("count", 3, Reason::parameter("orders")),
        ]
    );
}

#[test]
fn allowed_6_factories_and_new_objects() {
    let builder = TypeRef::class("java.lang.StringBuilder");
    let types =
        Types::new()
            .with_shape(
                class("ReportFactory")
                    .with_method(MethodShape::new("create").returning(TypeRef::class("Report"))),
            )
            .with_shape(class("Report").with_method(
                MethodShape::new("render").returning(TypeRef::class("java.lang.String")),
            ))
            .with_shape(
                class("java.lang.StringBuilder")
                    .with_method(
                        MethodShape::new("append")
                            .with_parameters(vec![TypeRef::class("java.lang.String")])
                            .returning(builder.clone()),
                    )
                    .with_method(
                        MethodShape::new("toString").returning(TypeRef::class("java.lang.String")),
                    ),
            );
    let snippet = "String report(ReportFactory factory) {\n    return factory.create().render();\n}\n\nString greeting(String name) {\n    return new StringBuilder().append(\"Hello \").append(name).toString();\n}";

    let verdicts =
        ["report", "greeting"].map(|method| verdicts_in_c("", "", snippet, method, &types));

    assert_eq!(
        verdicts,
        [
            vec![
                Verdict::allowed("create", 3, Reason::parameter("factory")),
                Verdict::allowed("render", 3, Reason::factory_result("create")),
            ],
            vec![
                Verdict::allowed("append", 7, Reason::created(builder.clone())),
                Verdict::allowed("append", 7, Reason::created(builder.clone())),
                Verdict::allowed("toString", 7, Reason::created(builder)),
            ],
        ]
    );
}

#[test]
fn allowed_7_passing_a_stranger_on_to_another_method() {
    let types = Types::new().with_shape(class("Order").with_method(getter(
        "getCustomer",
        "customer",
        "Customer",
    )));
    let snippet = "void notifyCustomer(Order order) {\n    notifier.send(order.getCustomer());\n}";

    let verdicts = verdicts_in_c("", "Notifier notifier;", snippet, "notifyCustomer", &types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 3, Reason::parameter("order")),
            Verdict::allowed("send", 3, Reason::field("notifier")),
        ]
    );
}

#[test]
fn allowed_8_own_fields_and_own_methods() {
    let snippet = "void pay(int amount) {\n    wallet.withdraw(amount);\n    this.audit();\n}";

    let verdicts = verdicts_in_c("", "Wallet wallet;", snippet, "pay", &Types::new());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("withdraw", 3, Reason::field("wallet")),
            Verdict::allowed("audit", 4, Reason::This),
        ]
    );
}

#[test]
fn allowed_9_talking_to_the_object_a_repository_query_returned() {
    let snippet = "void dispatch(OrderId id) {\n    orders.findById(id).ship();\n}";

    let verdicts = verdicts_in_c(
        "",
        "OrderRepository orders;",
        snippet,
        "dispatch",
        &repository_catalog(),
    );

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("findById", 3, Reason::field("orders")),
            Verdict::allowed("ship", 3, Reason::query_result("findById")),
        ]
    );
}

#[test]
fn out_of_scope_1_computing_outside_an_object_on_friend_values() {
    let snippet =
        "int totalOf(Order order) {\n    return order.getPrice() * order.getQuantity();\n}";

    let verdicts = verdicts_in_c("", "", snippet, "totalOf", &Types::new());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getPrice", 3, Reason::parameter("order")),
            Verdict::allowed("getQuantity", 3, Reason::parameter("order")),
        ]
    );
}

#[test]
fn out_of_scope_2_deciding_outside_an_object_on_a_friends_state() {
    let snippet = "void ship(Order order) {\n    if (order.getStatus() == Status.PAID) {\n        order.markShipped();\n    }\n}";

    let verdicts = verdicts_in_c("", "", snippet, "ship", &Types::new());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getStatus", 3, Reason::parameter("order")),
            Verdict::allowed("markShipped", 4, Reason::parameter("order")),
        ]
    );
}

#[test]
fn readme_violation_a_chain_through_getters_is_reported_on_line_three() {
    let source = "class OrderService {\n    void pay(Order order, Money amount) {\n        order.getCustomer().getWallet().withdraw(amount);\n    }\n}";

    let verdicts = verdicts_of(source, "OrderService", "pay", &shop_with_objects());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 3, Reason::parameter("order")),
            Verdict::violation("getWallet", 3, Reason::return_of("getCustomer")),
            Verdict::violation("withdraw", 3, Reason::return_of("getWallet")),
        ]
    );
}

#[test]
fn readme_allowed_telling_the_argument_what_to_do() {
    let source = "class OrderService {\n    void pay(Order order, Money amount) {\n        order.charge(amount);\n    }\n}";

    let verdicts = verdicts_of(source, "OrderService", "pay", &Types::new());

    assert_eq!(
        verdicts,
        [Verdict::allowed("charge", 3, Reason::parameter("order"))]
    );
}

#[test]
fn readme_allowed_a_chain_through_records_in_the_same_file() {
    let source = "record Server(String host, int port) {\n}\n\nrecord Config(Server server) {\n}\n\nclass Connector {\n    int portOf(Config config) {\n        return config.server().port();\n    }\n}";

    let verdicts = verdicts_of(source, "Connector", "portOf", &data_structure_catalog());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("server", 9, Reason::parameter("config")),
            Verdict::allowed("port", 9, classified_data_structure("Server")),
        ]
    );
}

#[test]
fn readme_hybrid_the_calls_around_a_hybrid_class_are_judged_on_their_own() {
    let source = "class Account {\n    private int balance;\n\n    int getBalance() {\n        return balance;\n    }\n\n    void setBalance(int balance) {\n        this.balance = balance;\n    }\n\n    void withdraw(int amount) {\n        if (amount > balance) {\n            throw new IllegalStateException(\"Insufficient balance\");\n        }\n        balance -= amount;\n    }\n}\n\nclass PaymentService {\n    void pay(Account account, int amount) {\n        account.setBalance(account.getBalance() - amount);\n    }\n}";

    let verdicts = verdicts_of(source, "PaymentService", "pay", &Types::new());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getBalance", 22, Reason::parameter("account")),
            Verdict::allowed("setBalance", 22, Reason::parameter("account")),
        ]
    );
}
