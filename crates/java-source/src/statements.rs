use tree_sitter::Node;

use crate::declarations::class_kind;
use crate::model::{
    Catch, ClassKind, ClassOrigin, DeclaredType, Expression, LocalVariable, Statement,
};
use crate::reader::Reader;
use crate::scope::{Binding, Scope};
use crate::tree::{line_of, named_children};

impl Reader<'_> {
    pub(crate) fn block_statements(&self, block: Node, scope: &mut Scope) -> Vec<Statement> {
        scope.open_frame();
        let statements = named_children(block)
            .into_iter()
            .flat_map(|child| self.statement(child, scope))
            .collect();
        scope.close_frame();
        statements
    }

    pub(crate) fn branch_statements(&self, node: Node, scope: &mut Scope) -> Vec<Statement> {
        if node.kind() == "block" {
            return self.block_statements(node, scope);
        }
        scope.open_frame();
        let statements = self.statement(node, scope);
        scope.close_frame();
        statements
    }

    fn statement(&self, node: Node, scope: &mut Scope) -> Vec<Statement> {
        match node.kind() {
            "local_variable_declaration" => self.local_variables(node, scope),
            "expression_statement" | "throw_statement" | "yield_statement" => named_children(node)
                .first()
                .map(|expression| vec![Statement::Expression(self.expression(*expression, scope))])
                .unwrap_or_default(),
            "return_statement" => vec![Statement::Return(
                named_children(node)
                    .first()
                    .map(|expression| self.expression(*expression, scope)),
            )],
            "block" => vec![Statement::Block(self.block_statements(node, scope))],
            "if_statement" => vec![self.if_statement(node, scope)],
            "for_statement" => vec![self.for_statement(node, scope)],
            "enhanced_for_statement" => self.for_each_statement(node, scope).into_iter().collect(),
            "while_statement" | "do_statement" => {
                self.while_statement(node, scope).into_iter().collect()
            }
            "try_statement" | "try_with_resources_statement" => {
                vec![self.try_statement(node, scope)]
            }
            "switch_expression" => vec![Statement::Expression(self.expression(node, scope))],
            "synchronized_statement" => self.synchronized_statement(node, scope),
            "labeled_statement" => named_children(node)
                .last()
                .map(|inner| self.statement(*inner, scope))
                .unwrap_or_default(),
            "assert_statement" | "explicit_constructor_invocation" => {
                vec![Statement::Expression(Expression::Operation(
                    self.operands(node, scope),
                ))]
            }
            kind if class_kind(kind).is_some_and(|class| class != ClassKind::Annotation) => {
                self.local_class(node, scope);
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    fn local_class(&self, node: Node, scope: &mut Scope) {
        let (Some(kind), Some(name)) = (class_kind(node.kind()), node.child_by_field_name("name"))
        else {
            return;
        };
        let Some(binary_name) = self.nested_names.get(&node.id()) else {
            return;
        };
        scope.declare_type(self.text.of(name), binary_name);
        self.read_nested(node, kind, ClassOrigin::Local, scope);
    }

    fn local_variables(&self, node: Node, scope: &mut Scope) -> Vec<Statement> {
        let Some(declared) = node.child_by_field_name("type") else {
            return Vec::new();
        };
        let declared_type = self.declared_type(declared, scope);
        let mut cursor = node.walk();
        let declarators: Vec<Node> = node
            .children_by_field_name("declarator", &mut cursor)
            .collect();
        declarators
            .into_iter()
            .filter_map(|declarator| {
                self.local_variable(declarator, &declared_type, scope)
                    .map(Statement::Local)
            })
            .collect()
    }

    fn local_variable(
        &self,
        declarator: Node,
        declared_type: &DeclaredType,
        scope: &mut Scope,
    ) -> Option<LocalVariable> {
        let name = declarator.child_by_field_name("name")?;
        let initializer = declarator
            .child_by_field_name("value")
            .map(|value| self.expression(value, scope));
        let name_text = self.text.of(name);
        scope.declare(name_text, Binding::Local);
        Some(LocalVariable {
            name: name_text.to_owned(),
            declared_type: self.with_dimensions(
                declared_type.clone(),
                declarator.child_by_field_name("dimensions"),
            ),
            initializer,
            line: line_of(name),
        })
    }

    fn if_statement(&self, node: Node, scope: &mut Scope) -> Statement {
        let condition = self.condition(node, scope);
        let then_branch = node
            .child_by_field_name("consequence")
            .map(|branch| self.branch_statements(branch, scope))
            .unwrap_or_default();
        let else_branch = node
            .child_by_field_name("alternative")
            .map(|branch| self.branch_statements(branch, scope))
            .unwrap_or_default();
        Statement::If {
            condition,
            then_branch,
            else_branch,
        }
    }

    fn for_statement(&self, node: Node, scope: &mut Scope) -> Statement {
        scope.open_frame();
        let mut cursor = node.walk();
        let init_nodes: Vec<Node> = node.children_by_field_name("init", &mut cursor).collect();
        let init = init_nodes
            .into_iter()
            .flat_map(|init| {
                if init.kind() == "local_variable_declaration" {
                    self.local_variables(init, scope)
                } else {
                    vec![Statement::Expression(self.expression(init, scope))]
                }
            })
            .collect();
        let condition = node
            .child_by_field_name("condition")
            .map(|condition| self.expression(condition, scope));
        let mut cursor = node.walk();
        let update_nodes: Vec<Node> = node.children_by_field_name("update", &mut cursor).collect();
        let update = update_nodes
            .into_iter()
            .map(|update| self.expression(update, scope))
            .collect();
        let body = node
            .child_by_field_name("body")
            .map(|body| self.branch_statements(body, scope))
            .unwrap_or_default();
        scope.close_frame();
        Statement::For {
            init,
            condition,
            update,
            body,
        }
    }

    fn for_each_statement(&self, node: Node, scope: &mut Scope) -> Option<Statement> {
        let name = node.child_by_field_name("name")?;
        let declared = node.child_by_field_name("type")?;
        let iterable = self.expression(node.child_by_field_name("value")?, scope);
        scope.open_frame();
        let name_text = self.text.of(name);
        scope.declare(name_text, Binding::Local);
        let variable = LocalVariable {
            name: name_text.to_owned(),
            declared_type: self.declared_type(declared, scope),
            initializer: None,
            line: line_of(name),
        };
        let body = node
            .child_by_field_name("body")
            .map(|body| self.branch_statements(body, scope))
            .unwrap_or_default();
        scope.close_frame();
        Some(Statement::ForEach {
            variable,
            iterable,
            body,
        })
    }

    fn while_statement(&self, node: Node, scope: &mut Scope) -> Option<Statement> {
        let condition = self.condition(node, scope);
        let body = self.branch_statements(node.child_by_field_name("body")?, scope);
        Some(Statement::While { condition, body })
    }

    fn try_statement(&self, node: Node, scope: &mut Scope) -> Statement {
        scope.open_frame();
        let resources = node
            .child_by_field_name("resources")
            .map(|specification| {
                named_children(specification)
                    .into_iter()
                    .filter_map(|resource| self.resource(resource, scope))
                    .collect()
            })
            .unwrap_or_default();
        let body = node
            .child_by_field_name("body")
            .map(|body| self.branch_statements(body, scope))
            .unwrap_or_default();
        scope.close_frame();
        let children = named_children(node);
        let catches = children
            .iter()
            .filter(|child| child.kind() == "catch_clause")
            .filter_map(|clause| self.catch(*clause, scope))
            .collect();
        let finally = children
            .iter()
            .find(|child| child.kind() == "finally_clause")
            .and_then(|clause| named_children(*clause).into_iter().next())
            .map(|block| self.branch_statements(block, scope))
            .unwrap_or_default();
        Statement::Try {
            resources,
            body,
            catches,
            finally,
        }
    }

    fn resource(&self, node: Node, scope: &mut Scope) -> Option<Statement> {
        let (Some(declared), Some(name), Some(value)) = (
            node.child_by_field_name("type"),
            node.child_by_field_name("name"),
            node.child_by_field_name("value"),
        ) else {
            return named_children(node)
                .first()
                .map(|existing| Statement::Expression(self.expression(*existing, scope)));
        };
        let initializer = self.expression(value, scope);
        let name_text = self.text.of(name);
        scope.declare(name_text, Binding::Local);
        Some(Statement::Local(LocalVariable {
            name: name_text.to_owned(),
            declared_type: self.declared_type(declared, scope),
            initializer: Some(initializer),
            line: line_of(name),
        }))
    }

    fn catch(&self, clause: Node, scope: &mut Scope) -> Option<Catch> {
        let children = named_children(clause);
        let parameter = children
            .iter()
            .find(|child| child.kind() == "catch_formal_parameter")?;
        let name = self
            .text
            .of(parameter.child_by_field_name("name")?)
            .to_owned();
        let types = named_children(*parameter)
            .into_iter()
            .filter(|child| child.kind() == "catch_type")
            .flat_map(named_children)
            .map(|catch_type| self.declared_type(catch_type, scope))
            .collect();
        scope.open_frame();
        scope.declare(&name, Binding::Local);
        let body = clause
            .child_by_field_name("body")
            .map(|body| self.branch_statements(body, scope))
            .unwrap_or_default();
        scope.close_frame();
        Some(Catch { name, types, body })
    }

    fn synchronized_statement(&self, node: Node, scope: &mut Scope) -> Vec<Statement> {
        let lock = named_children(node)
            .into_iter()
            .find(|child| child.kind() == "parenthesized_expression")
            .map(|lock| Statement::Expression(self.expression(lock, scope)));
        let body = node
            .child_by_field_name("body")
            .map(|body| self.branch_statements(body, scope))
            .unwrap_or_default();
        vec![Statement::Block(lock.into_iter().chain(body).collect())]
    }

    fn condition(&self, node: Node, scope: &mut Scope) -> Expression {
        node.child_by_field_name("condition")
            .map(|condition| self.expression(condition, scope))
            .unwrap_or(Expression::Operation(Vec::new()))
    }
}
