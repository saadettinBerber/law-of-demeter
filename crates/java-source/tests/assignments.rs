mod support;

use law_of_demeter_java_source::{Expression, LocalVariable, Statement};
use support::{body_of, call, field, local, parameter};

fn assignment(target: Expression, value: Expression) -> Statement {
    Statement::Expression(Expression::Assignment {
        target: Box::new(target),
        value: Box::new(value),
    })
}

#[test]
fn assigning_to_a_declared_local_is_an_assignment() {
    let body = body_of("void f(Order order) {\n Customer c;\n c = order.getCustomer();\n}");

    assert_eq!(
        body,
        [
            Statement::Local(LocalVariable {
                name: "c".to_owned(),
                declared_type: law_of_demeter_java_source::DeclaredType::class("shop.Customer"),
                initializer: None,
                line: 3,
            }),
            assignment(
                local("c"),
                call(parameter("order"), "getCustomer", vec![], 4)
            ),
        ]
    );
}

#[test]
fn assigning_to_a_field_is_an_assignment_whether_or_not_it_is_written_with_this() {
    let body =
        body_of("private Wallet wallet;\nvoid f(Wallet w) {\n this.wallet = w;\n wallet = w;\n}");

    assert_eq!(
        body,
        [
            assignment(field("wallet"), parameter("w")),
            assignment(field("wallet"), parameter("w")),
        ]
    );
}

#[test]
fn an_assignment_inside_an_assignment_nests_in_the_value() {
    let body = body_of("void f(Order a) {\n Order p, q;\n p = q = a;\n}");

    assert_eq!(
        body[2],
        assignment(
            local("p"),
            Expression::Assignment {
                target: Box::new(local("q")),
                value: Box::new(parameter("a")),
            }
        )
    );
}

#[test]
fn a_compound_assignment_stays_an_operation() {
    let body = body_of("private int total;\nvoid f(Order a) {\n total += a.size();\n}");

    assert_eq!(
        body,
        [Statement::Expression(Expression::Operation(vec![
            field("total"),
            call(parameter("a"), "size", vec![], 4)
        ]))]
    );
}

#[test]
fn an_assignment_inside_a_condition_is_an_expression_of_its_own() {
    let body = body_of("void f(Order a) {\n Order p;\n while ((p = a.next()) != null) {\n }\n}");

    let Statement::While { condition, .. } = &body[1] else {
        panic!("expected a while, got {body:?}");
    };
    let Expression::Operation(operands) = condition else {
        panic!("expected an operation, got {condition:?}");
    };
    assert_eq!(
        operands[0],
        Expression::Assignment {
            target: Box::new(local("p")),
            value: Box::new(call(parameter("a"), "next", vec![], 4)),
        }
    );
}
