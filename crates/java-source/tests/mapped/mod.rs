#![allow(dead_code)]

use std::path::Path;

use law_of_demeter_core::domain::method::{Call, Expression, Method};
use law_of_demeter_core::domain::type_shape::TypeRef;
use law_of_demeter_java_source::map_source;

pub const KNOWN_TYPES: &[&str] = &[
    "Order",
    "Order$Builder",
    "Customer",
    "Wallet",
    "Address",
    "OrderLine",
    "Status",
    "Notifier",
    "java.lang.String",
    "java.lang.Object",
    "java.lang.StringBuilder",
    "java.lang.IllegalStateException",
    "java.lang.Throwable",
    "java.io.IOException",
    "java.util.List",
    "x.Y",
];

pub fn mapped_methods(source: &str) -> Vec<Method> {
    map_source(Path::new("OrderService.java"), source, &|name| {
        KNOWN_TYPES.contains(&name)
    })
    .unwrap()
    .methods
}

pub fn method_f(source: &str) -> Method {
    mapped_methods(source)
        .into_iter()
        .find(|method| method.name() == "f")
        .unwrap()
}

pub fn method_in(source: &str, class_name: &str, name: &str) -> Method {
    mapped_methods(source)
        .into_iter()
        .find(|method| method.class_name() == class_name && method.name() == name)
        .unwrap()
}

pub fn class(name: &str) -> TypeRef {
    TypeRef::class(name)
}

pub fn parameter(name: &str, type_ref: TypeRef) -> Expression {
    Expression::parameter(name, type_ref)
}

pub fn call(receiver: Expression, name: &str, line: usize) -> Call {
    Call::new(receiver, name, line)
}
