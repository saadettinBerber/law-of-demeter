use std::path::Path;

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::Method;
use law_of_demeter_java_source::{MappedFile, map_source};

fn map(source: &str) -> MappedFile {
    map_source(Path::new("A.java"), source, &Types::new()).unwrap()
}

#[test]
fn every_class_declared_in_the_file_is_listed_by_binary_name() {
    let source = "package shop;\nclass Order {\n class Line {}\n void f() { new Object() {}; }\n}\nenum Color { RED }";

    let mapped = map(source);

    assert_eq!(
        mapped.classes,
        [
            "shop.Order",
            "shop.Order$Line",
            "shop.Order$1",
            "shop.Color"
        ]
    );
}

#[test]
fn every_method_constructor_and_initializer_becomes_a_method_with_its_line() {
    let source = "package shop;\nclass Order {\n Order() {}\n void f() {}\n static void g() {}\n int total = 1;\n}";

    let mapped = map(source);

    assert_eq!(
        mapped
            .methods
            .iter()
            .map(|m| (m.class_name(), m.name(), m.line()))
            .collect::<Vec<_>>(),
        [
            ("shop.Order", "<init>", 3),
            ("shop.Order", "f", 4),
            ("shop.Order", "g", 5),
            ("shop.Order", "<instance-init>", 6),
        ]
    );
}

#[test]
fn methods_of_nested_and_anonymous_classes_belong_to_their_own_class() {
    let source = "package shop;\nclass Order {\n void f() { new Object() { void m() {} }; }\n class Line { void g() {} }\n}";

    let mapped = map(source);

    let owners: Vec<(&str, &str)> = mapped
        .methods
        .iter()
        .map(|method| (method.class_name(), method.name()))
        .collect();
    assert_eq!(
        owners,
        [
            ("shop.Order", "f"),
            ("shop.Order$1", "m"),
            ("shop.Order$Line", "g"),
        ]
    );
}

#[test]
fn a_method_without_a_body_is_an_empty_method() {
    let mapped = map("package shop;\ninterface Shape {\n void draw();\n}");

    assert_eq!(mapped.methods, [Method::new("shop.Shape", "draw", 3)]);
}

#[test]
fn a_syntax_error_is_reported_with_the_given_path_and_line() {
    let error = map_source(
        Path::new("shop/Broken.java"),
        "class A {\n void f() {\n  int x = ;\n }\n}",
        &Types::new(),
    )
    .unwrap_err();

    assert_eq!(error.file, "shop/Broken.java");
    assert_eq!(error.line, 3);
}
