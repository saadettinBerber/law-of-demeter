mod support;

use law_of_demeter_java_source::{DeclaredType, Expression, PatternBinding, Statement};
use support::{body_of, call, local, parameter};

fn binding(name: &str, declared_type: DeclaredType) -> PatternBinding {
    PatternBinding {
        name: name.to_owned(),
        declared_type,
        is_component: false,
    }
}

fn component(name: &str, declared_type: DeclaredType) -> PatternBinding {
    PatternBinding {
        is_component: true,
        ..binding(name, declared_type)
    }
}

fn instance_of(body: &[Statement]) -> (Expression, DeclaredType, Vec<PatternBinding>) {
    let Statement::If { condition, .. } = &body[0] else {
        panic!("expected an if, got {body:?}");
    };
    let Expression::InstanceOf {
        operand,
        tested,
        bindings,
    } = condition
    else {
        panic!("expected an instanceof, got {condition:?}");
    };
    (*operand.clone(), tested.clone(), bindings.clone())
}

#[test]
fn an_instanceof_with_a_binding_declares_it_with_the_tested_type() {
    let body = body_of("void f(Object o) {\n if (o instanceof Order x) {\n x.go();\n }\n}");

    let (operand, tested, bindings) = instance_of(&body);

    assert_eq!(operand, parameter("o"));
    assert_eq!(tested, DeclaredType::class("shop.Order"));
    assert_eq!(bindings, [binding("x", DeclaredType::class("shop.Order"))]);
}

#[test]
fn an_instanceof_without_a_binding_declares_nothing() {
    let body = body_of("void f(Object o) {\n if (o instanceof Order) {\n }\n}");

    let (_, tested, bindings) = instance_of(&body);

    assert_eq!(tested, DeclaredType::class("shop.Order"));
    assert_eq!(bindings, []);
}

#[test]
fn a_binding_stays_visible_in_the_condition_after_it_and_in_the_branch() {
    let body =
        body_of("void f(Object o) {\n if (o instanceof Order x && x.ok()) {\n x.go();\n }\n}");

    let Statement::If {
        condition,
        then_branch,
        ..
    } = &body[0]
    else {
        panic!("expected an if, got {body:?}");
    };
    let Expression::Operation(operands) = condition else {
        panic!("expected an operation, got {condition:?}");
    };
    assert_eq!(operands[1], call(local("x"), "ok", vec![], 3));
    assert_eq!(
        then_branch,
        &[Statement::Expression(call(local("x"), "go", vec![], 4))]
    );
}

#[test]
fn a_record_pattern_binds_its_components_with_their_declared_types() {
    let body = body_of(
        "void f(Object o) {\n if (o instanceof Order(var a, Customer b, Wallet(var c))) {\n }\n}",
    );

    let (_, tested, bindings) = instance_of(&body);

    assert_eq!(tested, DeclaredType::class("shop.Order"));
    assert_eq!(
        bindings,
        [
            component("a", DeclaredType::Var),
            component("b", DeclaredType::class("shop.Customer")),
            component("c", DeclaredType::Var),
        ]
    );
}

#[test]
fn a_switch_rule_with_a_type_pattern_keeps_its_bindings_guard_and_body() {
    let body = body_of(
        "void f(Object o) {\n switch (o) {\n case Order x when x.ok() -> x.go();\n default -> {}\n }\n}",
    );

    let Statement::Expression(Expression::Switch(switch)) = &body[0] else {
        panic!("expected a switch, got {body:?}");
    };
    assert_eq!(
        switch.branches[0].bindings,
        [binding("x", DeclaredType::class("shop.Order"))]
    );
    assert_eq!(
        switch.branches[0].guard,
        Some(call(local("x"), "ok", vec![], 4))
    );
    assert_eq!(
        switch.branches[0].body,
        [Statement::Expression(call(local("x"), "go", vec![], 4))]
    );
    assert_eq!(switch.branches[1].bindings, []);
}

#[test]
fn a_switch_record_pattern_binds_its_components() {
    let body = body_of(
        "void f(Object o) {\n switch (o) {\n case Order(var a, Customer b) -> a.go();\n default -> {}\n }\n}",
    );

    let Statement::Expression(Expression::Switch(switch)) = &body[0] else {
        panic!("expected a switch, got {body:?}");
    };
    assert_eq!(
        switch.branches[0].bindings,
        [
            component("a", DeclaredType::Var),
            component("b", DeclaredType::class("shop.Customer")),
        ]
    );
    assert_eq!(
        switch.branches[0].body,
        [Statement::Expression(call(local("a"), "go", vec![], 4))]
    );
}

#[test]
fn a_binding_made_in_a_guard_is_visible_in_the_body_but_is_not_a_branch_binding() {
    let source = "void f(Object o, Object p) {\n switch (o) {\n case Order x when p instanceof Customer c -> c.go();\n default -> {}\n }\n}";

    let body = body_of(source);

    let Statement::Expression(Expression::Switch(switch)) = &body[0] else {
        panic!("expected a switch, got {body:?}");
    };
    assert_eq!(
        switch.branches[0].bindings,
        [binding("x", DeclaredType::class("shop.Order"))]
    );
    assert_eq!(
        switch.branches[0].guard,
        Some(Expression::InstanceOf {
            operand: Box::new(parameter("p")),
            tested: DeclaredType::class("shop.Customer"),
            bindings: vec![binding("c", DeclaredType::class("shop.Customer"))],
        })
    );
    assert_eq!(
        switch.branches[0].body,
        [Statement::Expression(call(local("c"), "go", vec![], 4))]
    );
}

#[test]
fn a_catch_parameter_keeps_its_declared_types_and_is_a_local_in_the_catch_body() {
    let source = "void f() {\n try {\n run();\n } catch (IllegalStateException | Order e) {\n e.go();\n }\n}";

    let body = body_of(source);

    let Statement::Try { catches, .. } = &body[0] else {
        panic!("expected a try, got {body:?}");
    };
    assert_eq!(catches[0].name, "e");
    assert_eq!(
        catches[0].types,
        [
            DeclaredType::class("java.lang.IllegalStateException"),
            DeclaredType::class("shop.Order")
        ]
    );
    assert_eq!(
        catches[0].body,
        [Statement::Expression(call(local("e"), "go", vec![], 6))]
    );
}
