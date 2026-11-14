use tree_sitter::{Node, Parser, Tree};

fn parse(source: &str) -> Tree {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_java::LANGUAGE.into())
        .unwrap();
    parser.parse(source, None).unwrap()
}

fn first_of_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    if node.kind() == kind {
        return Some(node);
    }
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find_map(|child| first_of_kind(child, kind))
}

fn text<'a>(node: Node, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}

fn field_text<'a>(node: Node, field: &str, source: &'a str) -> &'a str {
    text(node.child_by_field_name(field).unwrap(), source)
}

#[test]
fn a_record_parses_with_its_components() {
    let source = "record Server(String host, int port) {}";

    let tree = parse(source);

    let record = first_of_kind(tree.root_node(), "record_declaration").unwrap();
    assert!(!tree.root_node().has_error());
    assert_eq!(field_text(record, "name", source), "Server");
    assert_eq!(
        field_text(record, "parameters", source),
        "(String host, int port)"
    );
}

#[test]
fn generic_types_keep_their_arguments() {
    let source = "class A { Map<String, List<Order>> index; }";

    let tree = parse(source);

    let field = first_of_kind(tree.root_node(), "field_declaration").unwrap();
    let declared = field.child_by_field_name("type").unwrap();
    assert!(!tree.root_node().has_error());
    assert_eq!(declared.kind(), "generic_type");
    assert_eq!(text(declared, source), "Map<String, List<Order>>");
}

#[test]
fn a_lambda_exposes_its_parameters_and_body() {
    let source = "class A { void f() { run((a, b) -> a.go(b)); } }";

    let tree = parse(source);

    let lambda = first_of_kind(tree.root_node(), "lambda_expression").unwrap();
    assert!(!tree.root_node().has_error());
    assert_eq!(field_text(lambda, "parameters", source), "(a, b)");
    assert_eq!(field_text(lambda, "body", source), "a.go(b)");
}

#[test]
fn a_method_reference_is_its_own_node() {
    let source = "class A { void f() { orders.stream().filter(Order::isPaid); } }";

    let tree = parse(source);

    let reference = first_of_kind(tree.root_node(), "method_reference").unwrap();
    assert!(!tree.root_node().has_error());
    assert_eq!(text(reference, source), "Order::isPaid");
    assert_eq!(
        reference.named_child(0).map(|n| text(n, source)),
        Some("Order")
    );
    assert_eq!(
        reference.named_child(1).map(|n| text(n, source)),
        Some("isPaid")
    );
}

#[test]
fn a_text_block_is_one_string_literal() {
    let source = "class A { String t = \"\"\"\n    hello\n    \"\"\"; }";

    let tree = parse(source);

    let literal = first_of_kind(tree.root_node(), "string_literal").unwrap();
    assert!(!tree.root_node().has_error());
    assert!(first_of_kind(literal, "multiline_string_fragment").is_some());
}

#[test]
fn a_method_call_exposes_receiver_name_and_arguments() {
    let source = "class A { void f() { order.getCustomer().find(id, 3); } }";

    let tree = parse(source);

    let outer = first_of_kind(tree.root_node(), "method_invocation").unwrap();
    assert_eq!(field_text(outer, "name", source), "find");
    assert_eq!(field_text(outer, "arguments", source), "(id, 3)");
    assert_eq!(field_text(outer, "object", source), "order.getCustomer()");
}

#[test]
fn a_call_without_receiver_has_no_object() {
    let source = "class A { void f() { audit(); } }";

    let tree = parse(source);

    let call = first_of_kind(tree.root_node(), "method_invocation").unwrap();
    assert!(call.child_by_field_name("object").is_none());
}

#[test]
fn a_type_name_receiver_looks_like_a_plain_identifier() {
    let source = "class A { void f() { Order.builder(); } }";

    let tree = parse(source);

    let call = first_of_kind(tree.root_node(), "method_invocation").unwrap();
    assert_eq!(
        call.child_by_field_name("object").unwrap().kind(),
        "identifier"
    );
}

#[test]
fn a_syntax_error_is_flagged_and_located() {
    let source = "class A {\n  void f() {\n    int x = ;\n  }\n}";

    let tree = parse(source);

    let error = first_of_kind(tree.root_node(), "ERROR")
        .or_else(|| {
            let mut stack = vec![tree.root_node()];
            while let Some(node) = stack.pop() {
                if node.is_missing() {
                    return Some(node);
                }
                let mut cursor = node.walk();
                stack.extend(node.children(&mut cursor));
            }
            None
        })
        .unwrap();
    assert!(tree.root_node().has_error());
    assert_eq!(error.start_position().row, 2);
}
