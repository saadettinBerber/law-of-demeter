use law_of_demeter_java_source::{
    Class, DeclaredType, Method, Parameter, TypeName, TypeParameter, parse,
};

fn first_class(source: &str) -> Class {
    parse("A.java", source).unwrap().classes.remove(0)
}

fn parameter(name: &str, declared_type: DeclaredType) -> Parameter {
    Parameter {
        name: name.to_owned(),
        declared_type,
    }
}

fn method_named<'a>(class: &'a Class, name: &str) -> &'a Method {
    class
        .methods
        .iter()
        .find(|method| method.name == name)
        .unwrap()
}

#[test]
fn a_method_keeps_its_name_parameters_return_type_and_line() {
    let source = "package shop;\n\
        import java.util.List;\n\
        class OrderService {\n\
            static List<Order> pay(Order order, int amount) { return null; }\n\
        }";

    let class = first_class(source);

    let pay = method_named(&class, "pay");
    assert_eq!(pay.line, 4);
    assert!(pay.is_static);
    assert!(!pay.is_constructor());
    assert_eq!(
        pay.parameters,
        [
            parameter("order", DeclaredType::unresolved("Order")),
            parameter("amount", DeclaredType::primitive("int")),
        ]
    );
    assert_eq!(
        pay.returns,
        Some(DeclaredType::generic(
            "java.util.List",
            vec![DeclaredType::unresolved("Order")]
        ))
    );
}

#[test]
fn the_line_of_a_method_is_the_line_of_its_name() {
    let source = "class A {\n  @Override\n  @Deprecated\n  void f() {}\n}";

    let class = first_class(source);

    assert_eq!(method_named(&class, "f").line, 4);
}

#[test]
fn a_constructor_has_no_return_type() {
    let class = first_class("class Order { Order(int id) { this.id = id; } int id; }");

    let constructor = method_named(&class, "Order");
    assert!(constructor.is_constructor());
    assert_eq!(
        constructor.parameters,
        [parameter("id", DeclaredType::primitive("int"))]
    );
}

#[test]
fn varargs_are_an_array_parameter() {
    let class = first_class("class A { void f(String... names) {} }");

    assert_eq!(
        method_named(&class, "f").parameters,
        [parameter(
            "names",
            DeclaredType::array(DeclaredType::unresolved("String"))
        )]
    );
}

#[test]
fn a_method_type_parameter_is_a_type_variable() {
    let class = first_class("class A { <T> T first(java.util.List<T> items) { return null; } }");

    let first = method_named(&class, "first");
    assert_eq!(first.returns, Some(DeclaredType::variable("T")));
    assert_eq!(
        first.parameters,
        [parameter(
            "items",
            DeclaredType::generic("java.util.List", vec![DeclaredType::variable("T")])
        )]
    );
}

#[test]
fn a_method_without_a_body_has_none() {
    let class = first_class("interface Shape { void draw(); default void erase() {} }");

    assert_eq!(method_named(&class, "draw").body, None);
    assert_eq!(method_named(&class, "erase").body, Some(Vec::new()));
}

#[test]
fn a_compact_record_constructor_takes_the_record_components() {
    let class = first_class("record Server(String host, int port) { Server { check(); } }");

    let constructor = method_named(&class, "Server");
    assert!(constructor.is_constructor());
    assert_eq!(
        constructor.parameters,
        [
            parameter("host", DeclaredType::unresolved("String")),
            parameter("port", DeclaredType::primitive("int")),
        ]
    );
}

#[test]
fn methods_of_enums_and_nested_classes_belong_to_their_own_class() {
    let source = "enum Color { RED; void paint() {} }\nclass Outer { void a() {} class Inner { void b() {} } }";

    let classes = parse("A.java", source).unwrap().classes;

    let methods: Vec<(&str, Vec<&str>)> = classes
        .iter()
        .map(|class| {
            (
                class.name.as_str(),
                class.methods.iter().map(|m| m.name.as_str()).collect(),
            )
        })
        .collect();
    assert_eq!(
        methods,
        [
            ("Color", vec!["paint"]),
            ("Outer", vec!["a"]),
            ("Outer$Inner", vec!["b"])
        ]
    );
}

#[test]
fn a_method_keeps_the_names_and_first_bounds_of_its_own_type_parameters() {
    let source = "class OrderService<C extends Order> {\n <T extends Order & Customer, U, V extends Comparable<V>> void f(T item, U other, V third) {}\n void g(C item) {}\n}";

    let class = first_class(source);

    assert_eq!(
        method_named(&class, "f").type_parameters,
        [
            TypeParameter {
                name: "T".to_owned(),
                bound: Some(DeclaredType::unresolved("Order"))
            },
            TypeParameter {
                name: "U".to_owned(),
                bound: None
            },
            TypeParameter {
                name: "V".to_owned(),
                bound: Some(DeclaredType::Class {
                    name: TypeName::Unresolved("Comparable".to_owned()),
                    arguments: vec![DeclaredType::variable("V")]
                })
            },
        ]
    );
    assert_eq!(method_named(&class, "g").type_parameters, []);
}
