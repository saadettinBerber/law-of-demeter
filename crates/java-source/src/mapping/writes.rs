use std::collections::HashSet;

use crate::{Expression, Statement};

pub(super) fn assigned_in<'a>(
    statements: &[Statement],
    expressions: impl IntoIterator<Item = &'a Expression>,
) -> HashSet<String> {
    let mut writes = Writes::default();
    let declared = HashSet::new();
    writes.statements(statements, &declared);
    for expression in expressions {
        writes.expression(expression, &declared);
    }
    writes.locals
}

#[derive(Default)]
struct Writes {
    locals: HashSet<String>,
}

impl Writes {
    fn statements(&mut self, statements: &[Statement], outer: &HashSet<String>) {
        let mut declared = outer.clone();
        for statement in statements {
            self.statement(statement, &mut declared);
        }
    }

    fn statement(&mut self, statement: &Statement, declared: &mut HashSet<String>) {
        match statement {
            Statement::Local(local) => {
                declared.insert(local.name.clone());
                if let Some(initializer) = &local.initializer {
                    self.expression(initializer, declared);
                }
            }
            Statement::Expression(expression) | Statement::Return(Some(expression)) => {
                self.expression(expression, declared);
            }
            Statement::Return(None) => {}
            Statement::Block(body) => self.statements(body, declared),
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.expression(condition, declared);
                self.statements(then_branch, declared);
                self.statements(else_branch, declared);
            }
            Statement::For {
                init,
                condition,
                update,
                body,
            } => {
                let mut scope = declared.clone();
                for statement in init {
                    self.statement(statement, &mut scope);
                }
                if let Some(condition) = condition {
                    self.expression(condition, &scope);
                }
                self.statements(body, &scope);
                for expression in update {
                    self.expression(expression, &scope);
                }
            }
            Statement::ForEach {
                variable,
                iterable,
                body,
            } => {
                self.expression(iterable, declared);
                let mut scope = declared.clone();
                scope.insert(variable.name.clone());
                self.statements(body, &scope);
            }
            Statement::While { condition, body } | Statement::DoWhile { body, condition } => {
                self.expression(condition, declared);
                self.statements(body, declared);
            }
            Statement::Try {
                resources,
                body,
                catches,
                finally,
            } => {
                let mut scope = declared.clone();
                for resource in resources {
                    self.statement(resource, &mut scope);
                }
                self.statements(body, &scope);
                for caught in catches {
                    let mut scope = declared.clone();
                    scope.insert(caught.name.clone());
                    self.statements(&caught.body, &scope);
                }
                self.statements(finally, declared);
            }
        }
    }

    fn expression(&mut self, expression: &Expression, declared: &HashSet<String>) {
        match expression {
            Expression::Assignment { target, value } => {
                if let Expression::Local(name) = &**target
                    && !declared.contains(name)
                {
                    self.locals.insert(name.clone());
                }
                self.expression(target, declared);
                self.expression(value, declared);
            }
            Expression::Call(call) => {
                if let Some(receiver) = &call.receiver {
                    self.expression(receiver, declared);
                }
                for argument in &call.arguments {
                    self.expression(argument, declared);
                }
            }
            Expression::New(created) => {
                for argument in &created.arguments {
                    self.expression(argument, declared);
                }
            }
            Expression::Operation(operands) => {
                for operand in operands {
                    self.expression(operand, declared);
                }
            }
            Expression::FieldAccess { target, .. } | Expression::MethodReference { target, .. } => {
                self.expression(target, declared)
            }
            Expression::Cast { operand, .. } | Expression::InstanceOf { operand, .. } => {
                self.expression(operand, declared)
            }
            Expression::Switch(switch) => {
                self.expression(&switch.selector, declared);
                for branch in &switch.branches {
                    let mut scope = declared.clone();
                    scope.extend(branch.bindings.iter().map(|binding| binding.name.clone()));
                    if let Some(guard) = &branch.guard {
                        self.expression(guard, &scope);
                    }
                    self.statements(&branch.body, &scope);
                }
            }
            Expression::Lambda(_)
            | Expression::This
            | Expression::Super
            | Expression::Parameter(_)
            | Expression::Local(_)
            | Expression::LambdaParameter(_)
            | Expression::Captured(_)
            | Expression::Field(_)
            | Expression::OuterField { .. }
            | Expression::Type(_)
            | Expression::StaticImport { .. }
            | Expression::OnDemandStaticImport { .. }
            | Expression::Unresolved(_)
            | Expression::ClassLiteral(_)
            | Expression::Literal(_) => {}
        }
    }
}
