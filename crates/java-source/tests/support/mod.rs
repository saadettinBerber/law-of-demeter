#![allow(dead_code)]

use law_of_demeter_java_source::{
    Call, DeclaredType, Expression, ImplicitTarget, Method, New, Statement, TypeName, parse_with,
};

pub const KNOWN_TYPES: &[&str] = &[
    "shop.Order",
    "shop.Order$Line",
    "shop.Order$Builder",
    "shop.Customer",
    "shop.CustomerId",
    "shop.Wallet",
    "shop.Address",
    "shop.OrderLine",
    "shop.OrderId",
    "shop.OrderShippedEvent",
    "shop.Session",
    "shop.Status",
    "shop.Config",
    "shop.Server",
    "shop.ShippingRequest",
    "shop.ReportFactory",
    "shop.Notifier",
    "shop.Orders",
    "java.lang.String",
    "java.lang.StringBuilder",
    "java.lang.IllegalStateException",
    "java.util.List",
    "java.time.Instant",
    "java.time.Duration",
    "java.time.temporal.ChronoUnit",
];

pub fn method_named(members: &str, name: &str) -> Method {
    method_in("", "", members, name)
}

pub fn method_in(imports: &str, fields: &str, members: &str, name: &str) -> Method {
    let source = format!("package shop; {imports} class OrderService {{ {fields}\n{members}\n}}");
    let file = parse_with("OrderService.java", &source, &|candidate| {
        KNOWN_TYPES.contains(&candidate)
    })
    .unwrap();
    file.classes
        .into_iter()
        .flat_map(|class| class.methods)
        .find(|method| method.name == name)
        .unwrap()
}

pub fn body_of(members: &str) -> Vec<Statement> {
    method_named(members, "f").body.unwrap()
}

pub fn parameter(name: &str) -> Expression {
    Expression::Parameter(name.to_owned())
}

pub fn local(name: &str) -> Expression {
    Expression::Local(name.to_owned())
}

pub fn field(name: &str) -> Expression {
    Expression::Field(name.to_owned())
}

pub fn type_ref(binary_name: &str) -> Expression {
    Expression::Type(TypeName::Resolved(binary_name.to_owned()))
}

pub fn unresolved(name: &str) -> Expression {
    Expression::Unresolved(name.to_owned())
}

pub fn call(
    receiver: Expression,
    name: &str,
    arguments: Vec<Expression>,
    line: usize,
) -> Expression {
    Expression::Call(Call {
        receiver: Some(Box::new(receiver)),
        implicit_target: None,
        name: name.to_owned(),
        arguments,
        line,
    })
}

pub fn own_call(name: &str, arguments: Vec<Expression>, line: usize) -> Expression {
    Expression::Call(Call {
        receiver: None,
        implicit_target: Some(ImplicitTarget::OwnMethod),
        name: name.to_owned(),
        arguments,
        line,
    })
}

pub fn created(binary_name: &str, arguments: Vec<Expression>, line: usize) -> Expression {
    Expression::New(New {
        created: DeclaredType::class(binary_name),
        arguments,
        line,
    })
}
