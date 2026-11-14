use tree_sitter::Node;

use crate::model::{
    Call, DeclaredType, Expression, Lambda, LambdaParameter, New, Statement, Switch, SwitchBranch,
    TypeName,
};
use crate::reader::Reader;
use crate::scope::{Binding, Scope};
use crate::tree::{line_of, named_children};

impl Reader<'_> {
    pub(crate) fn expression(&self, node: Node, scope: &mut Scope) -> Expression {
        match node.kind() {
            "identifier" => self.name(self.text.of(node), scope),
            "this" => Expression::This,
            "super" => Expression::Super,
            "parenthesized_expression" => named_children(node)
                .first()
                .map_or(Expression::Operation(Vec::new()), |inner| {
                    self.expression(*inner, scope)
                }),
            "field_access" => self.field_access(node, scope),
            "method_invocation" => self.call(node, scope),
            "object_creation_expression" => self.creation(node, scope),
            "lambda_expression" => self.lambda(node, scope),
            "method_reference" => self.method_reference(node, scope),
            "cast_expression" => self.cast(node, scope),
            "switch_expression" => Expression::Switch(self.switch(node, scope)),
            "instanceof_expression" => self.instance_of(node, scope),
            kind if is_literal(kind) => Expression::Literal,
            _ => Expression::Operation(self.operands(node, scope)),
        }
    }

    pub(crate) fn operands(&self, node: Node, scope: &mut Scope) -> Vec<Expression> {
        named_children(node)
            .into_iter()
            .filter(|child| !is_type_node(child.kind()))
            .map(|child| self.expression(child, scope))
            .collect()
    }

    fn name(&self, name: &str, scope: &Scope) -> Expression {
        if let Some(binding) = scope.lookup(name) {
            return binding;
        }
        if let Some(owner) = self.owner_of_field(name, &scope.class) {
            return if owner == scope.class {
                Expression::Field(name.to_owned())
            } else {
                Expression::OuterField {
                    owner,
                    name: name.to_owned(),
                }
            };
        }
        match self.resolver.resolve_simple(name, &scope.class) {
            Some(binary_name) => Expression::Type(TypeName::Resolved(binary_name)),
            None => Expression::Unresolved(name.to_owned()),
        }
    }

    fn owner_of_field(&self, name: &str, class: &str) -> Option<String> {
        let mut owner = class;
        loop {
            if self
                .field_names
                .get(owner)
                .is_some_and(|names| names.contains(name))
            {
                return Some(owner.to_owned());
            }
            owner = owner.rsplit_once('$')?.0;
        }
    }

    fn field_access(&self, node: Node, scope: &mut Scope) -> Expression {
        let (Some(object), Some(field)) = (
            node.child_by_field_name("object"),
            node.child_by_field_name("field"),
        ) else {
            return Expression::Operation(Vec::new());
        };
        let name = self.text.of(field);
        if object.kind() == "this" {
            return Expression::Field(name.to_owned());
        }
        let target = self.expression(object, scope);
        if let Some(nested) = self.nested_type(&target, name) {
            return Expression::Type(nested);
        }
        if let Some(qualified) = self.qualified_type(node, scope) {
            return Expression::Type(qualified);
        }
        Expression::FieldAccess {
            target: Box::new(target),
            name: name.to_owned(),
        }
    }

    fn nested_type(&self, target: &Expression, name: &str) -> Option<TypeName> {
        let Expression::Type(TypeName::Resolved(owner)) = target else {
            return None;
        };
        self.resolver
            .known(&format!("{owner}${name}"))
            .map(TypeName::Resolved)
    }

    fn qualified_type(&self, node: Node, scope: &Scope) -> Option<TypeName> {
        let dotted = self.dotted_name(node)?;
        let root = dotted.split('.').next()?;
        let is_variable =
            scope.lookup(root).is_some() || self.owner_of_field(root, &scope.class).is_some();
        if is_variable {
            return None;
        }
        self.resolver
            .qualified_by_catalog(&dotted)
            .map(TypeName::Resolved)
    }

    fn dotted_name(&self, node: Node) -> Option<String> {
        match node.kind() {
            "identifier" => Some(self.text.of(node).to_owned()),
            "field_access" => {
                let object = self.dotted_name(node.child_by_field_name("object")?)?;
                let field = self.text.of(node.child_by_field_name("field")?);
                Some(format!("{object}.{field}"))
            }
            _ => None,
        }
    }

    fn call(&self, node: Node, scope: &mut Scope) -> Expression {
        let Some(name) = node.child_by_field_name("name") else {
            return Expression::Operation(Vec::new());
        };
        let receiver = node
            .child_by_field_name("object")
            .map(|object| Box::new(self.expression(object, scope)));
        let arguments = node
            .child_by_field_name("arguments")
            .map(|arguments| self.operands(arguments, scope))
            .unwrap_or_default();
        Expression::Call(Call {
            receiver,
            name: self.text.of(name).to_owned(),
            arguments,
            line: line_of(name),
        })
    }

    fn creation(&self, node: Node, scope: &mut Scope) -> Expression {
        let Some(created) = node.child_by_field_name("type") else {
            return Expression::Operation(Vec::new());
        };
        let arguments = node
            .child_by_field_name("arguments")
            .map(|arguments| self.operands(arguments, scope))
            .unwrap_or_default();
        Expression::New(New {
            created: self.declared_type(created, scope),
            arguments,
            line: line_of(node),
        })
    }

    fn lambda(&self, node: Node, scope: &mut Scope) -> Expression {
        let parameters = node
            .child_by_field_name("parameters")
            .map(|parameters| self.lambda_parameters(parameters, scope))
            .unwrap_or_default();
        scope.open_frame();
        for parameter in &parameters {
            scope.declare(&parameter.name, Binding::LambdaParameter);
        }
        let body = match node.child_by_field_name("body") {
            Some(body) if body.kind() == "block" => self.block_statements(body, scope),
            Some(body) => vec![Statement::Expression(self.expression(body, scope))],
            None => Vec::new(),
        };
        scope.close_frame();
        Expression::Lambda(Lambda { parameters, body })
    }

    fn lambda_parameters(&self, parameters: Node, scope: &Scope) -> Vec<LambdaParameter> {
        if parameters.kind() == "identifier" {
            return vec![LambdaParameter {
                name: self.text.of(parameters).to_owned(),
                declared_type: None,
            }];
        }
        named_children(parameters)
            .into_iter()
            .filter_map(|parameter| match parameter.kind() {
                "identifier" => Some(LambdaParameter {
                    name: self.text.of(parameter).to_owned(),
                    declared_type: None,
                }),
                "formal_parameter" => {
                    let name = parameter.child_by_field_name("name")?;
                    let declared =
                        self.declared_type(parameter.child_by_field_name("type")?, scope);
                    Some(LambdaParameter {
                        name: self.text.of(name).to_owned(),
                        declared_type: Some(declared)
                            .filter(|declared| *declared != DeclaredType::Var),
                    })
                }
                _ => None,
            })
            .collect()
    }

    fn method_reference(&self, node: Node, scope: &mut Scope) -> Expression {
        let children = named_children(node);
        let Some(target) = children.first() else {
            return Expression::Operation(Vec::new());
        };
        let name = children
            .get(1)
            .map_or("new", |name| self.text.of(*name))
            .to_owned();
        let target = if is_type_node(target.kind()) {
            match self.declared_type(*target, scope) {
                DeclaredType::Class { name, .. } => Expression::Type(name),
                _ => Expression::Operation(Vec::new()),
            }
        } else {
            self.expression(*target, scope)
        };
        Expression::MethodReference {
            target: Box::new(target),
            name,
        }
    }

    fn cast(&self, node: Node, scope: &mut Scope) -> Expression {
        let (Some(target), Some(operand)) = (
            node.child_by_field_name("type"),
            node.child_by_field_name("value"),
        ) else {
            return Expression::Operation(Vec::new());
        };
        Expression::Cast {
            target: self.declared_type(target, scope),
            operand: Box::new(self.expression(operand, scope)),
        }
    }

    fn instance_of(&self, node: Node, scope: &mut Scope) -> Expression {
        let tested = node
            .child_by_field_name("left")
            .map(|left| self.expression(left, scope));
        if let Some(binding) = node.child_by_field_name("name") {
            scope.declare(self.text.of(binding), Binding::Local);
        }
        self.declare_patterns(node, scope);
        Expression::Operation(tested.into_iter().collect())
    }

    fn declare_patterns(&self, node: Node, scope: &mut Scope) {
        for child in named_children(node) {
            if child.kind() == "type_pattern" {
                if let Some(binding) = named_children(child).last() {
                    scope.declare(self.text.of(*binding), Binding::Local);
                }
            } else {
                self.declare_patterns(child, scope);
            }
        }
    }

    pub(crate) fn switch(&self, node: Node, scope: &mut Scope) -> Switch {
        let selector = node
            .child_by_field_name("condition")
            .map_or(Expression::Operation(Vec::new()), |condition| {
                self.expression(condition, scope)
            });
        let branches = node
            .child_by_field_name("body")
            .map(|block| {
                named_children(block)
                    .into_iter()
                    .map(|branch| self.switch_branch(branch, scope))
                    .collect()
            })
            .unwrap_or_default();
        Switch {
            selector: Box::new(selector),
            branches,
        }
    }

    fn switch_branch(&self, branch: Node, scope: &mut Scope) -> SwitchBranch {
        scope.open_frame();
        let children = named_children(branch);
        let labels = children
            .iter()
            .filter(|child| child.kind() == "switch_label");
        let mut guard = None;
        for label in labels {
            self.declare_patterns(*label, scope);
            guard = guard.or_else(|| {
                named_children(*label)
                    .into_iter()
                    .find(|part| part.kind() == "guard")
                    .and_then(|part| named_children(part).first().copied())
                    .map(|condition| self.expression(condition, scope))
            });
        }
        let body = children
            .iter()
            .filter(|child| child.kind() != "switch_label")
            .flat_map(|statement| self.branch_statements(*statement, scope))
            .collect();
        scope.close_frame();
        SwitchBranch { guard, body }
    }
}

fn is_literal(kind: &str) -> bool {
    kind.ends_with("_literal") || matches!(kind, "true" | "false")
}

fn is_type_node(kind: &str) -> bool {
    matches!(
        kind,
        "integral_type"
            | "floating_point_type"
            | "boolean_type"
            | "void_type"
            | "type_identifier"
            | "scoped_type_identifier"
            | "generic_type"
            | "array_type"
            | "dimensions"
            | "annotation"
            | "marker_annotation"
            | "type_arguments"
            | "modifiers"
    )
}
