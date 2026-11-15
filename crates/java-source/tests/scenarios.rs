mod support;

use law_of_demeter_java_source::{
    DeclaredType, Expression, Method, Parameter, Statement, TypeName, parse_with,
};
use support::{KNOWN_TYPES, call, created, field, local, method_in, own_call, parameter, type_ref};

const IMPORTS: &str = "import java.time.Duration; import java.time.Instant; \
    import java.time.temporal.ChronoUnit; import java.util.List;";

fn scenario(fields: &str, snippet: &str, name: &str) -> Method {
    method_in(IMPORTS, fields, snippet, name)
}

fn expected(
    name: &str,
    line: usize,
    parameters: Vec<(&str, DeclaredType)>,
    returns: DeclaredType,
    body: Vec<Statement>,
) -> Method {
    Method {
        name: name.to_owned(),
        line,
        is_static: false,
        type_parameters: Vec::new(),
        parameters: parameters
            .into_iter()
            .map(|(name, declared_type)| Parameter {
                name: name.to_owned(),
                declared_type,
            })
            .collect(),
        returns: Some(returns),
        body: Some(body),
    }
}

fn void() -> DeclaredType {
    DeclaredType::primitive("void")
}

fn int() -> DeclaredType {
    DeclaredType::primitive("int")
}

fn shop(name: &str) -> DeclaredType {
    DeclaredType::class(&format!("shop.{name}"))
}

fn run(expression: Expression) -> Statement {
    Statement::Expression(expression)
}

fn read_field(target: Expression, name: &str) -> Expression {
    Expression::FieldAccess {
        target: Box::new(target),
        name: name.to_owned(),
    }
}

#[test]
fn violation_1_a_getter_chain() {
    let snippet = "void pay(Order order, int amount) {\n    order.getCustomer().getWallet().withdraw(amount);\n}";

    let method = scenario("", snippet, "pay");

    assert_eq!(
        method,
        expected(
            "pay",
            2,
            vec![("order", shop("Order")), ("amount", int())],
            void(),
            vec![run(call(
                call(
                    call(parameter("order"), "getCustomer", vec![], 3),
                    "getWallet",
                    vec![],
                    3
                ),
                "withdraw",
                vec![parameter("amount")],
                3
            ))]
        )
    );
}

#[test]
fn violation_2_a_chain_split_into_a_local_variable() {
    let snippet = "void pay(Order order, int amount) {\n    Customer customer = order.getCustomer();\n    customer.getWallet().withdraw(amount);\n}";

    let method = scenario("", snippet, "pay");

    assert_eq!(
        method.body,
        Some(vec![
            Statement::Local(law_of_demeter_java_source::LocalVariable {
                name: "customer".to_owned(),
                declared_type: shop("Customer"),
                initializer: Some(call(parameter("order"), "getCustomer", vec![], 3)),
                line: 3,
            }),
            run(call(
                call(local("customer"), "getWallet", vec![], 4),
                "withdraw",
                vec![parameter("amount")],
                4
            )),
        ])
    );
}

#[test]
fn violation_3_a_setter_on_a_returned_object() {
    let snippet = "void move(Order order, Address address) {\n    order.getCustomer().setAddress(address);\n}";

    let method = scenario("", snippet, "move");

    assert_eq!(
        method,
        expected(
            "move",
            2,
            vec![("order", shop("Order")), ("address", shop("Address"))],
            void(),
            vec![run(call(
                call(parameter("order"), "getCustomer", vec![], 3),
                "setAddress",
                vec![parameter("address")],
                3
            ))]
        )
    );
}

#[test]
fn violation_4_changing_a_returned_collection() {
    let snippet = "void addLine(Order order, OrderLine line) {\n    order.getLines().add(line);\n}";

    let method = scenario("", snippet, "addLine");

    assert_eq!(
        method.body,
        Some(vec![run(call(
            call(parameter("order"), "getLines", vec![], 3),
            "add",
            vec![parameter("line")],
            3
        ))])
    );
}

#[test]
fn violation_5_a_hybrid_class_with_lombok_annotations() {
    let source = "package shop;\n\
        import lombok.Getter;\n\
        import lombok.Setter;\n\
        @Getter\n\
        @Setter\n\
        class Account {\n\
            private int balance;\n\
            \n\
            void withdraw(int amount) {\n\
                if (amount > balance) {\n\
                    throw new IllegalStateException(\"Insufficient balance\");\n\
                }\n\
                balance -= amount;\n\
            }\n\
        }";

    let file = parse_with("Account.java", source, &|name| KNOWN_TYPES.contains(&name)).unwrap();

    let account = &file.classes[0];
    assert_eq!(
        account.annotations,
        [
            TypeName::Resolved("lombok.Getter".to_owned()),
            TypeName::Resolved("lombok.Setter".to_owned())
        ]
    );
    assert_eq!(account.fields[0].name, "balance");
    assert_eq!(
        account.methods[0],
        expected(
            "withdraw",
            9,
            vec![("amount", int())],
            void(),
            vec![
                Statement::If {
                    condition: Expression::Operation(vec![parameter("amount"), field("balance")]),
                    then_branch: vec![run(created(
                        "java.lang.IllegalStateException",
                        vec![Expression::Literal],
                        11
                    ))],
                    else_branch: vec![],
                },
                run(Expression::Operation(vec![
                    field("balance"),
                    parameter("amount")
                ])),
            ]
        )
    );
}

#[test]
fn violation_6_reaching_into_the_elements_of_an_inner_collection() {
    let snippet = "void discount(Order order, int rate) {\n    order.getLines().forEach(line -> line.applyDiscount(rate));\n}";

    let method = scenario("", snippet, "discount");

    let lambda = Expression::Lambda(law_of_demeter_java_source::Lambda {
        parameters: vec![law_of_demeter_java_source::LambdaParameter {
            name: "line".to_owned(),
            declared_type: None,
        }],
        body: vec![Statement::Return(Some(call(
            Expression::LambdaParameter("line".to_owned()),
            "applyDiscount",
            vec![parameter("rate")],
            3,
        )))],
    });
    assert_eq!(
        method.body,
        Some(vec![run(call(
            call(parameter("order"), "getLines", vec![], 3),
            "forEach",
            vec![lambda],
            3
        ))])
    );
}

#[test]
fn violation_7_reaching_into_what_a_data_structure_carries() {
    let snippet =
        "void onShipped(OrderShippedEvent event) {\n    log(event.getOrder().getTotal());\n}";

    let method = scenario("", snippet, "onShipped");

    assert_eq!(
        method.body,
        Some(vec![run(own_call(
            "log",
            vec![call(
                call(parameter("event"), "getOrder", vec![], 3),
                "getTotal",
                vec![],
                3
            )],
            3
        ))])
    );
}

#[test]
fn violation_8_talking_to_what_a_holder_getter_returns() {
    let snippet = "void checkout(Session session) {\n    session.getCart().clear();\n}";

    let method = scenario("", snippet, "checkout");

    assert_eq!(
        method.body,
        Some(vec![run(call(
            call(parameter("session"), "getCart", vec![], 3),
            "clear",
            vec![],
            3
        ))])
    );
}

#[test]
fn allowed_1_reading_a_value_from_a_parameter() {
    let snippet = "boolean canShip(Order order) {\n    return order.getStatus() == Status.PAID;\n}";

    let method = scenario("", snippet, "canShip");

    assert_eq!(
        method,
        expected(
            "canShip",
            2,
            vec![("order", shop("Order"))],
            DeclaredType::primitive("boolean"),
            vec![Statement::Return(Some(Expression::Operation(vec![
                call(parameter("order"), "getStatus", vec![], 3),
                read_field(type_ref("shop.Status"), "PAID"),
            ])))]
        )
    );
}

#[test]
fn allowed_2_operating_on_a_value() {
    let snippet = "Instant expiry(Instant issuedAt, Duration ttl) {\n    return issuedAt.plus(ttl).truncatedTo(ChronoUnit.SECONDS);\n}";

    let method = scenario("", snippet, "expiry");

    assert_eq!(
        method,
        expected(
            "expiry",
            2,
            vec![
                ("issuedAt", DeclaredType::class("java.time.Instant")),
                ("ttl", DeclaredType::class("java.time.Duration"))
            ],
            DeclaredType::class("java.time.Instant"),
            vec![Statement::Return(Some(call(
                call(parameter("issuedAt"), "plus", vec![parameter("ttl")], 3),
                "truncatedTo",
                vec![read_field(
                    type_ref("java.time.temporal.ChronoUnit"),
                    "SECONDS"
                )],
                3
            )))]
        )
    );
}

#[test]
fn allowed_3_reading_a_data_structure_with_a_chain() {
    let snippet = "int portOf(Config config) {\n    return config.server().port();\n}\n\
        String cityOf(ShippingRequest request) {\n    return request.getAddress().getCity();\n}\n\
        String keyOf(Customer customer) {\n    return customer.getId().getValue();\n}";

    let port = scenario("", snippet, "portOf");
    let city = scenario("", snippet, "cityOf");
    let key = scenario("", snippet, "keyOf");

    assert_eq!(
        port.body,
        Some(vec![Statement::Return(Some(call(
            call(parameter("config"), "server", vec![], 3),
            "port",
            vec![],
            3
        )))])
    );
    assert_eq!(city.line, 5);
    assert_eq!(
        city.body,
        Some(vec![Statement::Return(Some(call(
            call(parameter("request"), "getAddress", vec![], 6),
            "getCity",
            vec![],
            6
        )))])
    );
    assert_eq!(key.line, 8);
    assert_eq!(
        key.body,
        Some(vec![Statement::Return(Some(call(
            call(parameter("customer"), "getId", vec![], 9),
            "getValue",
            vec![],
            9
        )))])
    );
}

#[test]
fn allowed_3_records_are_classes_with_their_components_as_fields() {
    let source = "package shop;\n\
        record Server(String host, int port) {\n}\n\
        record Config(Server server) {\n}\n\
        class Connector {\n    int portOf(Config config) {\n        return config.server().port();\n    }\n}";

    let file = parse_with("Config.java", source, &|name| KNOWN_TYPES.contains(&name)).unwrap();

    let names: Vec<&str> = file
        .classes
        .iter()
        .map(|class| class.name.as_str())
        .collect();
    assert_eq!(names, ["shop.Server", "shop.Config", "shop.Connector"]);
    assert_eq!(file.classes[1].fields[0].name, "server");
    assert_eq!(file.classes[1].fields[0].declared_type, shop("Server"));
    assert_eq!(
        file.classes[2].methods[0].body,
        Some(vec![Statement::Return(Some(call(
            call(parameter("config"), "server", vec![], 8),
            "port",
            vec![],
            8
        )))])
    );
}

#[test]
fn allowed_4_a_builder_chain_and_a_constructor_reading_the_builder() {
    let chain = "Order create(String id, int total) {\n    return Order.builder().id(id).total(total).build();\n}";
    let constructor = "private final String id;\nprivate final int total;\n\
        private OrderService(Builder builder) {\n    this.id = builder.id;\n    this.total = builder.total;\n}\n\
        static class Builder {\n    String id;\n    int total;\n}";

    let create = scenario("", chain, "create");
    let source = format!("package shop; class OrderService {{\n{constructor}\n}}");
    let file = parse_with("OrderService.java", &source, &|name| {
        KNOWN_TYPES.contains(&name)
    })
    .unwrap();

    assert_eq!(
        create.body,
        Some(vec![Statement::Return(Some(call(
            call(
                call(
                    call(type_ref("shop.Order"), "builder", vec![], 3),
                    "id",
                    vec![parameter("id")],
                    3
                ),
                "total",
                vec![parameter("total")],
                3
            ),
            "build",
            vec![],
            3
        )))])
    );
    let constructor = &file.classes[0].methods[0];
    assert!(constructor.is_constructor());
    assert_eq!(
        constructor.parameters[0].declared_type,
        DeclaredType::class("shop.OrderService$Builder")
    );
    assert_eq!(
        constructor.body,
        Some(vec![
            run(Expression::Assignment {
                target: Box::new(field("id")),
                value: Box::new(read_field(parameter("builder"), "id")),
            }),
            run(Expression::Assignment {
                target: Box::new(field("total")),
                value: Box::new(read_field(parameter("builder"), "total")),
            }),
        ])
    );
}

#[test]
fn allowed_5_a_stream_with_a_method_reference() {
    let snippet = "long paidCount(List<Order> orders) {\n    return orders.stream().filter(Order::isPaid).count();\n}";

    let method = scenario("", snippet, "paidCount");

    assert_eq!(
        method,
        expected(
            "paidCount",
            2,
            vec![(
                "orders",
                DeclaredType::generic("java.util.List", vec![shop("Order")])
            )],
            DeclaredType::primitive("long"),
            vec![Statement::Return(Some(call(
                call(
                    call(parameter("orders"), "stream", vec![], 3),
                    "filter",
                    vec![Expression::MethodReference {
                        target: Box::new(type_ref("shop.Order")),
                        name: "isPaid".to_owned()
                    }],
                    3
                ),
                "count",
                vec![],
                3
            )))]
        )
    );
}

#[test]
fn allowed_6_a_factory_and_new_objects() {
    let snippet = "String report(ReportFactory factory) {\n    return factory.create().render();\n}\n\
        String greeting(String name) {\n    return new StringBuilder().append(\"Hello \").append(name).toString();\n}";

    let report = scenario("", snippet, "report");
    let greeting = scenario("", snippet, "greeting");

    assert_eq!(
        report.body,
        Some(vec![Statement::Return(Some(call(
            call(parameter("factory"), "create", vec![], 3),
            "render",
            vec![],
            3
        )))])
    );
    assert_eq!(
        greeting.body,
        Some(vec![Statement::Return(Some(call(
            call(
                call(
                    created("java.lang.StringBuilder", vec![], 6),
                    "append",
                    vec![Expression::Literal],
                    6
                ),
                "append",
                vec![parameter("name")],
                6
            ),
            "toString",
            vec![],
            6
        )))])
    );
}

#[test]
fn allowed_7_only_passing_a_stranger_on() {
    let snippet = "void notifyCustomer(Order order) {\n    notifier.send(order.getCustomer());\n}";

    let method = scenario("private Notifier notifier;", snippet, "notifyCustomer");

    assert_eq!(
        method.body,
        Some(vec![run(call(
            field("notifier"),
            "send",
            vec![call(parameter("order"), "getCustomer", vec![], 3)],
            3
        ))])
    );
}

#[test]
fn allowed_8_its_own_object_and_fields() {
    let snippet = "void pay(int amount) {\n    wallet.withdraw(amount);\n    this.audit();\n}";

    let method = scenario("private Wallet wallet;", snippet, "pay");

    assert_eq!(
        method.body,
        Some(vec![
            run(call(
                field("wallet"),
                "withdraw",
                vec![parameter("amount")],
                3
            )),
            run(call(Expression::This, "audit", vec![], 4)),
        ])
    );
}

#[test]
fn allowed_9_talking_to_what_a_repository_query_returns() {
    let snippet = "void dispatch(OrderId id) {\n    orders.findById(id).ship();\n}";

    let method = scenario("private Orders orders;", snippet, "dispatch");

    assert_eq!(
        method.body,
        Some(vec![run(call(
            call(field("orders"), "findById", vec![parameter("id")], 3),
            "ship",
            vec![],
            3
        ))])
    );
}

#[test]
fn out_of_scope_1_computing_outside_the_object() {
    let snippet =
        "int totalOf(Order order) {\n    return order.getPrice() * order.getQuantity();\n}";

    let method = scenario("", snippet, "totalOf");

    assert_eq!(
        method.body,
        Some(vec![Statement::Return(Some(Expression::Operation(vec![
            call(parameter("order"), "getPrice", vec![], 3),
            call(parameter("order"), "getQuantity", vec![], 3),
        ])))])
    );
}

#[test]
fn out_of_scope_2_reading_state_and_deciding_outside() {
    let snippet = "void ship(Order order) {\n    if (order.getStatus() == Status.PAID) {\n        order.markShipped();\n    }\n}";

    let method = scenario("", snippet, "ship");

    assert_eq!(
        method.body,
        Some(vec![Statement::If {
            condition: Expression::Operation(vec![
                call(parameter("order"), "getStatus", vec![], 3),
                read_field(type_ref("shop.Status"), "PAID"),
            ]),
            then_branch: vec![run(call(parameter("order"), "markShipped", vec![], 4))],
            else_branch: vec![],
        }])
    );
}

#[test]
fn readme_the_hybrid_account_and_its_payment_service() {
    let source = "package shop;\n\
        class Account {\n    private int balance;\n\n    int getBalance() {\n        return balance;\n    }\n\n    void setBalance(int balance) {\n        this.balance = balance;\n    }\n}\n\
        class PaymentService {\n    void pay(Account account, int amount) {\n        account.setBalance(account.getBalance() - amount);\n    }\n}";

    let file = parse_with("Account.java", source, &|_| false).unwrap();

    let pay = &file.classes[1].methods[0];
    assert_eq!(
        pay.body,
        Some(vec![run(call(
            parameter("account"),
            "setBalance",
            vec![Expression::Operation(vec![
                call(parameter("account"), "getBalance", vec![], 15),
                parameter("amount"),
            ])],
            15
        ))])
    );
    let set_balance = &file.classes[0].methods[1];
    assert_eq!(
        set_balance.body,
        Some(vec![run(Expression::Assignment {
            target: Box::new(field("balance")),
            value: Box::new(parameter("balance")),
        })])
    );
}
