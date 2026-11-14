use tree_sitter::Node;

use crate::model::DeclaredType;
use crate::reader::Reader;
use crate::scope::Scope;

impl Reader<'_> {
    pub(crate) fn declared_type(&self, node: Node, scope: &Scope) -> DeclaredType {
        match node.kind() {
            "integral_type" | "floating_point_type" | "boolean_type" | "void_type" => {
                DeclaredType::primitive(self.text.of(node))
            }
            "type_identifier" => self.named_type(self.text.of(node), scope),
            "scoped_type_identifier" => {
                let name: String = self.text.of(node).split_whitespace().collect();
                self.named_type(&name, scope)
            }
            "generic_type" => self.generic_type(node, scope),
            "array_type" => self.array_type(node, scope),
            "wildcard" => self.wildcard(node, scope),
            _ => DeclaredType::unresolved(self.text.of(node)),
        }
    }

    pub(crate) fn with_dimensions(
        &self,
        element: DeclaredType,
        dimensions: Option<Node>,
    ) -> DeclaredType {
        let count = dimensions.map_or(0, |node| self.text.of(node).matches('[').count());
        (0..count).fold(element, |inner, _| DeclaredType::array(inner))
    }

    fn named_type(&self, name: &str, scope: &Scope) -> DeclaredType {
        if name == "var" {
            DeclaredType::Var
        } else if scope.is_type_variable(name) {
            DeclaredType::variable(name)
        } else {
            DeclaredType::Class {
                name: self.resolver.resolve(name, &scope.class),
                arguments: Vec::new(),
            }
        }
    }

    fn generic_type(&self, node: Node, scope: &Scope) -> DeclaredType {
        let mut cursor = node.walk();
        let children: Vec<Node> = node.named_children(&mut cursor).collect();
        let base = children
            .iter()
            .find(|child| child.kind() != "type_arguments")
            .map(|base| self.declared_type(*base, scope));
        let arguments = children
            .iter()
            .find(|child| child.kind() == "type_arguments")
            .map(|arguments| {
                let mut cursor = arguments.walk();
                arguments
                    .named_children(&mut cursor)
                    .map(|argument| self.declared_type(argument, scope))
                    .collect()
            })
            .unwrap_or_default();
        match base {
            Some(DeclaredType::Class { name, .. }) => DeclaredType::Class { name, arguments },
            Some(other) => other,
            None => DeclaredType::unresolved(self.text.of(node)),
        }
    }

    fn array_type(&self, node: Node, scope: &Scope) -> DeclaredType {
        let element = node
            .child_by_field_name("element")
            .map(|element| self.declared_type(element, scope))
            .unwrap_or_else(|| DeclaredType::unresolved(self.text.of(node)));
        self.with_dimensions(element, node.child_by_field_name("dimensions"))
    }

    fn wildcard(&self, node: Node, scope: &Scope) -> DeclaredType {
        let mut cursor = node.walk();
        let children: Vec<Node> = node.named_children(&mut cursor).collect();
        let is_lower_bound = children.iter().any(|child| child.kind() == "super");
        let bound = children
            .iter()
            .rfind(|child| child.kind() != "super")
            .filter(|_| !is_lower_bound)
            .map(|bound| self.declared_type(*bound, scope));
        DeclaredType::wildcard(bound)
    }
}
