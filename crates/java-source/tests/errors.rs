use law_of_demeter_java_source::parse;

#[test]
fn a_syntax_error_reports_the_file_and_the_line() {
    let source = "class A {\n  void f() {\n    int x = ;\n  }\n}";

    let error = parse("shop/Broken.java", source).unwrap_err();

    assert_eq!(error.file, "shop/Broken.java");
    assert_eq!(error.line, 3);
}

#[test]
fn an_unclosed_class_reports_a_line_without_panicking() {
    let source = "class A {\n  void f() {\n";

    let error = parse("Open.java", source).unwrap_err();

    assert_eq!(error.file, "Open.java");
    assert!(error.line >= 2);
}

#[test]
fn an_empty_file_has_no_classes() {
    let file = parse("Empty.java", "").unwrap();

    assert!(file.classes.is_empty());
}
