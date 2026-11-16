mod support;

use law_of_demeter_java_source::{DeclaredType, Expression, LiteralType, LocalVariable, Statement};
use support::{body_of, call, field, local, own_call, parameter, type_ref, unresolved};

fn expression_statement(expression: Expression) -> Statement {
    Statement::Expression(expression)
}

#[test]
fn a_call_on_a_parameter_has_the_parameter_as_receiver() {
    let body = body_of("void f(Order order) {\n order.getCustomer();\n}");

    assert_eq!(
        body,
        [expression_statement(call(
            parameter("order"),
            "getCustomer",
            vec![],
            3
        ))]
    );
}

#[test]
fn a_chain_nests_each_call_in_the_receiver_of_the_next() {
    let body = body_of(
        "void f(Order order, int amount) {\n order.getCustomer().getWallet().withdraw(amount);\n}",
    );

    assert_eq!(
        body,
        [expression_statement(call(
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
    );
}

#[test]
fn each_call_of_a_chain_has_the_line_of_its_own_name() {
    let body = body_of("void f(Order order) {\n order\n .getCustomer()\n .getWallet();\n}");

    assert_eq!(
        body,
        [expression_statement(call(
            call(parameter("order"), "getCustomer", vec![], 4),
            "getWallet",
            vec![],
            5
        ))]
    );
}

#[test]
fn a_local_variable_keeps_its_declared_type_and_initializer() {
    let body = body_of(
        "void f(Order order) {\n Customer customer = order.getCustomer();\n customer.getWallet();\n}",
    );

    assert_eq!(
        body,
        [
            Statement::Local(LocalVariable {
                name: "customer".to_owned(),
                declared_type: DeclaredType::class("shop.Customer"),
                initializer: Some(call(parameter("order"), "getCustomer", vec![], 3)),
                line: 3,
            }),
            expression_statement(call(local("customer"), "getWallet", vec![], 4)),
        ]
    );
}

#[test]
fn a_var_declaration_has_the_var_type() {
    let body = body_of("void f(Order order) {\n var total = order.getTotal();\n}");

    assert_eq!(
        body,
        [Statement::Local(LocalVariable {
            name: "total".to_owned(),
            declared_type: DeclaredType::Var,
            initializer: Some(call(parameter("order"), "getTotal", vec![], 3)),
            line: 3,
        })]
    );
}

#[test]
fn a_declaration_with_several_names_gives_several_locals_and_none_needs_an_initializer() {
    let body = body_of("void f() {\n int a, b = 1;\n}");

    assert_eq!(
        body,
        [
            Statement::Local(LocalVariable {
                name: "a".to_owned(),
                declared_type: DeclaredType::primitive("int"),
                initializer: None,
                line: 3,
            }),
            Statement::Local(LocalVariable {
                name: "b".to_owned(),
                declared_type: DeclaredType::primitive("int"),
                initializer: Some(Expression::Literal(LiteralType::Int)),
                line: 3,
            }),
        ]
    );
}

#[test]
fn a_return_keeps_its_expression() {
    let body = body_of("Status f(Order order) {\n return order.getStatus();\n}");

    assert_eq!(
        body,
        [Statement::Return(Some(call(
            parameter("order"),
            "getStatus",
            vec![],
            3
        )))]
    );
}

#[test]
fn a_bare_return_has_no_expression() {
    let body = body_of("void f() {\n return;\n}");

    assert_eq!(body, [Statement::Return(None)]);
}

#[test]
fn a_field_receiver_is_a_field_whether_or_not_it_is_written_with_this() {
    let body = body_of(
        "private Wallet wallet;\nvoid f(int amount) {\n wallet.withdraw(amount);\n this.wallet.withdraw(amount);\n}",
    );

    assert_eq!(
        body,
        [
            expression_statement(call(
                field("wallet"),
                "withdraw",
                vec![parameter("amount")],
                4
            )),
            expression_statement(call(
                field("wallet"),
                "withdraw",
                vec![parameter("amount")],
                5
            )),
        ]
    );
}

#[test]
fn a_call_on_this_and_a_call_without_receiver_are_different() {
    let body = body_of("void f() {\n this.audit();\n audit();\n}");

    assert_eq!(
        body,
        [
            expression_statement(call(Expression::This, "audit", vec![], 3)),
            expression_statement(own_call("audit", vec![], 4)),
        ]
    );
}

#[test]
fn a_parameter_hides_a_field_of_the_same_name() {
    let body = body_of("private Wallet wallet;\nvoid f(Wallet wallet) {\n wallet.close();\n}");

    assert_eq!(
        body,
        [expression_statement(call(
            parameter("wallet"),
            "close",
            vec![],
            4
        ))]
    );
}

#[test]
fn a_local_variable_hides_a_field_of_the_same_name() {
    let members =
        "private Wallet wallet;\nvoid f() {\n Wallet wallet = open();\n wallet.close();\n}";

    let body = body_of(members);

    assert_eq!(
        body[1],
        expression_statement(call(local("wallet"), "close", vec![], 5))
    );
}

#[test]
fn a_type_name_receiver_is_a_type() {
    let body = body_of("void f() {\n Order.builder();\n}");

    assert_eq!(
        body,
        [expression_statement(call(
            type_ref("shop.Order"),
            "builder",
            vec![],
            3
        ))]
    );
}

#[test]
fn a_name_that_is_nothing_known_stays_unresolved() {
    let body = body_of("void f() {\n log.info();\n Ghost.make();\n}");

    assert_eq!(
        body,
        [
            expression_statement(call(unresolved("log"), "info", vec![], 3)),
            expression_statement(call(unresolved("Ghost"), "make", vec![], 4)),
        ]
    );
}

#[test]
fn arguments_keep_their_own_receivers() {
    let body = body_of(
        "private Notifier notifier;\nvoid f(Order order) {\n notifier.send(order.getCustomer());\n}",
    );

    assert_eq!(
        body,
        [expression_statement(call(
            field("notifier"),
            "send",
            vec![call(parameter("order"), "getCustomer", vec![], 4)],
            4
        ))]
    );
}

#[test]
fn reading_a_field_of_another_object_is_a_field_access() {
    let body = body_of("void f(Order order) {\n var s = order.status;\n var p = Status.PAID;\n}");

    let initializers: Vec<Expression> = body
        .into_iter()
        .filter_map(|statement| match statement {
            Statement::Local(local) => local.initializer,
            _ => None,
        })
        .collect();
    assert_eq!(
        initializers,
        [
            Expression::FieldAccess {
                target: Box::new(parameter("order")),
                name: "status".to_owned()
            },
            Expression::FieldAccess {
                target: Box::new(type_ref("shop.Status")),
                name: "PAID".to_owned()
            },
        ]
    );
}

#[test]
fn a_field_of_an_enclosing_class_names_its_owner() {
    let members = "private Wallet wallet;\nclass Inner {\n void f() {\n wallet.close();\n }\n}";

    let file = law_of_demeter_java_source::parse(
        "A.java",
        &format!("package shop; class OrderService {{\n{members}\n}}"),
    )
    .unwrap();

    let inner = &file.classes[1];
    assert_eq!(
        inner.methods[0].body,
        Some(vec![expression_statement(call(
            Expression::OuterField {
                owner: "shop.OrderService".to_owned(),
                name: "wallet".to_owned()
            },
            "close",
            vec![],
            5
        ))])
    );
}

#[test]
fn a_fully_qualified_static_call_is_a_type_when_the_catalog_knows_it() {
    let body = body_of("void f() {\n java.util.List.of();\n java.util.Ghost.of();\n}");

    assert_eq!(
        body[0],
        expression_statement(call(type_ref("java.util.List"), "of", vec![], 3))
    );
    assert_eq!(
        body[1],
        expression_statement(call(
            Expression::FieldAccess {
                target: Box::new(Expression::FieldAccess {
                    target: Box::new(unresolved("java")),
                    name: "util".to_owned()
                }),
                name: "Ghost".to_owned()
            },
            "of",
            vec![],
            4
        ))
    );
}

#[test]
fn a_nested_type_of_a_known_type_is_a_type_when_the_catalog_knows_it() {
    let body = body_of("void f() {\n Order.Line.of();\n}");

    assert_eq!(
        body,
        [expression_statement(call(
            type_ref("shop.Order$Line"),
            "of",
            vec![],
            3
        ))]
    );
}
