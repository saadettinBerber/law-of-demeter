use tree_sitter::{Node, Parser, Tree};

use crate::error::ParseError;
use crate::model::{Class, ClassKind, Import, SourceFile};

pub(crate) fn read_file(path: &str, source: &str) -> Result<SourceFile, ParseError> {
    let tree = parse_tree(path, source)?;
    let root = tree.root_node();
    let reader = Reader { source };
    let package = reader.package(root);
    let prefix = package
        .as_ref()
        .map_or_else(String::new, |name| format!("{name}."));
    let mut classes = Vec::new();
    reader.collect_classes(root, &prefix, None, &mut classes);
    Ok(SourceFile {
        path: path.to_owned(),
        package,
        imports: reader.imports(root),
        classes,
    })
}

fn parse_tree(path: &str, source: &str) -> Result<Tree, ParseError> {
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
            error.start_position().row + 1,
            if error.is_missing() {
                "missing token"
            } else {
                "syntax error"
            },
        )),
        None => Ok(tree),
    }
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

struct Reader<'a> {
    source: &'a str,
}

impl Reader<'_> {
    fn text(&self, node: Node) -> &str {
        self.source.get(node.byte_range()).unwrap_or_default()
    }

    fn package(&self, root: Node) -> Option<String> {
        let mut cursor = root.walk();
        let declaration = root
            .children(&mut cursor)
            .find(|child| child.kind() == "package_declaration")?;
        let mut cursor = declaration.walk();
        declaration
            .named_children(&mut cursor)
            .find(|child| matches!(child.kind(), "scoped_identifier" | "identifier"))
            .map(|name| self.text(name).to_owned())
    }

    fn imports(&self, root: Node) -> Vec<Import> {
        let mut cursor = root.walk();
        root.children(&mut cursor)
            .filter(|child| child.kind() == "import_declaration")
            .filter_map(|declaration| self.import(declaration))
            .collect()
    }

    fn import(&self, declaration: Node) -> Option<Import> {
        let mut cursor = declaration.walk();
        let children: Vec<Node> = declaration.children(&mut cursor).collect();
        let name = children
            .iter()
            .find(|child| matches!(child.kind(), "scoped_identifier" | "identifier"))?;
        let has = |kind: &str| children.iter().any(|child| child.kind() == kind);
        Some(Import {
            name: self.text(*name).to_owned(),
            is_static: has("static"),
            is_wildcard: has("asterisk"),
        })
    }
}

fn class_kind(node_kind: &str) -> Option<ClassKind> {
    match node_kind {
        "class_declaration" => Some(ClassKind::Class),
        "interface_declaration" => Some(ClassKind::Interface),
        "enum_declaration" => Some(ClassKind::Enum),
        "record_declaration" => Some(ClassKind::Record),
        "annotation_type_declaration" => Some(ClassKind::Annotation),
        _ => None,
    }
}

impl Reader<'_> {
    fn collect_classes(
        &self,
        container: Node,
        prefix: &str,
        outer: Option<&str>,
        classes: &mut Vec<Class>,
    ) {
        let mut cursor = container.walk();
        for child in container.named_children(&mut cursor) {
            if child.kind() == "enum_body_declarations" {
                self.collect_classes(child, prefix, outer, classes);
            } else if let Some(kind) = class_kind(child.kind()) {
                self.collect_class(child, kind, prefix, outer, classes);
            }
        }
    }

    fn collect_class(
        &self,
        declaration: Node,
        kind: ClassKind,
        prefix: &str,
        outer: Option<&str>,
        classes: &mut Vec<Class>,
    ) {
        let Some(name) = declaration.child_by_field_name("name") else {
            return;
        };
        let binary_name = format!("{prefix}{}", self.text(name));
        classes.push(Class {
            name: binary_name.clone(),
            kind,
            outer: outer.map(str::to_owned),
        });
        if let Some(body) = declaration.child_by_field_name("body") {
            self.collect_classes(
                body,
                &format!("{binary_name}$"),
                Some(&binary_name),
                classes,
            );
        }
    }
}
