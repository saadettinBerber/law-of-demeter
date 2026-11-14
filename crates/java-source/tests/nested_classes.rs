mod support;

use law_of_demeter_java_source::{
    Call, Class, ClassKind, ClassOrigin, DeclaredType, Expression, ImplicitTarget, LocalVariable,
    Method, New, STATIC_INITIALIZER, Statement, parse_with,
};
use support::{KNOWN_TYPES, call, field, parameter, type_ref, unresolved};

fn classes_of(source: &str) -> Vec<Class> {
    let source = format!("package shop; {source}");
    parse_with("A.java", &source, &|name| KNOWN_TYPES.contains(&name))
        .unwrap()
        .classes
}

fn names(classes: &[Class]) -> Vec<&str> {
    classes.iter().map(|class| class.name.as_str()).collect()
}

fn class_named<'a>(classes: &'a [Class], name: &str) -> &'a Class {
    classes
        .iter()
        .find(|class| class.name == name)
        .unwrap_or_else(|| panic!("no class {name} in {:?}", names(classes)))
}

fn method_named<'a>(class: &'a Class, name: &str) -> &'a Method {
    class
        .methods
        .iter()
        .find(|method| method.name == name)
        .unwrap_or_else(|| panic!("no method {name} in {:?}", class.methods))
}

fn body(class: &Class, method: &str) -> Vec<Statement> {
    method_named(class, method).body.clone().unwrap()
}

fn run(expression: Expression) -> Statement {
    Statement::Expression(expression)
}

fn captured(name: &str) -> Expression {
    Expression::Captured(name.to_owned())
}

fn outer_call(owner: &str, name: &str, line: usize) -> Expression {
    Expression::Call(Call {
        receiver: None,
        implicit_target: Some(ImplicitTarget::OuterMethod {
            owner: owner.to_owned(),
        }),
        name: name.to_owned(),
        arguments: vec![],
        line,
    })
}

#[test]
fn an_anonymous_class_is_a_class_of_its_own_and_the_new_expression_points_to_it() {
    let source = "class Outer {\n void f(Order order) {\n  Runnable r = new Runnable() {\n   public void run() {\n    order.go();\n    helper();\n   }\n  };\n }\n void helper() {}\n}";

    let classes = classes_of(source);

    assert_eq!(names(&classes), ["shop.Outer", "shop.Outer$1"]);
    let anonymous = &classes[1];
    assert_eq!(anonymous.origin, ClassOrigin::Anonymous);
    assert_eq!(anonymous.kind, ClassKind::Class);
    assert_eq!(anonymous.outer.as_deref(), Some("shop.Outer"));
    assert_eq!(anonymous.supertypes, [DeclaredType::unresolved("Runnable")]);
    assert_eq!(
        body(anonymous, "run"),
        [
            run(call(captured("order"), "go", vec![], 5)),
            run(outer_call("shop.Outer", "helper", 6)),
        ]
    );
    assert_eq!(classes[0].origin, ClassOrigin::Member);
    assert_eq!(
        body(&classes[0], "f"),
        [Statement::Local(LocalVariable {
            name: "r".to_owned(),
            declared_type: DeclaredType::unresolved("Runnable"),
            initializer: Some(Expression::New(New {
                created: DeclaredType::unresolved("Runnable"),
                arguments: vec![],
                anonymous_class: Some("shop.Outer$1".to_owned()),
                line: 3,
            })),
            line: 3,
        })]
    );
}

#[test]
fn anonymous_classes_are_numbered_per_enclosing_class_in_source_order() {
    let source = "class Outer {\n Object a = new Object() { Object b = new Object() {}; };\n Object c = new Object() {};\n class Inner { Object d = new Object() {}; }\n void f() { new Object() {}; }\n}";

    let classes = classes_of(source);

    assert_eq!(
        names(&classes),
        [
            "shop.Outer",
            "shop.Outer$1",
            "shop.Outer$1$1",
            "shop.Outer$2",
            "shop.Outer$Inner",
            "shop.Outer$Inner$1",
            "shop.Outer$3",
        ]
    );
    assert_eq!(
        class_named(&classes, "shop.Outer$1$1").outer.as_deref(),
        Some("shop.Outer$1")
    );
    assert_eq!(
        class_named(&classes, "shop.Outer$Inner$1").outer.as_deref(),
        Some("shop.Outer$Inner")
    );
}

#[test]
fn an_anonymous_class_in_the_arguments_is_numbered_before_the_one_that_takes_it() {
    let source = "class Outer {\n Object o = new Base(new Object() {}) {};\n}";

    let classes = classes_of(source);

    assert_eq!(
        names(&classes),
        ["shop.Outer", "shop.Outer$1", "shop.Outer$2"]
    );
    assert_eq!(classes[1].supertypes, [DeclaredType::unresolved("Object")]);
    assert_eq!(classes[2].supertypes, [DeclaredType::unresolved("Base")]);
}

#[test]
fn local_classes_are_numbered_per_enclosing_class_and_simple_name() {
    let source = "class Outer {\n void f() {\n  class Local { void m() { ping(); } }\n  class Helper {}\n  new Local();\n }\n void ping() {}\n void g() {\n  class Local {}\n  record Pair(int a) {}\n  enum Color { RED }\n  interface Shape {}\n }\n}";

    let classes = classes_of(source);

    assert_eq!(
        names(&classes),
        [
            "shop.Outer",
            "shop.Outer$1Local",
            "shop.Outer$1Helper",
            "shop.Outer$2Local",
            "shop.Outer$1Pair",
            "shop.Outer$1Color",
            "shop.Outer$1Shape",
        ]
    );
    let kinds: Vec<(ClassKind, ClassOrigin)> = classes[1..]
        .iter()
        .map(|class| (class.kind, class.origin))
        .collect();
    assert_eq!(
        kinds,
        [
            (ClassKind::Class, ClassOrigin::Local),
            (ClassKind::Class, ClassOrigin::Local),
            (ClassKind::Class, ClassOrigin::Local),
            (ClassKind::Record, ClassOrigin::Local),
            (ClassKind::Enum, ClassOrigin::Local),
            (ClassKind::Interface, ClassOrigin::Local),
        ]
    );
    assert_eq!(classes[1].outer.as_deref(), Some("shop.Outer"));
    assert_eq!(classes[4].fields[0].name, "a");
    assert_eq!(
        body(&classes[1], "m"),
        [run(outer_call("shop.Outer", "ping", 3))]
    );
}

#[test]
fn an_anonymous_class_and_a_local_class_are_counted_apart() {
    let classes =
        classes_of("class Outer {\n void f() {\n  new Object() {};\n  class Local {}\n }\n}");

    assert_eq!(
        names(&classes),
        ["shop.Outer", "shop.Outer$1", "shop.Outer$1Local"]
    );
}

#[test]
fn a_local_class_name_is_a_type_in_the_rest_of_the_method() {
    let source = "class Outer {\n void f() {\n  class Local { static Local make() { return null; } }\n  Local x = new Local();\n  Local.make();\n }\n}";

    let classes = classes_of(source);

    let statements = body(&classes[0], "f");
    let local_type = DeclaredType::class("shop.Outer$1Local");
    assert_eq!(
        statements,
        [
            Statement::Local(LocalVariable {
                name: "x".to_owned(),
                declared_type: local_type.clone(),
                initializer: Some(Expression::New(New {
                    created: local_type.clone(),
                    arguments: vec![],
                    anonymous_class: None,
                    line: 4,
                })),
                line: 4,
            }),
            run(call(type_ref("shop.Outer$1Local"), "make", vec![], 5)),
        ]
    );
    assert_eq!(method_named(&classes[1], "make").returns, Some(local_type));
}

#[test]
fn captured_variables_are_marked_and_what_the_class_declares_hides_them() {
    let source = "class Outer {\n private Wallet wallet;\n void f(Order order, Customer customer) {\n  new Object() {\n   private Customer customer;\n   void run(Order order) {\n    order.a();\n    customer.b();\n    wallet.c();\n   }\n   void other() {\n    order.d();\n   }\n  };\n }\n}";

    let classes = classes_of(source);

    let anonymous = &classes[1];
    assert_eq!(
        body(anonymous, "run"),
        [
            run(call(parameter("order"), "a", vec![], 7)),
            run(call(field("customer"), "b", vec![], 8)),
            run(call(
                Expression::OuterField {
                    owner: "shop.Outer".to_owned(),
                    name: "wallet".to_owned()
                },
                "c",
                vec![],
                9
            )),
        ]
    );
    assert_eq!(
        body(anonymous, "other"),
        [run(call(captured("order"), "d", vec![], 12))]
    );
}

#[test]
fn locals_of_loops_and_blocks_are_captured_but_only_while_they_are_in_scope() {
    let source = "class Outer {\n void f(List orders) {\n  for (Order o : orders) {\n   new Object() { void m() { o.go(); } };\n  }\n  new Object() { void m() { o.go(); } };\n }\n}";

    let classes = classes_of(source);

    assert_eq!(
        body(&classes[1], "m"),
        [run(call(captured("o"), "go", vec![], 4))]
    );
    assert_eq!(
        body(&classes[2], "m"),
        [run(call(unresolved("o"), "go", vec![], 6))]
    );
}

#[test]
fn a_local_class_captures_the_variables_and_local_types_declared_before_it() {
    let source = "class Outer {\n void f(Order order) {\n  class First {}\n  class Second {\n   First first;\n   void m() { order.go(); }\n  }\n }\n}";

    let classes = classes_of(source);

    let second = class_named(&classes, "shop.Outer$1Second");
    assert_eq!(
        second.fields[0].declared_type,
        DeclaredType::class("shop.Outer$1First")
    );
    assert_eq!(
        body(second, "m"),
        [run(call(captured("order"), "go", vec![], 6))]
    );
}

#[test]
fn the_type_parameters_of_the_enclosing_method_are_visible_in_the_class() {
    let source = "class Outer {\n <T> void f() {\n  new Box<T>() {\n   T item;\n  };\n }\n}";

    let classes = classes_of(source);

    assert_eq!(
        classes[1].fields[0].declared_type,
        DeclaredType::variable("T")
    );
    assert_eq!(
        classes[1].supertypes,
        [DeclaredType::Class {
            name: law_of_demeter_java_source::TypeName::Unresolved("Box".to_owned()),
            arguments: vec![DeclaredType::variable("T")]
        }]
    );
}

#[test]
fn enum_constant_bodies_are_anonymous_subclasses_of_the_enum() {
    let source = "enum Color {\n RED { void paint() { helper.go(); } },\n GREEN(1) { },\n BLUE;\n Color() {}\n Color(int x) {}\n}";

    let classes = classes_of(source);

    assert_eq!(
        names(&classes),
        ["shop.Color", "shop.Color$1", "shop.Color$2"]
    );
    let red = &classes[1];
    assert_eq!(red.origin, ClassOrigin::Anonymous);
    assert_eq!(red.outer.as_deref(), Some("shop.Color"));
    assert_eq!(red.supertypes, [DeclaredType::class("shop.Color")]);
    assert_eq!(
        body(red, "paint"),
        [run(call(unresolved("helper"), "go", vec![], 2))]
    );
    let constants = body(&classes[0], STATIC_INITIALIZER);
    let initialized = |name: &str, arguments: Vec<Expression>, anonymous: &str, line: usize| {
        run(Expression::Operation(vec![
            field(name),
            Expression::New(New {
                created: DeclaredType::class("shop.Color"),
                arguments,
                anonymous_class: Some(anonymous.to_owned()),
                line,
            }),
        ]))
    };
    assert_eq!(
        constants,
        [
            initialized("RED", vec![], "shop.Color$1", 2),
            initialized("GREEN", vec![Expression::Literal], "shop.Color$2", 3),
        ]
    );
}

#[test]
fn anonymous_classes_in_initializers_and_lambdas_are_read_too() {
    let source = "class Outer {\n Object a = new Object() { void m() { x.go(); } };\n static { run(() -> new Object() { void n() { y.go(); } }); }\n}";

    let classes = classes_of(source);

    assert_eq!(
        names(&classes),
        ["shop.Outer", "shop.Outer$1", "shop.Outer$2"]
    );
    assert_eq!(
        body(&classes[1], "m"),
        [run(call(unresolved("x"), "go", vec![], 2))]
    );
    assert_eq!(
        body(&classes[2], "n"),
        [run(call(unresolved("y"), "go", vec![], 3))]
    );
}

#[test]
fn an_anonymous_class_inside_an_anonymous_class_sees_both_enclosing_methods() {
    let source = "class Outer {\n void f(Order order) {\n  new Object() {\n   void m(Customer customer) {\n    new Object() {\n     void n() { order.a(); customer.b(); }\n    };\n   }\n  };\n }\n}";

    let classes = classes_of(source);

    assert_eq!(
        names(&classes),
        ["shop.Outer", "shop.Outer$1", "shop.Outer$1$1"]
    );
    assert_eq!(
        body(&classes[2], "n"),
        [
            run(call(captured("order"), "a", vec![], 6)),
            run(call(captured("customer"), "b", vec![], 6)),
        ]
    );
}

#[test]
fn an_unqualified_call_to_a_method_the_anonymous_class_declares_is_its_own_method() {
    let source = "class Outer {\n Object o = new Object() {\n  void a() { b(); c(); }\n  void b() {}\n };\n void c() {}\n}";

    let classes = classes_of(source);

    let own = Expression::Call(Call {
        receiver: None,
        implicit_target: Some(ImplicitTarget::OwnMethod),
        name: "b".to_owned(),
        arguments: vec![],
        line: 3,
    });
    assert_eq!(
        body(&classes[1], "a"),
        [run(own), run(outer_call("shop.Outer", "c", 3))]
    );
}

#[test]
fn a_lambda_parameter_is_captured_by_a_class_declared_inside_the_lambda() {
    let source = "class Outer {\n void f(Order order) {\n  order.each(item -> new Object() {\n   void m() { item.go(); }\n  });\n }\n}";

    let classes = classes_of(source);

    assert_eq!(
        body(&classes[1], "m"),
        [run(call(captured("item"), "go", vec![], 4))]
    );
}
