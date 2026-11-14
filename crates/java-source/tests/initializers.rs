mod support;

use law_of_demeter_java_source::{
    Class, DeclaredType, Expression, INSTANCE_INITIALIZER, Method, STATIC_INITIALIZER, Statement,
    parse_with,
};
use support::{KNOWN_TYPES, call, created, field, own_call, type_ref};

fn first_class(source: &str) -> Class {
    let source = format!("package shop; {source}");
    parse_with("A.java", &source, &|name| KNOWN_TYPES.contains(&name))
        .unwrap()
        .classes
        .remove(0)
}

fn method<'a>(class: &'a Class, name: &str) -> &'a Method {
    class
        .methods
        .iter()
        .find(|method| method.name == name)
        .unwrap_or_else(|| panic!("no method {name} in {:?}", class.methods))
}

fn initialization(field_name: &str, value: Expression) -> Statement {
    Statement::Expression(Expression::Operation(vec![field(field_name), value]))
}

#[test]
fn instance_field_initializers_become_one_synthetic_instance_method() {
    let class = first_class(
        "class A {\n private Notifier notifier;\n private Wallet wallet = notifier.open();\n}",
    );

    let init = method(&class, INSTANCE_INITIALIZER);

    assert_eq!(init.name, "<instance-init>");
    assert_eq!(init.line, 3);
    assert!(!init.is_static);
    assert!(!init.is_constructor());
    assert!(init.is_initializer());
    assert_eq!(init.parameters, []);
    assert_eq!(init.returns, Some(DeclaredType::primitive("void")));
    assert_eq!(
        init.body,
        Some(vec![initialization(
            "wallet",
            call(field("notifier"), "open", vec![], 3)
        )])
    );
}

#[test]
fn static_field_initializers_become_one_synthetic_static_method() {
    let class = first_class("class A {\n static final Order DEFAULT = Order.builder().build();\n}");

    let init = method(&class, STATIC_INITIALIZER);

    assert_eq!(init.name, "<static-init>");
    assert!(init.is_static);
    assert_eq!(
        init.body,
        Some(vec![initialization(
            "DEFAULT",
            call(
                call(type_ref("shop.Order"), "builder", vec![], 2),
                "build",
                vec![],
                2
            )
        )])
    );
    assert!(class.methods.iter().all(|m| m.name != INSTANCE_INITIALIZER));
}

#[test]
fn blocks_and_field_initializers_keep_their_source_order() {
    let source =
        "class A {\n int a = first();\n { second(); }\n int b = third();\n static { fourth(); }\n}";

    let class = first_class(source);

    assert_eq!(
        method(&class, INSTANCE_INITIALIZER).body,
        Some(vec![
            initialization("a", own_call("first", vec![], 2)),
            Statement::Block(vec![Statement::Expression(own_call("second", vec![], 3))]),
            initialization("b", own_call("third", vec![], 4)),
        ])
    );
    assert_eq!(
        method(&class, STATIC_INITIALIZER).body,
        Some(vec![Statement::Block(vec![Statement::Expression(
            own_call("fourth", vec![], 5)
        )])])
    );
    assert_eq!(method(&class, INSTANCE_INITIALIZER).line, 2);
    assert_eq!(method(&class, STATIC_INITIALIZER).line, 5);
}

#[test]
fn a_declaration_with_several_names_initializes_each_one_that_has_a_value() {
    let class = first_class("class A {\n int a = f(), b, c = g();\n}");

    assert_eq!(
        method(&class, INSTANCE_INITIALIZER).body,
        Some(vec![
            initialization("a", own_call("f", vec![], 2)),
            initialization("c", own_call("g", vec![], 2)),
        ])
    );
}

#[test]
fn fields_without_values_add_no_initializer_method() {
    let class = first_class("class A {\n int a;\n static int b;\n void f() {}\n}");

    let names: Vec<&str> = class.methods.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, ["f"]);
}

#[test]
fn initializer_methods_come_after_the_declared_methods() {
    let class = first_class("class A {\n int a = 1;\n static int b = 2;\n void f() {}\n}");

    let names: Vec<&str> = class.methods.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, ["f", STATIC_INITIALIZER, INSTANCE_INITIALIZER]);
}

#[test]
fn a_lambda_in_a_field_initializer_keeps_its_calls() {
    let class =
        first_class("class A {\n Comparator c = (a, b) -> a.rank().compareTo(b.rank());\n}");

    let Some(body) = &method(&class, INSTANCE_INITIALIZER).body else {
        panic!("expected a body");
    };
    let Statement::Expression(Expression::Operation(operands)) = &body[0] else {
        panic!("expected an initialization, got {body:?}");
    };
    let Expression::Lambda(lambda) = &operands[1] else {
        panic!("expected a lambda, got {operands:?}");
    };
    assert_eq!(lambda.parameters.len(), 2);
    assert_eq!(lambda.body.len(), 1);
}

#[test]
fn enum_constants_with_arguments_are_initialized_in_the_static_method() {
    let source =
        "enum Color {\n RED(helper.make()), GREEN, BLUE(1);\n private Color(Object o) {}\n}";

    let class = first_class(source);

    assert_eq!(
        method(&class, STATIC_INITIALIZER).body,
        Some(vec![
            initialization(
                "RED",
                created(
                    "shop.Color",
                    vec![call(
                        Expression::Unresolved("helper".to_owned()),
                        "make",
                        vec![],
                        2
                    )],
                    2
                )
            ),
            initialization("BLUE", created("shop.Color", vec![Expression::Literal], 2)),
        ])
    );
}

#[test]
fn interface_constants_are_initialized_in_the_static_method() {
    let class = first_class("interface Limits {\n int MAX = Config.max();\n}");

    assert_eq!(
        method(&class, STATIC_INITIALIZER).body,
        Some(vec![initialization(
            "MAX",
            call(type_ref("shop.Config"), "max", vec![], 2)
        )])
    );
}

#[test]
fn an_instance_block_that_starts_with_a_nested_block_keeps_every_statement() {
    let class = first_class("class A {\n {\n { inner(); }\n outer();\n }\n}");

    assert_eq!(
        method(&class, INSTANCE_INITIALIZER).body,
        Some(vec![Statement::Block(vec![
            Statement::Block(vec![Statement::Expression(own_call("inner", vec![], 3))]),
            Statement::Expression(own_call("outer", vec![], 4)),
        ])])
    );
}
