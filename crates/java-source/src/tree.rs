use tree_sitter::{Node, Parser, Tree};

use crate::error::ParseError;

pub(crate) fn parse_tree(path: &str, source: &str) -> Result<Tree, ParseError> {
    let failure = |line: usize, message: &str| ParseError {
        file: path.to_owned(),
        line,
        message: message.to_owned(),
    };
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_java::LANGUAGE.into())
        .map_err(|error| failure(1, &format!("cannot load the Java grammar: {error}")))?;
    let tree = parser
        .parse(source, None)
        .ok_or_else(|| failure(1, "the parser gave no result"))?;
    match first_error(tree.root_node()) {
        Some(error) => Err(failure(
            line_of(error),
            if error.is_missing() {
                "missing token"
            } else {
                "syntax error"
            },
        )),
        None => Ok(tree),
    }
}

pub(crate) fn line_of(node: Node) -> usize {
    node.start_position().row + 1
}

fn first_error(node: Node) -> Option<Node> {
    if !node.has_error() && !node.is_missing() {
        return None;
    }
    if node.is_error() || node.is_missing() {
        return Some(node);
    }
    let mut cursor = node.walk();
    node.children(&mut cursor).find_map(first_error)
}
