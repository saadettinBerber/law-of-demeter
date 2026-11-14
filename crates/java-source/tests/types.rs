use law_of_demeter_java_source::{DeclaredType, parse, parse_with};

fn type_of(header: &str, declared: &str, known: &[&str]) -> DeclaredType {
    let source = format!("{header}\nclass Order {{ {declared} value; }}");
    let file = parse_with("Order.java", &source, &|name| known.contains(&name)).unwrap();
    file.classes[0].fields[0].declared_type.clone()
}

#[test]
fn a_single_type_import_names_the_type() {
    let declared = type_of("import shop.customers.Customer;", "Customer", &[]);

    assert_eq!(declared, DeclaredType::class("shop.customers.Customer"));
}

#[test]
fn a_nested_type_of_an_imported_type_uses_a_dollar() {
    let declared = type_of("import java.util.Map;", "Map.Entry", &[]);

    assert_eq!(declared, DeclaredType::class("java.util.Map$Entry"));
}

#[test]
fn importing_a_nested_type_directly_uses_a_dollar() {
    let declared = type_of("import java.util.Map.Entry;", "Entry", &[]);

    assert_eq!(declared, DeclaredType::class("java.util.Map$Entry"));
}

#[test]
fn a_fully_qualified_name_is_resolved_as_written() {
    let declared = type_of("", "java.util.Map.Entry", &[]);

    assert_eq!(declared, DeclaredType::class("java.util.Map$Entry"));
}

#[test]
fn a_type_declared_in_the_same_file_is_resolved_without_a_catalog() {
    let source =
        "package shop;\nclass Order { Line line; class Line {} }\nclass Customer { Order order; }";

    let file = parse("Order.java", source).unwrap();

    assert_eq!(
        file.classes[0].fields[0].declared_type,
        DeclaredType::class("shop.Order$Line")
    );
    assert_eq!(
        file.classes[2].fields[0].declared_type,
        DeclaredType::class("shop.Order")
    );
}

#[test]
fn a_same_package_type_is_resolved_when_the_catalog_knows_it() {
    let declared = type_of("package shop;", "Customer", &["shop.Customer"]);

    assert_eq!(declared, DeclaredType::class("shop.Customer"));
}

#[test]
fn a_type_in_the_default_package_is_resolved_when_the_catalog_knows_it() {
    let declared = type_of("", "Customer", &["Customer"]);

    assert_eq!(declared, DeclaredType::class("Customer"));
}

#[test]
fn a_java_lang_type_is_resolved_when_the_catalog_knows_it() {
    let declared = type_of("package shop;", "String", &["java.lang.String"]);

    assert_eq!(declared, DeclaredType::class("java.lang.String"));
}

#[test]
fn a_wildcard_import_resolves_a_type_the_catalog_knows() {
    let declared = type_of("import java.util.*;", "List", &["java.util.List"]);

    assert_eq!(declared, DeclaredType::class("java.util.List"));
}

#[test]
fn a_type_known_in_two_wildcard_imports_stays_unresolved() {
    let declared = type_of(
        "import java.util.*;\nimport java.awt.*;",
        "List",
        &["java.util.List", "java.awt.List"],
    );

    assert_eq!(declared, DeclaredType::unresolved("List"));
}

#[test]
fn the_same_package_wins_over_a_wildcard_import() {
    let declared = type_of(
        "package shop;\nimport java.util.*;",
        "List",
        &["shop.List", "java.util.List"],
    );

    assert_eq!(declared, DeclaredType::class("shop.List"));
}

#[test]
fn a_single_type_import_wins_over_the_same_package() {
    let declared = type_of(
        "package shop;\nimport java.util.List;",
        "List",
        &["shop.List"],
    );

    assert_eq!(declared, DeclaredType::class("java.util.List"));
}

#[test]
fn a_type_nothing_knows_stays_unresolved() {
    let declared = type_of("package shop;\nimport java.util.*;", "Ghost", &[]);

    assert_eq!(declared, DeclaredType::unresolved("Ghost"));
}

#[test]
fn without_a_catalog_only_imports_and_the_file_resolve_types() {
    let file = parse(
        "Order.java",
        "import java.util.List;\nclass Order { List a; String b; }",
    )
    .unwrap();

    assert_eq!(
        file.classes[0].fields[0].declared_type,
        DeclaredType::class("java.util.List")
    );
    assert_eq!(
        file.classes[0].fields[1].declared_type,
        DeclaredType::unresolved("String")
    );
}

#[test]
fn generic_arguments_are_resolved_too() {
    let declared = type_of(
        "import java.util.*;\nimport shop.Line;",
        "Map<String, List<Line>>",
        &["java.util.Map", "java.util.List", "java.lang.String"],
    );

    assert_eq!(
        declared,
        DeclaredType::generic(
            "java.util.Map",
            vec![
                DeclaredType::class("java.lang.String"),
                DeclaredType::generic("java.util.List", vec![DeclaredType::class("shop.Line")]),
            ]
        )
    );
}

#[test]
fn a_diamond_has_no_arguments() {
    let declared = type_of("import java.util.List;", "List<>", &[]);

    assert_eq!(declared, DeclaredType::class("java.util.List"));
}

#[test]
fn primitives_and_arrays_keep_their_shape() {
    assert_eq!(type_of("", "int", &[]), DeclaredType::primitive("int"));
    assert_eq!(
        type_of("", "int[][]", &[]),
        DeclaredType::array(DeclaredType::array(DeclaredType::primitive("int")))
    );
}

#[test]
fn a_class_type_parameter_is_a_type_variable() {
    let file = parse("Box.java", "class Box<T> { T item; T[] items; }").unwrap();

    assert_eq!(
        file.classes[0].fields[0].declared_type,
        DeclaredType::variable("T")
    );
    assert_eq!(
        file.classes[0].fields[1].declared_type,
        DeclaredType::array(DeclaredType::variable("T"))
    );
}

#[test]
fn a_wildcard_keeps_only_its_upper_bound() {
    let declared = type_of(
        "import java.util.List;\nimport shop.Line;",
        "List<? extends Line>",
        &[],
    );

    assert_eq!(
        declared,
        DeclaredType::generic(
            "java.util.List",
            vec![DeclaredType::wildcard(Some(DeclaredType::class(
                "shop.Line"
            )))]
        )
    );
}

#[test]
fn an_unbounded_or_lower_bounded_wildcard_has_no_bound() {
    let declared = type_of(
        "import java.util.List;\nimport shop.Line;",
        "List<? super Line>",
        &[],
    );

    assert_eq!(
        declared,
        DeclaredType::generic("java.util.List", vec![DeclaredType::wildcard(None)])
    );
}
