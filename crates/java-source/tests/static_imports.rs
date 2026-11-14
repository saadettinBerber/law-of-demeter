mod support;

use law_of_demeter_java_source::{Expression, ImplicitTarget, Statement, TypeName};
use support::{method_in, parameter};

fn implicit_targets(imports: &str, members: &str) -> Vec<Option<ImplicitTarget>> {
    let body = method_in(imports, "", members, "f").body.unwrap();
    body.into_iter()
        .map(|statement| match statement {
            Statement::Expression(Expression::Call(call)) => call.implicit_target,
            other => panic!("expected a call, got {other:?}"),
        })
        .collect()
}

fn resolved(name: &str) -> TypeName {
    TypeName::Resolved(name.to_owned())
}

#[test]
fn a_call_without_receiver_to_a_declared_method_is_an_own_method() {
    let targets = implicit_targets("", "void f() {\n audit();\n}\nvoid audit() {}");

    assert_eq!(targets, [Some(ImplicitTarget::OwnMethod)]);
}

#[test]
fn a_call_with_a_receiver_has_no_implicit_target() {
    let targets = implicit_targets("", "void f(Order order) {\n order.audit();\n}");

    assert_eq!(targets, [None]);
}

#[test]
fn a_single_static_import_names_the_type_that_provides_the_method() {
    let targets = implicit_targets(
        "import static java.lang.Math.max;",
        "void f() {\n max(1, 2);\n}",
    );

    assert_eq!(
        targets,
        [Some(ImplicitTarget::StaticImport {
            owner: resolved("java.lang.Math")
        })]
    );
}

#[test]
fn a_static_import_of_a_nested_type_uses_a_dollar() {
    let targets = implicit_targets(
        "import static shop.Order.Line.make;",
        "void f() {\n make();\n}",
    );

    assert_eq!(
        targets,
        [Some(ImplicitTarget::StaticImport {
            owner: resolved("shop.Order$Line")
        })]
    );
}

#[test]
fn a_declared_method_hides_a_static_import_of_the_same_name() {
    let targets = implicit_targets(
        "import static java.lang.Math.max;",
        "void f() {\n max(1, 2);\n}\nint max(int a, int b) { return a; }",
    );

    assert_eq!(targets, [Some(ImplicitTarget::OwnMethod)]);
}

#[test]
fn on_demand_static_imports_list_every_candidate_type_in_order() {
    let targets = implicit_targets(
        "import static java.lang.Math.*; import static shop.Order.*;",
        "void f() {\n max(1, 2);\n}",
    );

    assert_eq!(
        targets,
        [Some(ImplicitTarget::OnDemandStaticImport {
            candidates: vec![resolved("java.lang.Math"), resolved("shop.Order")]
        })]
    );
}

#[test]
fn a_single_static_import_wins_over_on_demand_ones() {
    let targets = implicit_targets(
        "import static shop.Order.*; import static java.lang.Math.max;",
        "void f() {\n max(1, 2);\n}",
    );

    assert_eq!(
        targets,
        [Some(ImplicitTarget::StaticImport {
            owner: resolved("java.lang.Math")
        })]
    );
}

#[test]
fn a_method_of_an_enclosing_class_names_its_owner() {
    let members = "void audit() {}\nclass Inner {\n void f() {\n audit();\n }\n}";
    let source = format!("package shop; class OrderService {{\n{members}\n}}");

    let file = law_of_demeter_java_source::parse("A.java", &source).unwrap();

    let body = file.classes[1].methods[0].body.as_ref().unwrap();
    let Statement::Expression(Expression::Call(call)) = &body[0] else {
        panic!("expected a call");
    };
    assert_eq!(
        call.implicit_target,
        Some(ImplicitTarget::OuterMethod {
            owner: "shop.OrderService".to_owned()
        })
    );
}

#[test]
fn a_static_imported_name_used_as_a_value_is_a_static_import() {
    let body = method_in(
        "import static shop.Status.PAID;",
        "",
        "void f(Order order) {\n order.set(PAID);\n}",
        "f",
    )
    .body
    .unwrap();

    let Statement::Expression(Expression::Call(call)) = &body[0] else {
        panic!("expected a call");
    };
    assert_eq!(
        call.arguments,
        [Expression::StaticImport {
            owner: resolved("shop.Status"),
            name: "PAID".to_owned()
        }]
    );
    assert_eq!(call.receiver.as_deref(), Some(&parameter("order")));
}
