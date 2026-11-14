use super::{Call, Creation, Expression, Lambda, Method, Reason, Verdict};
use crate::domain::classification::{Classification, Types};
use crate::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

#[test]
fn calling_a_method_on_a_returned_object_is_a_violation() {
    let pay = Method::new("OrderService", "pay", 1).call(
        Call::new(
            Expression::parameter("order", TypeRef::class("Order")),
            "getCustomer",
            1,
        )
        .then("getWallet", 1)
        .then("withdraw", 1),
    );

    let verdicts = pay.verdicts(&shop());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 1, Reason::parameter("order")),
            Verdict::violation("getWallet", 1, Reason::return_of("getCustomer")),
            Verdict::violation("withdraw", 1, Reason::return_of("getWallet")),
        ]
    );
}

#[test]
fn calling_methods_on_this_and_own_fields_is_allowed() {
    let pay = Method::new("OrderService", "pay", 1)
        .call(Call::new(
            Expression::field("wallet", TypeRef::class("Wallet")),
            "withdraw",
            1,
        ))
        .call(Call::new(Expression::This, "audit", 1));

    let verdicts = pay.verdicts(&Types::new());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("withdraw", 1, Reason::field("wallet")),
            Verdict::allowed("audit", 1, Reason::This),
        ]
    );
}

#[test]
fn passing_a_returned_object_as_an_argument_is_allowed() {
    let notify_customer = Method::new("OrderService", "pay", 1).call(
        Call::new(
            Expression::field("notifier", TypeRef::class("Notifier")),
            "send",
            1,
        )
        .with(Expression::return_of(Call::new(
            Expression::parameter("order", TypeRef::class("Order")),
            "getCustomer",
            1,
        ))),
    );

    let verdicts = notify_customer.verdicts(&Types::new());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 1, Reason::parameter("order")),
            Verdict::allowed("send", 1, Reason::field("notifier")),
        ]
    );
}

#[test]
fn calling_a_setter_on_a_returned_object_is_a_violation() {
    let move_customer = Method::new("OrderService", "pay", 1).call(
        Call::new(
            Expression::parameter("order", TypeRef::class("Order")),
            "getCustomer",
            1,
        )
        .then("setAddress", 1)
        .with(Expression::parameter("address", TypeRef::class("Address"))),
    );

    let verdicts = move_customer.verdicts(&shop());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 1, Reason::parameter("order")),
            Verdict::violation("setAddress", 1, Reason::return_of("getCustomer")),
        ]
    );
}

#[test]
fn calling_a_method_on_an_unassigned_local_is_a_violation() {
    let pay = Method::new("OrderService", "pay", 1).call(Call::new(
        Expression::local("customer"),
        "getWallet",
        1,
    ));

    let verdicts = pay.verdicts(&Types::new());

    assert_eq!(
        verdicts,
        [Verdict::violation(
            "getWallet",
            1,
            Reason::unresolved_type(None, Reason::unassigned_local("customer"))
        )]
    );
}

#[test]
fn calling_a_method_on_a_local_holding_a_returned_object_is_a_violation() {
    let pay = Method::new("OrderService", "pay", 1)
        .assign_var(
            "customer",
            Expression::return_of(Call::new(
                Expression::parameter("order", TypeRef::class("Order")),
                "getCustomer",
                1,
            )),
        )
        .call(Call::new(Expression::local("customer"), "getWallet", 1).then("withdraw", 1));

    let verdicts = pay.verdicts(&shop());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 1, Reason::parameter("order")),
            Verdict::violation(
                "getWallet",
                1,
                Reason::local("customer", Reason::return_of("getCustomer"))
            ),
            Verdict::violation("withdraw", 1, Reason::return_of("getWallet")),
        ]
    );
}

#[test]
fn calls_inside_an_assigned_value_are_judged_once_however_often_the_local_is_used() {
    let inspect = Method::new("OrderService", "pay", 1)
        .assign_var(
            "customer",
            Expression::return_of(Call::new(
                Expression::parameter("order", TypeRef::class("Order")),
                "getCustomer",
                1,
            )),
        )
        .call(Call::new(Expression::local("customer"), "getWallet", 1))
        .call(Call::new(Expression::local("customer"), "getAddress", 1));

    let verdicts = inspect.verdicts(&shop());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 1, Reason::parameter("order")),
            Verdict::violation(
                "getWallet",
                1,
                Reason::local("customer", Reason::return_of("getCustomer"))
            ),
            Verdict::violation(
                "getAddress",
                1,
                Reason::local("customer", Reason::return_of("getCustomer"))
            ),
        ]
    );
}

#[test]
fn calling_a_method_on_a_local_holding_a_parameter_is_allowed() {
    let pay = Method::new("OrderService", "pay", 1)
        .assign_var(
            "current",
            Expression::parameter("order", TypeRef::class("Order")),
        )
        .call(
            Call::new(Expression::local("current"), "charge", 1)
                .with(Expression::parameter("amount", TypeRef::class("Amount"))),
        );

    let verdicts = pay.verdicts(&Types::new());

    assert_eq!(
        verdicts,
        [Verdict::allowed(
            "charge",
            1,
            Reason::local("current", Reason::parameter("order"))
        )]
    );
}

#[test]
fn calling_a_method_on_a_lambda_parameter_fed_by_a_returned_collection_is_a_violation() {
    let discount = Method::new("OrderService", "pay", 1).call(
        Call::new(
            Expression::parameter("order", TypeRef::class("Order")),
            "getLines",
            1,
        )
        .then("forEach", 1)
        .with(
            Lambda::new(&["line"]).call(
                Call::new(Expression::local("line"), "applyDiscount", 1)
                    .with(Expression::parameter("rate", TypeRef::class("Rate"))),
            ),
        ),
    );

    let verdicts = discount.verdicts(&shop());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getLines", 1, Reason::parameter("order")),
            Verdict::violation(
                "applyDiscount",
                1,
                Reason::lambda_parameter("line", Reason::return_of("getLines"))
            ),
            Verdict::allowed(
                "forEach",
                1,
                Reason::classified(Classification::Container, order_lines())
            ),
        ]
    );
}

#[test]
fn calling_a_method_on_a_lambda_parameter_fed_by_an_own_field_is_allowed() {
    let ship_all = Method::new("OrderService", "pay", 1).call(
        Call::new(
            Expression::field("orders", TypeRef::class("Orders")),
            "forEach",
            1,
        )
        .with(Lambda::new(&["order"]).call(Call::new(
            Expression::local("order"),
            "ship",
            1,
        ))),
    );

    let verdicts = ship_all.verdicts(&Types::new());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed(
                "ship",
                1,
                Reason::lambda_parameter("order", Reason::field("orders"))
            ),
            Verdict::allowed("forEach", 1, Reason::field("orders")),
        ]
    );
}

#[test]
fn calling_a_method_on_an_object_created_in_the_method_is_allowed() {
    let greeting = Method::new("OrderService", "pay", 1).call(Call::new(
        Expression::created(Creation::new(TypeRef::class("StringBuilder"))),
        "append",
        1,
    ));

    let verdicts = greeting.verdicts(&Types::new());

    assert_eq!(
        verdicts,
        [Verdict::allowed(
            "append",
            1,
            Reason::created(TypeRef::class("StringBuilder"))
        )]
    );
}

#[test]
fn calls_inside_constructor_arguments_are_judged() {
    let print = Method::new("OrderService", "pay", 1).call(Call::new(
        Expression::created(
            Creation::new(TypeRef::class("Receipt")).with(Expression::return_of(
                Call::new(
                    Expression::parameter("order", TypeRef::class("Order")),
                    "getCustomer",
                    1,
                )
                .then("getName", 1),
            )),
        ),
        "print",
        1,
    ));

    let verdicts = print.verdicts(&shop());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 1, Reason::parameter("order")),
            Verdict::violation("getName", 1, Reason::return_of("getCustomer")),
            Verdict::allowed("print", 1, Reason::created(TypeRef::class("Receipt"))),
        ]
    );
}

#[test]
fn calling_a_static_method_on_a_type_is_allowed() {
    let create = Method::new("OrderService", "pay", 1).call(Call::new(
        Expression::static_type(TypeRef::class("Order")),
        "builder",
        1,
    ));

    let verdicts = create.verdicts(&Types::new());

    assert_eq!(
        verdicts,
        [Verdict::allowed(
            "builder",
            1,
            Reason::static_type(TypeRef::class("Order"))
        )]
    );
}

#[test]
fn a_lambda_body_can_assign_a_local_and_call_it() {
    let inspect_all = Method::new("OrderService", "pay", 1).call(
        Call::new(
            Expression::field(
                "orders",
                TypeRef::generic("java.util.List", vec![TypeRef::class("Order")]),
            ),
            "forEach",
            1,
        )
        .with(
            Lambda::new(&["order"])
                .assign_var(
                    "customer",
                    Expression::return_of(Call::new(Expression::local("order"), "getCustomer", 1)),
                )
                .call(Call::new(Expression::local("customer"), "getWallet", 1)),
        ),
    );

    let verdicts = inspect_all.verdicts(&shop());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed(
                "getCustomer",
                1,
                Reason::lambda_parameter("order", Reason::field("orders"))
            ),
            Verdict::violation(
                "getWallet",
                1,
                Reason::local("customer", Reason::return_of("getCustomer"))
            ),
            Verdict::allowed("forEach", 1, Reason::field("orders")),
        ]
    );
}

#[test]
fn a_loop_variable_takes_the_standing_of_the_iterable_and_judges_it_once() {
    let discount = Method::new("OrderService", "pay", 1)
        .for_each(
            "line",
            Expression::return_of(Call::new(
                Expression::parameter("order", TypeRef::class("Order")),
                "getLines",
                1,
            )),
        )
        .call(
            Call::new(Expression::local("line"), "applyDiscount", 1)
                .with(Expression::parameter("rate", TypeRef::class("Rate"))),
        );

    let verdicts = discount.verdicts(&shop());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getLines", 1, Reason::parameter("order")),
            Verdict::violation(
                "applyDiscount",
                1,
                Reason::loop_variable("line", Reason::return_of("getLines"))
            ),
        ]
    );
}

#[test]
fn every_verdict_carries_the_line_of_its_call() {
    let pay = Method::new("OrderService", "pay", 10)
        .call(
            Call::new(
                Expression::parameter("order", TypeRef::class("Order")),
                "getCustomer",
                11,
            )
            .then("getWallet", 12)
            .then("withdraw", 13),
        )
        .call(Call::new(Expression::This, "audit", 14));

    let verdicts = pay.verdicts(&shop());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 11, Reason::parameter("order")),
            Verdict::violation("getWallet", 12, Reason::return_of("getCustomer")),
            Verdict::violation("withdraw", 13, Reason::return_of("getWallet")),
            Verdict::allowed("audit", 14, Reason::This),
        ]
    );
}

#[test]
fn a_method_knows_its_class_its_name_and_its_line() {
    let pay = Method::new("com.shop.OrderService$Payments", "pay", 42);

    assert_eq!(
        (pay.class_name(), pay.name(), pay.line()),
        ("com.shop.OrderService$Payments", "pay", 42)
    );
}

#[test]
fn calling_a_method_on_a_literal_is_allowed() {
    let shout =
        Method::new("Greeter", "shout", 1).call(Call::new(Expression::literal(), "toUpperCase", 2));

    let verdicts = shout.verdicts(&Types::new());

    assert_eq!(
        verdicts,
        [Verdict::allowed("toUpperCase", 2, Reason::Literal)]
    );
}

fn shop() -> Types {
    Types::new()
        .with_shape(
            TypeShape::new("Order", TypeKind::Class)
                .with_method(
                    MethodShape::new("getCustomer")
                        .reading("customer")
                        .returning(TypeRef::class("Customer")),
                )
                .with_method(
                    MethodShape::new("getLines")
                        .reading("lines")
                        .returning(order_lines()),
                ),
        )
        .with_shape(
            TypeShape::new("Customer", TypeKind::Class)
                .with_method(
                    MethodShape::new("getWallet")
                        .reading("wallet")
                        .returning(TypeRef::class("Wallet")),
                )
                .with_method(
                    MethodShape::new("setAddress")
                        .writing("address")
                        .with_parameters(vec![TypeRef::class("Address")]),
                )
                .with_method(MethodShape::new("rename")),
        )
        .with_shape(
            TypeShape::new("Wallet", TypeKind::Class).with_method(MethodShape::new("withdraw")),
        )
        .with_shape(
            TypeShape::new("OrderLine", TypeKind::Class)
                .with_method(MethodShape::new("applyDiscount")),
        )
}

fn order_with_lines() -> Types {
    Types::new()
        .with_shape(
            TypeShape::new("Order", TypeKind::Class).with_method(
                MethodShape::new("getLines")
                    .reading("lines")
                    .returning(order_lines()),
            ),
        )
        .with_shape(
            TypeShape::new("OrderLine", TypeKind::Class).with_method(
                MethodShape::new("getSku")
                    .reading("sku")
                    .returning(TypeRef::class("java.lang.String")),
            ),
        )
}

fn order_lines() -> TypeRef {
    TypeRef::generic("java.util.List", vec![TypeRef::class("OrderLine")])
}

#[test]
fn a_lambda_parameter_takes_the_element_type_of_the_container_feeding_it() {
    let list_skus = Method::new("OrderService", "listSkus", 1).call(
        Call::new(
            Expression::parameter("order", TypeRef::class("Order")),
            "getLines",
            2,
        )
        .then("forEach", 2)
        .with(Lambda::new(&["line"]).call(Call::new(
            Expression::local("line"),
            "getSku",
            2,
        ))),
    );

    let verdicts = list_skus.verdicts(&order_with_lines());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getLines", 2, Reason::parameter("order")),
            Verdict::allowed(
                "getSku",
                2,
                Reason::classified(Classification::DataStructure, TypeRef::class("OrderLine"))
            ),
            Verdict::allowed(
                "forEach",
                2,
                Reason::classified(Classification::Container, order_lines())
            ),
        ]
    );
}

#[test]
fn a_loop_variable_takes_the_element_type_of_the_iterable() {
    let list_skus = Method::new("OrderService", "listSkus", 1)
        .for_each(
            "line",
            Expression::return_of(Call::new(
                Expression::parameter("order", TypeRef::class("Order")),
                "getLines",
                2,
            )),
        )
        .call(Call::new(Expression::local("line"), "getSku", 3));

    let verdicts = list_skus.verdicts(&order_with_lines());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getLines", 2, Reason::parameter("order")),
            Verdict::allowed(
                "getSku",
                3,
                Reason::classified(Classification::DataStructure, TypeRef::class("OrderLine"))
            ),
        ]
    );
}

#[test]
fn a_var_local_takes_the_type_of_its_value() {
    let types = Types::new()
        .with_shape(
            TypeShape::new("Config", TypeKind::Record).with_method(
                MethodShape::new("server")
                    .reading("server")
                    .returning(TypeRef::class("Server")),
            ),
        )
        .with_shape(TypeShape::new("Server", TypeKind::Record));
    let read_port = Method::new("Connector", "portOf", 1)
        .assign_var(
            "server",
            Expression::return_of(Call::new(
                Expression::parameter("config", TypeRef::class("Config")),
                "server",
                2,
            )),
        )
        .call(Call::new(Expression::local("server"), "port", 3));

    let verdicts = read_port.verdicts(&types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("server", 2, Reason::parameter("config")),
            Verdict::allowed(
                "port",
                3,
                Reason::classified(Classification::DataStructure, TypeRef::class("Server"))
            ),
        ]
    );
}

#[test]
fn a_local_with_a_declared_type_takes_that_type_when_its_value_has_none() {
    let types = Types::new().with_shape(TypeShape::new("Server", TypeKind::Record));
    let read_port = Method::new("Connector", "portOf", 1)
        .assign(
            "server",
            TypeRef::class("Server"),
            Expression::return_of(Call::new(
                Expression::parameter("config", TypeRef::class("Config")),
                "server",
                2,
            )),
        )
        .call(Call::new(Expression::local("server"), "port", 3));

    let verdicts = read_port.verdicts(&types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("server", 2, Reason::parameter("config")),
            Verdict::allowed(
                "port",
                3,
                Reason::classified(Classification::DataStructure, TypeRef::class("Server"))
            ),
        ]
    );
}

#[test]
fn a_fluent_call_on_a_returned_object_is_allowed() {
    let builder = TypeRef::class("ReportBuilder");
    let types = Types::new()
        .with_shape(
            TypeShape::new("Order", TypeKind::Class).with_method(
                MethodShape::new("getReportBuilder")
                    .reading("builder")
                    .returning(builder.clone()),
            ),
        )
        .with_shape(
            TypeShape::new("ReportBuilder", TypeKind::Class)
                .with_method(
                    MethodShape::new("title")
                        .with_parameters(vec![TypeRef::class("java.lang.String")])
                        .returning(builder.clone()),
                )
                .with_method(
                    MethodShape::new("footer")
                        .with_parameters(vec![TypeRef::class("java.lang.String")])
                        .returning(builder.clone()),
                ),
        );
    let prepare = Method::new("OrderService", "prepare", 1).call(
        Call::new(
            Expression::parameter("order", TypeRef::class("Order")),
            "getReportBuilder",
            2,
        )
        .then("title", 2)
        .with(Expression::literal())
        .then("footer", 2)
        .with(Expression::literal()),
    );

    let verdicts = prepare.verdicts(&types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getReportBuilder", 2, Reason::parameter("order")),
            Verdict::allowed("title", 2, Reason::fluent(builder.clone())),
            Verdict::allowed("footer", 2, Reason::fluent(builder)),
        ]
    );
}

fn settings_with_behavior() -> Types {
    Types::new().with_shape(
        TypeShape::new("Settings", TypeKind::Class)
            .with_method(
                MethodShape::new("current")
                    .as_static()
                    .returning(TypeRef::class("Settings")),
            )
            .with_method(MethodShape::new("refresh")),
    )
}

#[test]
fn the_result_of_a_static_call_is_a_friend() {
    let reload = Method::new("Reloader", "reload", 1).call(
        Call::new(
            Expression::static_type(TypeRef::class("Settings")),
            "current",
            2,
        )
        .then("refresh", 2),
    );

    let verdicts = reload.verdicts(&settings_with_behavior());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed(
                "current",
                2,
                Reason::static_type(TypeRef::class("Settings"))
            ),
            Verdict::allowed("refresh", 2, Reason::static_result("current")),
        ]
    );
}

#[test]
fn the_result_of_a_factory_call_on_a_stranger_is_a_stranger() {
    let types = Types::new()
        .with_shape(
            TypeShape::new("Order", TypeKind::Class).with_method(
                MethodShape::new("getFactory")
                    .reading("factory")
                    .returning(TypeRef::class("ReportFactory")),
            ),
        )
        .with_shape(
            TypeShape::new("ReportFactory", TypeKind::Class)
                .with_method(MethodShape::new("create").returning(TypeRef::class("Report"))),
        )
        .with_shape(
            TypeShape::new("Report", TypeKind::Class).with_method(MethodShape::new("render")),
        );
    let report = Method::new("Reports", "report", 1).call(
        Call::new(
            Expression::parameter("order", TypeRef::class("Order")),
            "getFactory",
            2,
        )
        .then("create", 2)
        .then("render", 2),
    );

    let verdicts = report.verdicts(&types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getFactory", 2, Reason::parameter("order")),
            Verdict::violation("create", 2, Reason::return_of("getFactory")),
            Verdict::violation("render", 2, Reason::return_of("create")),
        ]
    );
}

fn customers_with_queries() -> Types {
    Types::new()
        .with_shape(
            TypeShape::new("Order", TypeKind::Class).with_method(
                MethodShape::new("getCustomer")
                    .reading("customer")
                    .returning(TypeRef::class("Customer")),
            ),
        )
        .with_shape(
            TypeShape::new("Customer", TypeKind::Class).with_method(
                MethodShape::new("findLine")
                    .with_parameters(vec![TypeRef::class("LineId")])
                    .returning(TypeRef::class("OrderLine")),
            ),
        )
        .with_shape(
            TypeShape::new("OrderLine", TypeKind::Class)
                .with_method(MethodShape::new("applyDiscount")),
        )
}

#[test]
fn the_result_of_a_query_called_on_a_stranger_is_a_stranger() {
    let discount = Method::new("OrderService", "discount", 1).call(
        Call::new(
            Expression::parameter("order", TypeRef::class("Order")),
            "getCustomer",
            2,
        )
        .then("findLine", 2)
        .with(Expression::parameter("id", TypeRef::class("LineId")))
        .then("applyDiscount", 2),
    );

    let verdicts = discount.verdicts(&customers_with_queries());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 2, Reason::parameter("order")),
            Verdict::violation("findLine", 2, Reason::return_of("getCustomer")),
            Verdict::violation("applyDiscount", 2, Reason::return_of("findLine")),
        ]
    );
}

#[test]
fn the_result_of_a_query_called_on_this_is_a_stranger() {
    let types = Types::new()
        .with_shape(
            TypeShape::new("OrderService", TypeKind::Class).with_method(
                MethodShape::new("findLine")
                    .with_parameters(vec![TypeRef::class("LineId")])
                    .returning(TypeRef::class("OrderLine")),
            ),
        )
        .with_shape(
            TypeShape::new("OrderLine", TypeKind::Class)
                .with_method(MethodShape::new("applyDiscount")),
        );
    let discount = Method::new("OrderService", "discount", 1).call(
        Call::new(Expression::This, "findLine", 2)
            .with(Expression::parameter("id", TypeRef::class("LineId")))
            .then("applyDiscount", 2),
    );

    let verdicts = discount.verdicts(&types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("findLine", 2, Reason::This),
            Verdict::violation("applyDiscount", 2, Reason::return_of("findLine")),
        ]
    );
}

#[test]
fn the_result_of_a_query_called_on_a_local_holding_a_field_is_a_friend() {
    let types = Types::new()
        .with_shape(
            TypeShape::new("OrderRepository", TypeKind::Class).with_method(
                MethodShape::new("findById")
                    .with_parameters(vec![TypeRef::class("OrderId")])
                    .returning(TypeRef::class("Order")),
            ),
        )
        .with_shape(TypeShape::new("Order", TypeKind::Class).with_method(MethodShape::new("ship")));
    let dispatch = Method::new("Dispatcher", "dispatch", 1)
        .assign_var(
            "repository",
            Expression::field("orders", TypeRef::class("OrderRepository")),
        )
        .call(
            Call::new(Expression::local("repository"), "findById", 3)
                .with(Expression::parameter("id", TypeRef::class("OrderId")))
                .then("ship", 3),
        );

    let verdicts = dispatch.verdicts(&types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed(
                "findById",
                3,
                Reason::local("repository", Reason::field("orders"))
            ),
            Verdict::allowed("ship", 3, Reason::query_result("findById")),
        ]
    );
}

#[test]
fn calling_a_method_on_a_returned_object_of_unknown_type_is_a_violation() {
    let pay = Method::new("OrderService", "pay", 1).call(
        Call::new(
            Expression::parameter("order", TypeRef::class("Order")),
            "getCustomer",
            2,
        )
        .then("getWallet", 2),
    );

    let verdicts = pay.verdicts(&Types::new());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 2, Reason::parameter("order")),
            Verdict::violation(
                "getWallet",
                2,
                Reason::unresolved_type(None, Reason::return_of("getCustomer"))
            ),
        ]
    );
}

#[test]
fn calling_a_method_on_a_returned_object_of_a_type_without_shape_is_a_violation() {
    let types = Types::new().with_shape(
        TypeShape::new("Order", TypeKind::Class).with_method(
            MethodShape::new("getCustomer")
                .reading("customer")
                .returning(TypeRef::class("Customer")),
        ),
    );
    let pay = Method::new("OrderService", "pay", 1).call(
        Call::new(
            Expression::parameter("order", TypeRef::class("Order")),
            "getCustomer",
            2,
        )
        .then("getWallet", 2),
    );

    let verdicts = pay.verdicts(&types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 2, Reason::parameter("order")),
            Verdict::violation(
                "getWallet",
                2,
                Reason::unresolved_type(
                    Some(TypeRef::class("Customer")),
                    Reason::return_of("getCustomer")
                )
            ),
        ]
    );
}

#[test]
fn calling_a_method_on_a_returned_value_is_allowed() {
    let types = Types::new()
        .with_shape(
            TypeShape::new("Order", TypeKind::Class).with_method(
                MethodShape::new("getStatus")
                    .reading("status")
                    .returning(TypeRef::class("Status")),
            ),
        )
        .with_shape(TypeShape::new("Status", TypeKind::Enum));
    let describe = Method::new("Reports", "describe", 1).call(
        Call::new(
            Expression::parameter("order", TypeRef::class("Order")),
            "getStatus",
            2,
        )
        .then("name", 2),
    );

    let verdicts = describe.verdicts(&types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getStatus", 2, Reason::parameter("order")),
            Verdict::allowed(
                "name",
                2,
                Reason::classified(Classification::Value, TypeRef::class("Status"))
            ),
        ]
    );
}
