use std::collections::{HashMap, HashSet};

use tree_sitter::Node;

use crate::model::{ClassKind, ClassOrigin};
use crate::reader::Text;
use crate::scope::Capture;
use crate::tree::{members, named_children};

pub(crate) struct Declaration<'t> {
    pub(crate) node: Node<'t>,
    pub(crate) body: Option<Node<'t>>,
    pub(crate) kind: ClassKind,
    pub(crate) origin: ClassOrigin,
    pub(crate) name: String,
    pub(crate) outer: Option<String>,
    pub(crate) type_variables: Vec<String>,
    pub(crate) capture: Capture,
}

#[derive(Default)]
pub(crate) struct Declarations<'t> {
    pub(crate) list: Vec<Declaration<'t>>,
    pub(crate) names: HashMap<usize, String>,
}

pub(crate) fn class_kind(node_kind: &str) -> Option<ClassKind> {
    match node_kind {
        "class_declaration" => Some(ClassKind::Class),
        "interface_declaration" => Some(ClassKind::Interface),
        "enum_declaration" => Some(ClassKind::Enum),
        "record_declaration" => Some(ClassKind::Record),
        "annotation_type_declaration" => Some(ClassKind::Annotation),
        _ => None,
    }
}

struct Header<'a> {
    kind: ClassKind,
    origin: ClassOrigin,
    name: String,
    outer: Option<&'a str>,
}

struct Enclosing<'a> {
    class: &'a str,
    type_variables: &'a [String],
}

impl Text<'_> {
    pub(crate) fn declarations<'t>(
        &self,
        root: Node<'t>,
        package: Option<&str>,
    ) -> Declarations<'t> {
        let prefix = package.map_or_else(String::new, |name| format!("{name}."));
        let mut found = Declarations::default();
        for child in named_children(root) {
            if let Some(kind) = class_kind(child.kind()) {
                self.collect_member_class(child, kind, &prefix, None, &[], &mut found);
            }
        }
        found
    }

    pub(crate) fn field_names(&self, declaration: &Declaration) -> HashSet<String> {
        let components = declaration
            .node
            .child_by_field_name("parameters")
            .map(|parameters| {
                named_children(parameters)
                    .into_iter()
                    .filter_map(|parameter| parameter.child_by_field_name("name"))
                    .map(|name| self.of(name).to_owned())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let declared = declaration.body.map(members).unwrap_or_default();
        let members = declared.into_iter().flat_map(|member| match member.kind() {
            "field_declaration" | "constant_declaration" => {
                let mut cursor = member.walk();
                member
                    .children_by_field_name("declarator", &mut cursor)
                    .filter_map(|declarator| declarator.child_by_field_name("name"))
                    .map(|name| self.of(name).to_owned())
                    .collect()
            }
            "enum_constant" => member
                .child_by_field_name("name")
                .map(|name| self.of(name).to_owned())
                .into_iter()
                .collect(),
            _ => Vec::new(),
        });
        components.into_iter().chain(members).collect()
    }

    pub(crate) fn method_names(&self, declaration: &Declaration) -> HashSet<String> {
        declaration
            .body
            .map(members)
            .unwrap_or_default()
            .into_iter()
            .filter(|member| member.kind() == "method_declaration")
            .filter_map(|member| member.child_by_field_name("name"))
            .map(|name| self.of(name).to_owned())
            .collect()
    }

    pub(crate) fn type_parameters(&self, declaration: Node) -> Vec<String> {
        let Some(parameters) = declaration.child_by_field_name("type_parameters") else {
            return Vec::new();
        };
        named_children(parameters)
            .into_iter()
            .filter(|parameter| parameter.kind() == "type_parameter")
            .filter_map(|parameter| parameter.named_child(0))
            .map(|name| self.of(name).to_owned())
            .collect()
    }

    fn collect_member_class<'t>(
        &self,
        node: Node<'t>,
        kind: ClassKind,
        prefix: &str,
        outer: Option<&str>,
        enclosing_type_variables: &[String],
        found: &mut Declarations<'t>,
    ) {
        let Some(name) = node.child_by_field_name("name") else {
            return;
        };
        let binary_name = format!("{prefix}{}", self.of(name));
        let header = Header {
            kind,
            origin: ClassOrigin::Member,
            name: binary_name,
            outer,
        };
        self.add_class(node, header, enclosing_type_variables, found);
    }

    fn add_class<'t>(
        &self,
        node: Node<'t>,
        header: Header,
        enclosing_type_variables: &[String],
        found: &mut Declarations<'t>,
    ) {
        let body = match header.origin {
            ClassOrigin::Anonymous => named_children(node)
                .into_iter()
                .find(|child| child.kind() == "class_body"),
            _ => node.child_by_field_name("body"),
        };
        let mut type_variables = enclosing_type_variables.to_vec();
        type_variables.extend(self.type_parameters(node));
        found.list.push(Declaration {
            node,
            body,
            kind: header.kind,
            origin: header.origin,
            name: header.name.clone(),
            outer: header.outer.map(str::to_owned),
            type_variables: type_variables.clone(),
            capture: Capture::default(),
        });
        if let Some(body) = body {
            self.collect_members(&header.name, &type_variables, body, found);
        }
    }

    fn collect_members<'t>(
        &self,
        class: &str,
        type_variables: &[String],
        body: Node<'t>,
        found: &mut Declarations<'t>,
    ) {
        let mut counters = HashMap::new();
        for member in members(body) {
            if let Some(kind) = class_kind(member.kind()) {
                self.collect_member_class(
                    member,
                    kind,
                    &format!("{class}$"),
                    Some(class),
                    type_variables,
                    found,
                );
                continue;
            }
            let enclosing = Enclosing {
                class,
                type_variables,
            };
            self.collect_local(member, &enclosing, &mut counters, found);
        }
    }

    fn collect_local<'t>(
        &self,
        node: Node<'t>,
        enclosing: &Enclosing,
        counters: &mut HashMap<String, usize>,
        found: &mut Declarations<'t>,
    ) {
        let kind = node.kind();
        if matches!(kind, "object_creation_expression" | "enum_constant") {
            let body = match kind {
                "enum_constant" => node.child_by_field_name("body"),
                _ => named_children(node)
                    .into_iter()
                    .find(|child| child.kind() == "class_body"),
            };
            for child in named_children(node) {
                if Some(child) != body {
                    self.collect_local(child, enclosing, counters, found);
                }
            }
            if body.is_some() {
                self.collect_nested_class(node, ClassKind::Class, "", enclosing, counters, found);
            }
        } else if let Some(class) = class_kind(kind).filter(|class| *class != ClassKind::Annotation)
        {
            let name = node
                .child_by_field_name("name")
                .map(|name| self.of(name).to_owned())
                .unwrap_or_default();
            self.collect_nested_class(node, class, &name, enclosing, counters, found);
        } else {
            for child in named_children(node) {
                self.collect_local(child, enclosing, counters, found);
            }
        }
    }

    fn collect_nested_class<'t>(
        &self,
        node: Node<'t>,
        kind: ClassKind,
        simple_name: &str,
        enclosing: &Enclosing,
        counters: &mut HashMap<String, usize>,
        found: &mut Declarations<'t>,
    ) {
        let index = counters.entry(simple_name.to_owned()).or_insert(0);
        *index += 1;
        let name = format!("{}${index}{simple_name}", enclosing.class);
        found.names.insert(node.id(), name.clone());
        let origin = if simple_name.is_empty() {
            ClassOrigin::Anonymous
        } else {
            ClassOrigin::Local
        };
        let header = Header {
            kind,
            origin,
            name,
            outer: Some(enclosing.class),
        };
        self.add_class(node, header, enclosing.type_variables, found);
    }
}
