use law_of_demeter_java_source::{ClassKind, SourceFile, parse};

fn parse_ok(source: &str) -> SourceFile {
    parse("A.java", source).unwrap()
}

fn names(file: &SourceFile) -> Vec<&str> {
    file.classes
        .iter()
        .map(|class| class.name.as_str())
        .collect()
}

#[test]
fn a_class_is_named_by_its_package_and_name() {
    let file = parse_ok("package shop.orders;\nclass Order {}");

    assert_eq!(names(&file), ["shop.orders.Order"]);
}

#[test]
fn a_class_in_the_default_package_has_a_plain_name() {
    let file = parse_ok("class Order {}");

    assert_eq!(names(&file), ["Order"]);
}

#[test]
fn nested_classes_are_listed_after_their_outer_class_with_dollar_names() {
    let source = "package shop;\n\
        class Order {\n\
            class Line { class Part {} }\n\
            static class Builder {}\n\
        }\n\
        class Customer {}";

    let file = parse_ok(source);

    assert_eq!(
        names(&file),
        [
            "shop.Order",
            "shop.Order$Line",
            "shop.Order$Line$Part",
            "shop.Order$Builder",
            "shop.Customer"
        ]
    );
}

#[test]
fn a_nested_class_knows_its_outer_class() {
    let file = parse_ok("package shop;\nclass Order { class Line {} }");

    assert_eq!(file.classes[0].outer, None);
    assert_eq!(file.classes[1].outer.as_deref(), Some("shop.Order"));
}

#[test]
fn every_kind_of_type_declaration_is_listed_with_its_kind() {
    let source = "class C {}\n\
        interface I {}\n\
        enum E { A }\n\
        record R(int x) {}\n\
        @interface N {}";

    let file = parse_ok(source);

    let kinds: Vec<ClassKind> = file.classes.iter().map(|class| class.kind).collect();
    assert_eq!(
        kinds,
        [
            ClassKind::Class,
            ClassKind::Interface,
            ClassKind::Enum,
            ClassKind::Record,
            ClassKind::Annotation
        ]
    );
}

#[test]
fn types_nested_in_enums_and_interfaces_are_listed() {
    let source = "enum E { A; class InEnum {} }\ninterface I { class InInterface {} }";

    let file = parse_ok(source);

    assert_eq!(names(&file), ["E", "E$InEnum", "I", "I$InInterface"]);
}
