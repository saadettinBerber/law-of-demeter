use std::collections::{HashMap, HashSet};

use tree_sitter::Node;

use crate::declarations::{Declaration, Declarations};
use crate::error::ParseError;
use crate::model::{Class, ClassKind, ClassOrigin, DeclaredType, Import, SourceFile, TypeName};
use crate::resolver::TypeResolver;
use crate::scope::Scope;
use crate::tree::{named_children, parse_tree};

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
    let Declarations { list, names } = text.declarations(root, package.as_deref());
    let declared = list
        .iter()
        .filter(|declaration| declaration.origin == ClassOrigin::Member)
        .map(|declaration| declaration.name.clone())
        .collect::<HashSet<_>>();
    let reader = Reader {
        text,
        resolver: TypeResolver::new(package.as_deref(), &imports, declared, type_exists),
        field_names: names_by_class(&list, |declaration| text.field_names(declaration)),
        method_names: names_by_class(&list, |declaration| text.method_names(declaration)),
        nested_names: names,
    };
    let mut read: HashMap<String, Class> = list
        .iter()
        .filter(|declaration| declaration.origin == ClassOrigin::Member)
        .flat_map(|declaration| reader.read_class(declaration))
        .map(|class| (class.name.clone(), class))
        .collect();
    let classes = list
        .iter()
        .filter_map(|declaration| read.remove(&declaration.name))
        .collect();
    Ok(SourceFile {
        path: path.to_owned(),
        package,
        imports,
        classes,
    })
}

fn names_by_class(
    declarations: &[Declaration],
    names: impl Fn(&Declaration) -> HashSet<String>,
) -> HashMap<String, HashSet<String>> {
    declarations
        .iter()
        .map(|declaration| (declaration.name.clone(), names(declaration)))
        .collect()
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
}

pub(crate) struct Reader<'a> {
    pub(crate) text: Text<'a>,
    pub(crate) resolver: TypeResolver<'a>,
    pub(crate) field_names: HashMap<String, HashSet<String>>,
    pub(crate) method_names: HashMap<String, HashSet<String>>,
    pub(crate) nested_names: HashMap<usize, String>,
}

impl Reader<'_> {
    pub(crate) fn read_class(&self, declaration: &Declaration) -> Vec<Class> {
        let scope = Scope::for_class(declaration);
        let mut nested = Vec::new();
        let class = Class {
            name: declaration.name.clone(),
            kind: declaration.kind,
            origin: declaration.origin,
            outer: declaration.outer.clone(),
            captured: declaration.capture.variables(),
            annotations: self.annotations(declaration.node, &scope),
            supertypes: self.supertypes(declaration, &scope),
            fields: self.fields(declaration),
            methods: self.methods(declaration, &mut nested),
        };
        let mut classes = vec![class];
        classes.extend(nested);
        classes
    }

    pub(crate) fn read_nested<'t>(
        &self,
        node: Node<'t>,
        kind: ClassKind,
        origin: ClassOrigin,
        scope: &mut Scope,
    ) -> Option<String> {
        let name = self.nested_names.get(&node.id())?.clone();
        let body = match origin {
            ClassOrigin::Anonymous => named_children(node)
                .into_iter()
                .find(|child| child.kind() == "class_body"),
            _ => node.child_by_field_name("body"),
        };
        let declaration = Declaration {
            node,
            body,
            kind,
            origin,
            name: name.clone(),
            outer: Some(scope.class.clone()),
            type_variables: scope.type_variables().to_vec(),
            capture: scope.capture(),
        };
        scope.add_nested(self.read_class(&declaration));
        Some(name)
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

    fn supertypes(&self, declaration: &Declaration, scope: &Scope) -> Vec<DeclaredType> {
        match (declaration.node.kind(), &declaration.outer) {
            ("object_creation_expression", _) => declaration
                .node
                .child_by_field_name("type")
                .map(|created| self.declared_type(created, scope))
                .into_iter()
                .collect(),
            ("enum_constant", Some(enum_name)) => vec![DeclaredType::class(enum_name)],
            _ => self.declared_supertypes(declaration.node, scope),
        }
    }

    fn declared_supertypes(&self, declaration: Node, scope: &Scope) -> Vec<DeclaredType> {
        let clauses: Vec<Node> = named_children(declaration)
            .into_iter()
            .filter(|child| {
                matches!(
                    child.kind(),
                    "superclass" | "super_interfaces" | "extends_interfaces"
                )
            })
            .collect();
        clauses
            .into_iter()
            .flat_map(named_children)
            .flat_map(|child| {
                if child.kind() == "type_list" {
                    named_children(child)
                } else {
                    vec![child]
                }
            })
            .map(|supertype| self.declared_type(supertype, scope))
            .collect()
    }
}
