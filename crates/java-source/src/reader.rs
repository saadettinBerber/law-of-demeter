use std::collections::HashSet;

use tree_sitter::Node;

use crate::error::ParseError;
use crate::model::DeclaredType;
use crate::model::{Class, ClassKind, Import, SourceFile, TypeName};
use crate::resolver::TypeResolver;
use crate::scope::Scope;
use crate::tree::parse_tree;

pub(crate) fn read_file(
    path: &str,
    source: &str,
    type_exists: &dyn Fn(&str) -> bool,
) -> Result<SourceFile, ParseError> {
    let tree = parse_tree(path, source)?;
    let root = tree.root_node();
    let text = Text { source };
    let package = text.package(root);
    let imports = text.imports(root);
    let prefix = package
        .as_ref()
        .map_or_else(String::new, |name| format!("{name}."));
    let mut declarations = Vec::new();
    text.collect_declarations(root, &prefix, None, &[], &mut declarations);
    let declared = declarations
        .iter()
        .map(|declaration| declaration.name.clone())
        .collect::<HashSet<_>>();
    let resolver = TypeResolver::new(package.as_deref(), &imports, declared, type_exists);
    let reader = Reader { text, resolver };
    let classes = declarations
        .iter()
        .map(|declaration| reader.class(declaration))
        .collect();
    Ok(SourceFile {
        path: path.to_owned(),
        package,
        imports,
        classes,
    })
}

#[derive(Clone, Copy)]
pub(crate) struct Text<'a> {
    source: &'a str,
}

impl Text<'_> {
    pub(crate) fn of(&self, node: Node) -> &str {
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
            .map(|name| self.of(name).to_owned())
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
            name: self.of(*name).to_owned(),
            is_static: has("static"),
            is_wildcard: has("asterisk"),
        })
    }

    fn collect_declarations<'t>(
        &self,
        container: Node<'t>,
        prefix: &str,
        outer: Option<&str>,
        type_variables: &[String],
        declarations: &mut Vec<Declaration<'t>>,
    ) {
        let mut cursor = container.walk();
        for child in container.named_children(&mut cursor) {
            if child.kind() == "enum_body_declarations" {
                self.collect_declarations(child, prefix, outer, type_variables, declarations);
            } else if let Some(kind) = class_kind(child.kind()) {
                self.collect_declaration(child, kind, prefix, outer, type_variables, declarations);
            }
        }
    }

    fn collect_declaration<'t>(
        &self,
        node: Node<'t>,
        kind: ClassKind,
        prefix: &str,
        outer: Option<&str>,
        enclosing_type_variables: &[String],
        declarations: &mut Vec<Declaration<'t>>,
    ) {
        let Some(name) = node.child_by_field_name("name") else {
            return;
        };
        let binary_name = format!("{prefix}{}", self.of(name));
        let mut type_variables = enclosing_type_variables.to_vec();
        type_variables.extend(self.type_parameters(node));
        declarations.push(Declaration {
            node,
            kind,
            name: binary_name.clone(),
            outer: outer.map(str::to_owned),
            type_variables: type_variables.clone(),
        });
        if let Some(body) = node.child_by_field_name("body") {
            self.collect_declarations(
                body,
                &format!("{binary_name}$"),
                Some(&binary_name),
                &type_variables,
                declarations,
            );
        }
    }

    pub(crate) fn type_parameters(&self, declaration: Node) -> Vec<String> {
        let Some(parameters) = declaration.child_by_field_name("type_parameters") else {
            return Vec::new();
        };
        let mut cursor = parameters.walk();
        parameters
            .named_children(&mut cursor)
            .filter(|parameter| parameter.kind() == "type_parameter")
            .filter_map(|parameter| parameter.named_child(0))
            .map(|name| self.of(name).to_owned())
            .collect()
    }
}

pub(crate) struct Declaration<'t> {
    pub(crate) node: Node<'t>,
    pub(crate) kind: ClassKind,
    pub(crate) name: String,
    pub(crate) outer: Option<String>,
    pub(crate) type_variables: Vec<String>,
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

pub(crate) struct Reader<'a> {
    pub(crate) text: Text<'a>,
    pub(crate) resolver: TypeResolver<'a>,
}

impl Reader<'_> {
    fn class(&self, declaration: &Declaration) -> Class {
        let scope = Scope::new(&declaration.name, &declaration.type_variables);
        Class {
            name: declaration.name.clone(),
            kind: declaration.kind,
            outer: declaration.outer.clone(),
            annotations: self.annotations(declaration.node, &scope),
            supertypes: self.supertypes(declaration.node, &scope),
            fields: self.fields(declaration),
            methods: self.methods(declaration),
        }
    }

    fn annotations(&self, declaration: Node, scope: &Scope) -> Vec<TypeName> {
        let mut cursor = declaration.walk();
        let Some(modifiers) = declaration
            .children(&mut cursor)
            .find(|child| child.kind() == "modifiers")
        else {
            return Vec::new();
        };
        let mut cursor = modifiers.walk();
        modifiers
            .named_children(&mut cursor)
            .filter(|child| matches!(child.kind(), "marker_annotation" | "annotation"))
            .filter_map(|annotation| annotation.child_by_field_name("name"))
            .map(|name| self.resolver.resolve(self.text.of(name), &scope.class))
            .collect()
    }

    fn supertypes(&self, declaration: Node, scope: &Scope) -> Vec<DeclaredType> {
        let mut cursor = declaration.walk();
        let clauses: Vec<Node> = declaration
            .children(&mut cursor)
            .filter(|child| {
                matches!(
                    child.kind(),
                    "superclass" | "super_interfaces" | "extends_interfaces"
                )
            })
            .collect();
        clauses
            .into_iter()
            .flat_map(|clause| {
                let mut cursor = clause.walk();
                clause
                    .named_children(&mut cursor)
                    .flat_map(|child| {
                        if child.kind() == "type_list" {
                            let mut cursor = child.walk();
                            child.named_children(&mut cursor).collect()
                        } else {
                            vec![child]
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .map(|supertype| self.declared_type(supertype, scope))
            .collect()
    }
}
