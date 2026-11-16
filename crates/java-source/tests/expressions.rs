mod support;

use law_of_demeter_java_source::{
    DeclaredType, Expression, Lambda, LambdaParameter, LiteralType, LocalVariable, Statement,
};
use support::{body_of, call, created, field, local, own_call, parameter, type_ref};

fn initializer_of(body: &[Statement]) -> Expression {
    match &body[0] {
        Statement::Local(LocalVariable { initializer, .. }) => initializer.clone().unwrap(),
        other => panic!("expected a local variable, got {other:?}"),
    }
}

#[test]
fn a_new_expression_keeps_its_type_arguments_and_line() {
    let body = body_of("void f(String name) {\n var b = new StringBuilder(name);\n}");

    assert_eq!(
        initializer_of(&body),
        created("java.lang.StringBuilder", vec![parameter("name")], 3)
    );
}

#[test]
fn a_call_on_a_new_object_has_the_new_object_as_receiver() {
    let body = body_of(
        "void f(String name) {\n var s = new StringBuilder().append(\"Hello \").append(name).toString();\n}",
    );

    assert_eq!(
        initializer_of(&body),
        call(
            call(
                call(
                    created("java.lang.StringBuilder", vec![], 3),
                    "append",
                    vec![Expression::Literal(LiteralType::String)],
                    3
                ),
                "append",
                vec![parameter("name")],
                3
            ),
            "toString",
            vec![],
            3
        )
    );
}

#[test]
fn a_diamond_and_a_generic_new_keep_their_arguments() {
    let body = body_of("void f() {\n List<String> a = new ArrayList<>();\n}");

    assert_eq!(
        initializer_of(&body),
        Expression::New(law_of_demeter_java_source::New {
            created: DeclaredType::unresolved("ArrayList"),
            arguments: vec![],
            anonymous_class: None,
            line: 3,
        })
    );
}

#[test]
fn a_lambda_has_parameters_that_hide_everything_else() {
    let body = body_of(
        "void f(Order order, int rate) {\n order.getLines().forEach(line -> line.applyDiscount(rate));\n}",
    );

    assert_eq!(
        body,
        [Statement::Expression(call(
            call(parameter("order"), "getLines", vec![], 3),
            "forEach",
            vec![Expression::Lambda(Lambda {
                parameters: vec![LambdaParameter {
                    name: "line".to_owned(),
                    declared_type: None
                }],
                body: vec![Statement::Return(Some(call(
                    Expression::LambdaParameter("line".to_owned()),
                    "applyDiscount",
                    vec![parameter("rate")],
                    3
                )))],
            })],
            3
        ))]
    );
}

#[test]
fn lambda_parameters_can_be_typed_and_the_body_can_be_a_block() {
    let body = body_of("void f() {\n run((Order a, var b) -> {\n return a.get(b);\n });\n}");

    let Statement::Expression(Expression::Call(call)) = &body[0] else {
        panic!("expected a call, got {body:?}");
    };
    assert_eq!(
        call.arguments,
        [Expression::Lambda(Lambda {
            parameters: vec![
                LambdaParameter {
                    name: "a".to_owned(),
                    declared_type: Some(DeclaredType::class("shop.Order"))
                },
                LambdaParameter {
                    name: "b".to_owned(),
                    declared_type: None
                },
            ],
            body: vec![Statement::Return(Some(call_on_lambda_parameter(
                "a", "get", 4
            )))],
        })]
    );
}

fn call_on_lambda_parameter(receiver: &str, name: &str, line: usize) -> Expression {
    call(
        Expression::LambdaParameter(receiver.to_owned()),
        name,
        vec![Expression::LambdaParameter("b".to_owned())],
        line,
    )
}

#[test]
fn a_method_reference_has_a_target_and_a_name() {
    let body = body_of(
        "private Wallet wallet;\nvoid f(List<Order> orders) {\n orders.stream().filter(Order::isPaid).map(wallet::open).map(this::make).map(Order::new);\n}",
    );

    let Statement::Expression(Expression::Call(outermost)) = &body[0] else {
        panic!("expected a call, got {body:?}");
    };
    let reference = |target: Expression, name: &str| Expression::MethodReference {
        target: Box::new(target),
        name: name.to_owned(),
        line: 4,
    };
    assert_eq!(
        outermost.arguments,
        [reference(type_ref("shop.Order"), "new")]
    );
    let Expression::Call(third) = outermost.receiver.as_deref().unwrap() else {
        panic!("expected a call");
    };
    assert_eq!(third.arguments, [reference(Expression::This, "make")]);
    let Expression::Call(second) = third.receiver.as_deref().unwrap() else {
        panic!("expected a call");
    };
    assert_eq!(second.arguments, [reference(field("wallet"), "open")]);
    let Expression::Call(first) = second.receiver.as_deref().unwrap() else {
        panic!("expected a call");
    };
    assert_eq!(
        first.arguments,
        [reference(type_ref("shop.Order"), "isPaid")]
    );
}

#[test]
fn a_method_reference_keeps_the_line_it_is_on() {
    let body = body_of(
        "void f(List<Order> orders) {\n orders.stream()\n  .map(Order::isPaid)\n  .forEach(this::ship);\n}",
    );

    let Statement::Expression(Expression::Call(for_each)) = &body[0] else {
        panic!("expected a call, got {body:?}");
    };
    let Expression::Call(map) = for_each.receiver.as_deref().unwrap() else {
        panic!("expected a call");
    };
    let reference = |target: Expression, name: &str, line: usize| Expression::MethodReference {
        target: Box::new(target),
        name: name.to_owned(),
        line,
    };
    assert_eq!(
        (&map.arguments, &for_each.arguments),
        (
            &vec![reference(type_ref("shop.Order"), "isPaid", 4)],
            &vec![reference(Expression::This, "ship", 5)]
        )
    );
}

#[test]
fn a_cast_keeps_its_target_type() {
    let body = body_of("void f(Object o) {\n ((Order) o).ship();\n}");

    assert_eq!(
        body,
        [Statement::Expression(call(
            Expression::Cast {
                target: DeclaredType::class("shop.Order"),
                operand: Box::new(parameter("o"))
            },
            "ship",
            vec![],
            3
        ))]
    );
}

#[test]
fn operators_keep_only_the_operands_that_lead_to_calls() {
    let body = body_of(
        "boolean f(Order a, Order b) {\n return a.x() == Status.PAID ? b.y() : !a.z() && b.w() > 1;\n}",
    );

    let Statement::Return(Some(Expression::Operation(operands))) = &body[0] else {
        panic!("expected an operation, got {body:?}");
    };
    let calls: Vec<String> = flatten(operands);
    assert_eq!(calls, ["x", "y", "z", "w"]);
}

fn flatten(operands: &[Expression]) -> Vec<String> {
    operands
        .iter()
        .flat_map(|operand| match operand {
            Expression::Operation(inner) => flatten(inner),
            Expression::Call(call) => vec![call.name.clone()],
            _ => vec![],
        })
        .collect()
}

#[test]
fn an_assignment_keeps_both_sides() {
    let body = body_of("private int count;\nvoid f(Order a) {\n this.count = a.size();\n}");

    assert_eq!(
        body,
        [Statement::Expression(Expression::Assignment {
            target: Box::new(field("count")),
            value: Box::new(call(parameter("a"), "size", vec![], 4)),
        })]
    );
}

#[test]
fn an_instanceof_pattern_binds_a_local() {
    let body =
        body_of("void f(Object o) {\n if (o instanceof Order x && x.ok()) {\n x.go();\n }\n}");

    let Statement::If { then_branch, .. } = &body[0] else {
        panic!("expected an if, got {body:?}");
    };
    assert_eq!(
        then_branch,
        &[Statement::Expression(call(local("x"), "go", vec![], 4))]
    );
}

#[test]
fn literals_text_blocks_and_array_creation_are_kept_minimal() {
    let body = body_of(
        "void f(Order a) {\n var t = \"\"\"\n hi\n \"\"\";\n var n = new int[a.size()];\n var m = a.id() + 1;\n}",
    );

    assert_eq!(
        initializer_of(&body),
        Expression::Literal(LiteralType::String)
    );
    let Statement::Local(array) = &body[1] else {
        panic!("expected a local");
    };
    assert_eq!(
        array.initializer,
        Some(Expression::Operation(vec![Expression::Operation(vec![
            call(parameter("a"), "size", vec![], 6)
        ])]))
    );
}

#[test]
fn calls_on_unresolved_and_own_names_are_kept_in_order() {
    let body = body_of("void f() {\n run(helper());\n}");

    assert_eq!(
        body,
        [Statement::Expression(own_call(
            "run",
            vec![own_call("helper", vec![], 3)],
            3
        ))]
    );
}

#[test]
fn a_class_literal_keeps_the_type_it_names() {
    let body = body_of("void f() {\n var a = Order.class;\n var b = Order[].class;\n}");

    let Statement::Local(first) = &body[0] else {
        panic!("expected a local variable, got {body:?}");
    };
    assert_eq!(
        first.initializer,
        Some(Expression::ClassLiteral(DeclaredType::class("shop.Order")))
    );
    assert_eq!(
        initializer_of(&body[1..]),
        Expression::ClassLiteral(DeclaredType::array(DeclaredType::class("shop.Order")))
    );
}

#[test]
fn an_expression_body_returns_its_value_while_a_block_keeps_its_statements() {
    let body = body_of(
        "void f(Order order) {\n run(x -> order.get(x));\n run(x -> { order.put(x); });\n}",
    );

    let bodies: Vec<Vec<Statement>> = body
        .iter()
        .map(|statement| match statement {
            Statement::Expression(Expression::Call(call)) => match &call.arguments[0] {
                Expression::Lambda(lambda) => lambda.body.clone(),
                other => panic!("expected a lambda, got {other:?}"),
            },
            other => panic!("expected a call, got {other:?}"),
        })
        .collect();
    let on_x = |name: &str, line| {
        call(
            parameter("order"),
            name,
            vec![Expression::LambdaParameter("x".to_owned())],
            line,
        )
    };
    assert_eq!(
        bodies,
        [
            vec![Statement::Return(Some(on_x("get", 3)))],
            vec![Statement::Expression(on_x("put", 4))],
        ]
    );
}
