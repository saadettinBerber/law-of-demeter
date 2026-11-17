use std::path::Path;

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::Method;
use law_of_demeter_java_source::map_source;

fn mapped_method(source: &str, class: &str, name: &str) -> Method {
    map_source(Path::new("A.java"), source, &Types::new())
        .unwrap()
        .methods
        .into_iter()
        .find(|method| method.class_name() == class && method.name() == name)
        .unwrap()
}

#[test]
fn a_method_of_an_anonymous_class_knows_the_method_it_is_declared_in() {
    let source =
        "package shop;\nclass Order {\n void pay() {\n  new Object() { void run() {} };\n }\n}";

    let run = mapped_method(source, "shop.Order$1", "run");

    assert_eq!(
        run,
        Method::new("shop.Order$1", "run", 4).declared_within("shop.Order", "pay")
    );
}

#[test]
fn a_local_class_in_a_constructor_is_declared_within_the_constructor() {
    let source = "package shop;\nclass Order {\n Order() {\n  class Audit { void log() {} }\n }\n}";

    let log = mapped_method(source, "shop.Order$1Audit", "log");

    assert_eq!(
        log,
        Method::new("shop.Order$1Audit", "log", 4).declared_within("shop.Order", "<init>")
    );
}

#[test]
fn an_anonymous_class_in_a_field_initializer_is_declared_within_the_initializer() {
    let source = "package shop;\nclass Order {\n Object task = new Object() { void run() {} };\n}";

    let run = mapped_method(source, "shop.Order$1", "run");

    assert_eq!(
        run,
        Method::new("shop.Order$1", "run", 3).declared_within("shop.Order", "<instance-init>")
    );
}

#[test]
fn a_class_nested_in_the_method_of_an_anonymous_class_is_declared_within_both_methods() {
    let source = "package shop;\nclass Order {\n void pay() {\n  new Object() {\n   void run() {\n    new Object() { void log() {} };\n   }\n  };\n }\n}";

    let log = mapped_method(source, "shop.Order$1$1", "log");

    assert_eq!(
        log,
        Method::new("shop.Order$1$1", "log", 6)
            .declared_within("shop.Order$1", "run")
            .declared_within("shop.Order", "pay")
    );
}
