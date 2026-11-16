use std::collections::{HashMap, HashSet};

use tree_sitter::Node;

use crate::model::{
    Call, ClassKind, ClassOrigin, DeclaredType, Expression, ImplicitTarget, Lambda,
    LambdaParameter, LiteralType, New, PatternBinding, Statement, Switch, SwitchBranch, TypeName,
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
            "assignment_expression" => self.assignment(node, scope),
            "field_access" => self.field_access(node, scope),
            "method_invocation" => self.call(node, scope),
            "object_creation_expression" => self.creation(node, scope),
            "lambda_expression" => self.lambda(node, scope),
            "method_reference" => self.method_reference(node, scope),
            "cast_expression" => self.cast(node, scope),
            "switch_expression" => Expression::Switch(self.switch(node, scope)),
            "instanceof_expression" => self.instance_of(node, scope),
            "class_literal" => self.class_literal(node, scope),
            kind if is_literal(kind) => Expression::Literal(literal_type(kind, self.text.of(node))),
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
        if self.declares_field(&scope.class, name) {
            return Expression::Field(name.to_owned());
        }
        if scope.is_captured(name) {
            return Expression::Captured(name.to_owned());
        }
        if let Some(owner) = scope
            .class
            .rsplit_once('$')
            .and_then(|(outer, _)| self.owner_of_field(name, outer))
        {
            return Expression::OuterField {
                owner,
                name: name.to_owned(),
            };
        }
        if let Some(owner) = self.resolver.static_import_owner(name) {
            return Expression::StaticImport {
                owner: TypeName::Resolved(owner.to_owned()),
                name: name.to_owned(),
            };
        }
        let binary_name = scope
            .lookup_type(name)
            .map(str::to_owned)
            .or_else(|| self.resolver.resolve_simple(name, &scope.class));
        match binary_name {
            Some(binary_name) => Expression::Type(TypeName::Resolved(binary_name)),
            None => self.unresolved_name(name),
        }
    }

    fn unresolved_name(&self, name: &str) -> Expression {
        let candidates = self.on_demand_static_owners();
        if candidates.is_empty() {
            Expression::Unresolved(name.to_owned())
        } else {
            Expression::OnDemandStaticImport {
                name: name.to_owned(),
                candidates,
            }
        }
    }

    fn on_demand_static_owners(&self) -> Vec<TypeName> {
        self.resolver
            .static_on_demand_owners()
            .iter()
            .map(|owner| TypeName::Resolved(owner.clone()))
            .collect()
    }

    fn declares_field(&self, class: &str, name: &str) -> bool {
        self.field_names
            .get(class)
            .is_some_and(|names| names.contains(name))
    }

    fn owner_of_field(&self, name: &str, class: &str) -> Option<String> {
        declaring_class(&self.field_names, name, class)
    }

    fn implicit_target(&self, name: &str, scope: &Scope) -> ImplicitTarget {
        match declaring_class(&self.method_names, name, &scope.class) {
            Some(owner) if owner == scope.class => return ImplicitTarget::OwnMethod,
            Some(owner) => return ImplicitTarget::OuterMethod { owner },
            None => {}
        }
        if let Some(owner) = self.resolver.static_import_owner(name) {
            return ImplicitTarget::StaticImport {
                owner: TypeName::Resolved(owner.to_owned()),
            };
        }
        let candidates = self.on_demand_static_owners();
        if candidates.is_empty() {
            ImplicitTarget::OwnMethod
        } else {
            ImplicitTarget::OnDemandStaticImport { candidates }
        }
    }

    fn assignment(&self, node: Node, scope: &mut Scope) -> Expression {
        let (Some(left), Some(right)) = (
            node.child_by_field_name("left"),
            node.child_by_field_name("right"),
        ) else {
            return Expression::Operation(Vec::new());
        };
        let is_plain = node
            .child_by_field_name("operator")
            .is_some_and(|operator| self.text.of(operator) == "=");
        let target = self.expression(left, scope);
        let value = self.expression(right, scope);
        if is_plain {
            Expression::Assignment {
                target: Box::new(target),
                value: Box::new(value),
            }
        } else {
            Expression::Operation(vec![target, value])
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
        let implicit_target = receiver
            .is_none()
            .then(|| self.implicit_target(self.text.of(name), scope));
        let arguments = node
            .child_by_field_name("arguments")
            .map(|arguments| self.operands(arguments, scope))
            .unwrap_or_default();
        Expression::Call(Call {
            receiver,
            implicit_target,
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
            anonymous_class: self.read_nested(
                node,
                ClassKind::Class,
                ClassOrigin::Anonymous,
                scope,
            ),
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
            scope.declare(
                &parameter.name,
                Binding::LambdaParameter,
                parameter.declared_type.clone().unwrap_or(DeclaredType::Var),
            );
        }
        let body = match node.child_by_field_name("body") {
            Some(body) if body.kind() == "block" => self.block_statements(body, scope),
            Some(body) => vec![Statement::Return(Some(self.expression(body, scope)))],
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
            line: line_of(node),
        }
    }

    fn class_literal(&self, node: Node, scope: &Scope) -> Expression {
        named_children(node)
            .first()
            .map_or(Expression::Literal(LiteralType::Null), |named| {
                Expression::ClassLiteral(self.declared_type(*named, scope))
            })
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
        let operand = node
            .child_by_field_name("left")
            .map_or(Expression::Operation(Vec::new()), |left| {
                self.expression(left, scope)
            });
        let (tested, bindings) = match node.child_by_field_name("pattern") {
            Some(pattern) => (
                named_children(pattern)
                    .first()
                    .map_or(DeclaredType::Var, |tested| {
                        self.declared_type(*tested, scope)
                    }),
                self.pattern_bindings(pattern, scope),
            ),
            None => {
                let tested = node
                    .child_by_field_name("right")
                    .map_or(DeclaredType::Var, |tested| {
                        self.declared_type(tested, scope)
                    });
                let bindings = node
                    .child_by_field_name("name")
                    .map(|name| PatternBinding {
                        name: self.text.of(name).to_owned(),
                        declared_type: tested.clone(),
                        is_component: false,
                    })
                    .into_iter()
                    .collect();
                (tested, bindings)
            }
        };
        for binding in &bindings {
            scope.declare(&binding.name, Binding::Local, binding.declared_type.clone());
        }
        Expression::InstanceOf {
            operand: Box::new(operand),
            tested,
            bindings,
        }
    }

    fn pattern_bindings(&self, node: Node, scope: &Scope) -> Vec<PatternBinding> {
        let children = named_children(node);
        let is_binding = matches!(node.kind(), "type_pattern" | "record_pattern_component");
        if let (true, [declared, name]) = (is_binding, children.as_slice())
            && name.kind() == "identifier"
        {
            return vec![PatternBinding {
                name: self.text.of(*name).to_owned(),
                declared_type: self.declared_type(*declared, scope),
                is_component: node.kind() == "record_pattern_component",
            }];
        }
        children
            .into_iter()
            .flat_map(|child| self.pattern_bindings(child, scope))
            .collect()
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
        let labels: Vec<Node> = children
            .iter()
            .filter(|child| child.kind() == "switch_label")
            .copied()
            .collect();
        let bindings: Vec<PatternBinding> = labels
            .iter()
            .flat_map(|label| named_children(*label))
            .filter(|part| part.kind() == "pattern")
            .flat_map(|pattern| self.pattern_bindings(pattern, scope))
            .collect();
        for binding in &bindings {
            scope.declare(&binding.name, Binding::Local, binding.declared_type.clone());
        }
        let guard = labels
            .iter()
            .flat_map(|label| named_children(*label))
            .find(|part| part.kind() == "guard")
            .and_then(|part| named_children(part).first().copied())
            .map(|condition| self.expression(condition, scope));
        let body = children
            .iter()
            .filter(|child| child.kind() != "switch_label")
            .flat_map(|statement| self.branch_statements(*statement, scope))
            .collect();
        scope.close_frame();
        SwitchBranch {
            bindings,
            guard,
            body,
        }
    }
}

fn declaring_class(
    declared: &HashMap<String, HashSet<String>>,
    name: &str,
    class: &str,
) -> Option<String> {
    let mut owner = class;
    loop {
        if declared
            .get(owner)
            .is_some_and(|names| names.contains(name))
        {
            return Some(owner.to_owned());
        }
        owner = owner.rsplit_once('$')?.0;
    }
}

fn literal_type(kind: &str, text: &str) -> LiteralType {
    match kind {
        "true" | "false" => LiteralType::Boolean,
        "character_literal" => LiteralType::Char,
        "null_literal" => LiteralType::Null,
        "decimal_floating_point_literal" | "hex_floating_point_literal" => floating_type(text),
        kind if kind.ends_with("_integer_literal") => integer_type(text),
        _ => LiteralType::String,
    }
}

fn integer_type(text: &str) -> LiteralType {
    if text.ends_with(['l', 'L']) {
        LiteralType::Long
    } else {
        LiteralType::Int
    }
}

fn floating_type(text: &str) -> LiteralType {
    if text.ends_with(['f', 'F']) {
        LiteralType::Float
    } else {
        LiteralType::Double
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
