mod support;

use law_of_demeter_java_source::{Catch, DeclaredType, Expression, LocalVariable, Statement};
use support::{body_of, call, local, parameter};

fn go(receiver: Expression, line: usize) -> Statement {
    Statement::Expression(call(receiver, "go", vec![], line))
}

#[test]
fn an_if_keeps_its_condition_and_both_branches() {
    let body = body_of(
        "void f(Order a, Order b, Order c) {\n if (a.ok()) {\n b.go();\n } else {\n c.go();\n }\n}",
    );

    assert_eq!(
        body,
        [Statement::If {
            condition: call(parameter("a"), "ok", vec![], 3),
            then_branch: vec![go(parameter("b"), 4)],
            else_branch: vec![go(parameter("c"), 6)],
        }]
    );
}

#[test]
fn an_if_without_braces_or_else_still_has_its_branch() {
    let body = body_of("void f(Order a) {\n if (a.ok()) a.go();\n}");

    assert_eq!(
        body,
        [Statement::If {
            condition: call(parameter("a"), "ok", vec![], 3),
            then_branch: vec![go(parameter("a"), 3)],
            else_branch: vec![],
        }]
    );
}

#[test]
fn a_for_loop_keeps_init_condition_update_and_body() {
    let body = body_of(
        "void f(Order order) {\n for (int i = 0; order.more(i); order.next()) {\n order.go();\n }\n}",
    );

    assert_eq!(
        body,
        [Statement::For {
            init: vec![Statement::Local(LocalVariable {
                name: "i".to_owned(),
                declared_type: DeclaredType::primitive("int"),
                initializer: Some(Expression::Literal),
                line: 3,
            })],
            condition: Some(call(parameter("order"), "more", vec![local("i")], 3)),
            update: vec![call(parameter("order"), "next", vec![], 3)],
            body: vec![go(parameter("order"), 4)],
        }]
    );
}

#[test]
fn an_enhanced_for_has_its_variable_and_what_it_iterates() {
    let body =
        body_of("void f(Order order) {\n for (Line line : order.getLines()) {\n line.go();\n }\n}");

    assert_eq!(
        body,
        [Statement::ForEach {
            variable: LocalVariable {
                name: "line".to_owned(),
                declared_type: DeclaredType::unresolved("Line"),
                initializer: None,
                line: 3,
            },
            iterable: call(parameter("order"), "getLines", vec![], 3),
            body: vec![go(local("line"), 4)],
        }]
    );
}

#[test]
fn a_while_and_a_do_while_keep_condition_and_body() {
    let body = body_of(
        "void f(Order a) {\n while (a.ok()) {\n a.go();\n }\n do {\n a.go();\n } while (a.ok());\n}",
    );

    assert_eq!(
        body,
        [
            Statement::While {
                condition: call(parameter("a"), "ok", vec![], 3),
                body: vec![go(parameter("a"), 4)],
            },
            Statement::While {
                condition: call(parameter("a"), "ok", vec![], 8),
                body: vec![go(parameter("a"), 7)],
            },
        ]
    );
}

#[test]
fn a_try_keeps_resources_body_catches_and_finally() {
    let source = "void f(Order a) {\n try (Res r = a.open()) {\n r.go();\n } catch (IOException | RuntimeException e) {\n e.go();\n } finally {\n a.go();\n }\n}";

    let body = body_of(source);

    assert_eq!(
        body,
        [Statement::Try {
            resources: vec![Statement::Local(LocalVariable {
                name: "r".to_owned(),
                declared_type: DeclaredType::unresolved("Res"),
                initializer: Some(call(parameter("a"), "open", vec![], 3)),
                line: 3,
            })],
            body: vec![go(local("r"), 4)],
            catches: vec![Catch {
                name: "e".to_owned(),
                types: vec![
                    DeclaredType::unresolved("IOException"),
                    DeclaredType::unresolved("RuntimeException")
                ],
                body: vec![go(local("e"), 6)],
            }],
            finally: vec![go(parameter("a"), 8)],
        }]
    );
}

#[test]
fn a_switch_has_a_selector_and_a_branch_for_every_case() {
    let source = "void f(Order a) {\n switch (a.kind()) {\n case 1:\n a.go();\n break;\n default:\n a.stop();\n }\n}";

    let body = body_of(source);

    let Statement::Expression(Expression::Switch(switch)) = &body[0] else {
        panic!("expected a switch, got {body:?}");
    };
    assert_eq!(*switch.selector, call(parameter("a"), "kind", vec![], 3));
    let bodies: Vec<&Vec<Statement>> = switch.branches.iter().map(|branch| &branch.body).collect();
    assert_eq!(
        bodies,
        [
            &vec![go(parameter("a"), 5)],
            &vec![Statement::Expression(call(
                parameter("a"),
                "stop",
                vec![],
                8
            ))]
        ]
    );
}

#[test]
fn a_switch_rule_with_a_pattern_binds_a_local_and_keeps_its_guard() {
    let source = "void f(Object o) {\n switch (o) {\n case Order x when x.ok() -> x.go();\n default -> {}\n }\n}";

    let body = body_of(source);

    let Statement::Expression(Expression::Switch(switch)) = &body[0] else {
        panic!("expected a switch, got {body:?}");
    };
    assert_eq!(
        switch.branches[0].guard,
        Some(call(local("x"), "ok", vec![], 4))
    );
    assert_eq!(switch.branches[0].body, [go(local("x"), 4)]);
    assert_eq!(switch.branches[1].body, []);
}

#[test]
fn a_switch_expression_is_an_initializer_like_any_other_expression() {
    let source = "void f(Order a) {\n int n = switch (a.kind()) {\n case 1 -> a.one();\n default -> {\n yield a.other();\n }\n };\n}";

    let body = body_of(source);

    let Statement::Local(local) = &body[0] else {
        panic!("expected a local, got {body:?}");
    };
    let Some(Expression::Switch(switch)) = &local.initializer else {
        panic!("expected a switch initializer, got {local:?}");
    };
    assert_eq!(
        switch.branches[0].body,
        [Statement::Expression(call(
            parameter("a"),
            "one",
            vec![],
            4
        ))]
    );
    assert_eq!(
        switch.branches[1].body,
        [Statement::Expression(call(
            parameter("a"),
            "other",
            vec![],
            6
        ))]
    );
}

#[test]
fn a_local_goes_out_of_scope_with_its_block() {
    let body = body_of(
        "private Wallet r;\nvoid f(Order a) {\n { Wallet r = a.open(); r.go(); }\n r.go();\n}",
    );

    assert_eq!(body[1], go(Expression::Field("r".to_owned()), 5));
}

#[test]
fn a_synchronized_block_and_a_labeled_statement_keep_their_calls() {
    let body = body_of(
        "void f(Order a) {\n synchronized (a.lock()) {\n a.go();\n }\n outer: while (a.ok()) { a.go(); }\n}",
    );

    assert_eq!(
        body[0],
        Statement::Block(vec![
            Statement::Expression(call(parameter("a"), "lock", vec![], 3)),
            go(parameter("a"), 4)
        ])
    );
    assert!(matches!(body[1], Statement::While { .. }));
}

#[test]
fn the_iterated_expression_does_not_see_the_loop_variable() {
    let body = body_of(
        "private Wallet wallet;\nvoid f() {\n for (Wallet wallet : wallet.items()) {\n wallet.go();\n }\n}",
    );

    let Statement::ForEach { iterable, body, .. } = &body[0] else {
        panic!("expected a for each, got {body:?}");
    };
    assert_eq!(
        *iterable,
        call(Expression::Field("wallet".to_owned()), "items", vec![], 4)
    );
    assert_eq!(body, &[go(local("wallet"), 5)]);
}
