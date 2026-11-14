use law_of_demeter_java_source::{Class, DeclaredType, Field, parse};

fn first_class(source: &str) -> Class {
    parse("A.java", source).unwrap().classes.remove(0)
}

fn field(name: &str, declared_type: DeclaredType, is_static: bool) -> Field {
    Field {
        name: name.to_owned(),
        declared_type,
        is_static,
    }
}

#[test]
fn fields_keep_their_names_declared_types_and_static_modifier() {
    let class = first_class("class A { private int count; static final String NAME = \"a\"; }");

    assert_eq!(
        class.fields,
        [
            field("count", DeclaredType::primitive("int"), false),
            field("NAME", DeclaredType::unresolved("String"), true),
        ]
    );
}

#[test]
fn one_declaration_with_several_names_gives_several_fields() {
    let class = first_class("class A { int x, y[]; }");

    assert_eq!(
        class.fields,
        [
            field("x", DeclaredType::primitive("int"), false),
            field(
                "y",
                DeclaredType::array(DeclaredType::primitive("int")),
                false
            ),
        ]
    );
}

#[test]
fn record_components_are_fields() {
    let class = first_class("record Server(String host, int port) {}");

    assert_eq!(
        class.fields,
        [
            field("host", DeclaredType::unresolved("String"), false),
            field("port", DeclaredType::primitive("int"), false),
        ]
    );
}

#[test]
fn enum_constants_are_static_fields_of_the_enum_type() {
    let class = first_class("package shop;\nenum Status { PAID, SHIPPED; int code; }");

    assert_eq!(
        class.fields,
        [
            field("PAID", DeclaredType::class("shop.Status"), true),
            field("SHIPPED", DeclaredType::class("shop.Status"), true),
            field("code", DeclaredType::primitive("int"), false),
        ]
    );
}

#[test]
fn interface_constants_are_static() {
    let class = first_class("interface Limits { int MAX = 3; }");

    assert_eq!(
        class.fields,
        [field("MAX", DeclaredType::primitive("int"), true)]
    );
}
