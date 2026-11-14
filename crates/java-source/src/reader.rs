use std::collections::HashSet;

use tree_sitter::Node;

use crate::error::ParseError;
use crate::model::{Class, ClassKind, DeclaredType, Field, Import, SourceFile};
use crate::resolver::TypeResolver;
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
struct Text<'a> {
    source: &'a str,
}

impl Text<'_> {
    fn of(&self, node: Node) -> &str {
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

    fn type_parameters(&self, declaration: Node) -> Vec<String> {
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

struct Declaration<'t> {
    node: Node<'t>,
    kind: ClassKind,
    name: String,
    outer: Option<String>,
    type_variables: Vec<String>,
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

struct Reader<'a> {
    text: Text<'a>,
    resolver: TypeResolver<'a>,
}

impl Reader<'_> {
    fn class(&self, declaration: &Declaration) -> Class {
        Class {
            name: declaration.name.clone(),
            kind: declaration.kind,
            outer: declaration.outer.clone(),
            fields: self.fields(declaration),
        }
    }

    fn fields(&self, declaration: &Declaration) -> Vec<Field> {
        let mut fields = self.record_components(declaration);
        let Some(body) = declaration.node.child_by_field_name("body") else {
            return fields;
        };
        for member in members(body) {
            match member.kind() {
                "field_declaration" | "constant_declaration" => {
                    fields.extend(self.field_declaration(member, declaration))
                }
                "enum_constant" => fields.extend(self.enum_constant(member, declaration)),
                _ => {}
            }
        }
        fields
    }

    fn record_components(&self, declaration: &Declaration) -> Vec<Field> {
        let Some(components) = declaration.node.child_by_field_name("parameters") else {
            return Vec::new();
        };
        let mut cursor = components.walk();
        components
            .named_children(&mut cursor)
            .filter(|component| component.kind() == "formal_parameter")
            .filter_map(|component| {
                let name = component.child_by_field_name("name")?;
                let declared = component.child_by_field_name("type")?;
                Some(Field {
                    name: self.text.of(name).to_owned(),
                    declared_type: self.declared_type(declared, declaration),
                    is_static: false,
                })
            })
            .collect()
    }

    fn field_declaration(&self, node: Node, declaration: &Declaration) -> Vec<Field> {
        let Some(declared) = node.child_by_field_name("type") else {
            return Vec::new();
        };
        let is_static = has_modifier(node, "static") || declaration.kind == ClassKind::Interface;
        let mut cursor = node.walk();
        node.children_by_field_name("declarator", &mut cursor)
            .filter_map(|declarator| {
                let name = declarator.child_by_field_name("name")?;
                Some(Field {
                    name: self.text.of(name).to_owned(),
                    declared_type: self.with_dimensions(
                        self.declared_type(declared, declaration),
                        declarator.child_by_field_name("dimensions"),
                    ),
                    is_static,
                })
            })
            .collect()
    }

    fn enum_constant(&self, node: Node, declaration: &Declaration) -> Option<Field> {
        let name = node.child_by_field_name("name")?;
        Some(Field {
            name: self.text.of(name).to_owned(),
            declared_type: DeclaredType::class(&declaration.name),
            is_static: true,
        })
    }

    fn declared_type(&self, node: Node, declaration: &Declaration) -> DeclaredType {
        match node.kind() {
            "integral_type" | "floating_point_type" | "boolean_type" | "void_type" => {
                DeclaredType::primitive(self.text.of(node))
            }
            "type_identifier" => self.named_type(self.text.of(node), declaration),
            "scoped_type_identifier" => {
                let name: String = self.text.of(node).split_whitespace().collect();
                self.named_type(&name, declaration)
            }
            "generic_type" => self.generic_type(node, declaration),
            "array_type" => self.array_type(node, declaration),
            "wildcard" => self.wildcard(node, declaration),
            _ => DeclaredType::unresolved(self.text.of(node)),
        }
    }

    fn named_type(&self, name: &str, declaration: &Declaration) -> DeclaredType {
        if name == "var" {
            DeclaredType::Var
        } else if declaration
            .type_variables
            .iter()
            .any(|variable| variable == name)
        {
            DeclaredType::variable(name)
        } else {
            DeclaredType::Class {
                name: self.resolver.resolve(name, &declaration.name),
                arguments: Vec::new(),
            }
        }
    }

    fn generic_type(&self, node: Node, declaration: &Declaration) -> DeclaredType {
        let mut cursor = node.walk();
        let children: Vec<Node> = node.named_children(&mut cursor).collect();
        let base = children
            .iter()
            .find(|child| child.kind() != "type_arguments")
            .map(|base| self.declared_type(*base, declaration));
        let arguments = children
            .iter()
            .find(|child| child.kind() == "type_arguments")
            .map(|arguments| {
                let mut cursor = arguments.walk();
                arguments
                    .named_children(&mut cursor)
                    .map(|argument| self.declared_type(argument, declaration))
                    .collect()
            })
            .unwrap_or_default();
        match base {
            Some(DeclaredType::Class { name, .. }) => DeclaredType::Class { name, arguments },
            Some(other) => other,
            None => DeclaredType::unresolved(self.text.of(node)),
        }
    }

    fn array_type(&self, node: Node, declaration: &Declaration) -> DeclaredType {
        let element = node
            .child_by_field_name("element")
            .map(|element| self.declared_type(element, declaration))
            .unwrap_or_else(|| DeclaredType::unresolved(self.text.of(node)));
        self.with_dimensions(element, node.child_by_field_name("dimensions"))
    }

    fn with_dimensions(&self, element: DeclaredType, dimensions: Option<Node>) -> DeclaredType {
        let count = dimensions.map_or(0, |node| self.text.of(node).matches('[').count());
        (0..count).fold(element, |inner, _| DeclaredType::array(inner))
    }

    fn wildcard(&self, node: Node, declaration: &Declaration) -> DeclaredType {
        let mut cursor = node.walk();
        let children: Vec<Node> = node.named_children(&mut cursor).collect();
        let is_lower_bound = children.iter().any(|child| child.kind() == "super");
        let bound = children
            .iter()
            .rfind(|child| child.kind() != "super")
            .filter(|_| !is_lower_bound)
            .map(|bound| self.declared_type(*bound, declaration));
        DeclaredType::wildcard(bound)
    }
}

fn members(body: Node) -> Vec<Node> {
    let mut cursor = body.walk();
    body.named_children(&mut cursor)
        .flat_map(|member| {
            if member.kind() == "enum_body_declarations" {
                self::members(member)
            } else {
                vec![member]
            }
        })
        .collect()
}

fn has_modifier(node: Node, modifier: &str) -> bool {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .filter(|child| child.kind() == "modifiers")
        .any(|modifiers| {
            let mut cursor = modifiers.walk();
            modifiers
                .children(&mut cursor)
                .any(|child| child.kind() == modifier)
        })
}
