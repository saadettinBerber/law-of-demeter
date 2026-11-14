use law_of_demeter_core::domain::method::Expression as Value;
use law_of_demeter_core::domain::type_shape::TypeRef;

use crate::{Catch, Expression, LocalVariable, Statement};

use super::MethodMapper;
use super::types::{type_ref, type_ref_or_object};

impl MethodMapper<'_> {
    pub(super) fn statements(&mut self, statements: &[Statement]) {
        for statement in statements {
            self.statement(statement);
        }
    }

    fn statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Local(local) => self.local(local),
            Statement::Expression(expression) | Statement::Return(Some(expression)) => {
                self.evaluate(expression);
            }
            Statement::Return(None) => {}
            Statement::Block(body) => self.statements(body),
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.evaluate(condition);
                self.statements(then_branch);
                self.statements(else_branch);
            }
            Statement::For {
                init,
                condition,
                update,
                body,
            } => {
                self.statements(init);
                if let Some(condition) = condition {
                    self.evaluate(condition);
                }
                self.statements(body);
                for update in update {
                    self.evaluate(update);
                }
            }
            Statement::ForEach {
                variable,
                iterable,
                body,
            } => {
                let iterable = self.expression(iterable);
                self.locals
                    .insert(variable.name.clone(), type_ref(&variable.declared_type));
                self.steps.for_each(&variable.name, iterable);
                self.statements(body);
            }
            Statement::While { condition, body } => {
                self.evaluate(condition);
                self.statements(body);
            }
            Statement::DoWhile { body, condition } => {
                self.statements(body);
                self.evaluate(condition);
            }
            Statement::Try {
                resources,
                body,
                catches,
                finally,
            } => {
                self.statements(resources);
                self.statements(body);
                for caught in catches {
                    self.catch(caught);
                }
                self.statements(finally);
            }
        }
    }

    pub(super) fn evaluate(&mut self, expression: &Expression) {
        let value = self.expression(expression);
        self.steps.evaluate(value);
    }

    fn local(&mut self, local: &LocalVariable) {
        let declared = type_ref(&local.declared_type);
        self.locals.insert(local.name.clone(), declared.clone());
        let Some(initializer) = &local.initializer else {
            return;
        };
        let value = self.expression(initializer);
        self.steps.assign(&local.name, declared, value);
    }

    fn catch(&mut self, caught: &Catch) {
        let declared = match caught.types.as_slice() {
            [only] => type_ref_or_object(only),
            _ => TypeRef::class("java.lang.Throwable"),
        };
        let value = Value::parameter(&caught.name, declared.clone());
        self.locals
            .insert(caught.name.clone(), Some(declared.clone()));
        self.steps.assign(&caught.name, Some(declared), value);
        self.statements(&caught.body);
    }
}
