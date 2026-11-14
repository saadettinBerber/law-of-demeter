use law_of_demeter_java_source::{Import, parse};

#[test]
fn the_package_and_imports_are_read() {
    let source = "package shop.orders;\n\
        import shop.customers.Customer;\n\
        import java.util.*;\n\
        import static java.lang.Math.max;\n\
        class A {}";

    let file = parse("A.java", source).unwrap();

    assert_eq!(file.path, "A.java");
    assert_eq!(file.package.as_deref(), Some("shop.orders"));
    assert_eq!(
        file.imports,
        [
            Import::single("shop.customers.Customer"),
            Import::wildcard("java.util"),
            Import::static_single("java.lang.Math.max"),
        ]
    );
}

#[test]
fn a_file_without_package_is_in_the_default_package() {
    let file = parse("A.java", "class A {}").unwrap();

    assert_eq!(file.package, None);
}
