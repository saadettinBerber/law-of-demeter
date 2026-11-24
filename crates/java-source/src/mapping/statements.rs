use law_of_demeter_core::domain::method::Expression as Value;
use law_of_demeter_core::domain::type_shape::TypeRef;

use crate::{Catch, Expression, LocalVariable, Statement};

use super::MethodMapper;
use super::expressions::expecting;

impl MethodMapper<'_> {
    pub(super) fn statements(&mut self, statements: &[Statement]) {
        for statement in statements {
            self.statement(statement);
        }
    }

    fn statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Local(local) => self.local(local),
            Statement::Expression(expression) => self.evaluate(expression),
            Statement::Return(Some(expression)) => self.returned(expression),
            Statement::Return(None) => {}
            Statement::Block(body) => self.statements(body),
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.evaluate(condition);
                let before = self.exact_locals.clone();
                self.statements(then_branch);
                let after_then = self.exact_locals.clone();
                self.exact_locals = before;
                self.statements(else_branch);
                self.exact_locals.retain(|name| after_then.contains(name));
            }
            Statement::For {
                init,
                condition,
                update,
                body,
            } => {
                self.statements(init);
                self.exact_locals.clear();
                if let Some(condition) = condition {
                    self.evaluate(condition);
                }
                self.statements(body);
                for update in update {
                    self.evaluate(update);
                }
                self.exact_locals.clear();
            }
            Statement::ForEach {
                variable,
                iterable,
                body,
            } => {
                let iterable = self.expression(iterable);
                self.locals.insert(
                    variable.name.clone(),
                    self.type_ref(&variable.declared_type),
                );
                self.steps.for_each(&variable.name, iterable);
                self.exact_locals.clear();
                self.statements(body);
                self.exact_locals.clear();
            }
            Statement::While { condition, body } => {
                self.exact_locals.clear();
                self.evaluate(condition);
                self.statements(body);
                self.exact_locals.clear();
            }
            Statement::DoWhile { body, condition } => {
                self.exact_locals.clear();
                self.statements(body);
                self.evaluate(condition);
                self.exact_locals.clear();
            }
            Statement::Try {
                resources,
                body,
                catches,
                finally,
            } => {
                self.statements(resources);
                self.exact_locals.clear();
                self.statements(body);
                for caught in catches {
                    self.exact_locals.clear();
                    self.catch(caught);
                }
                self.exact_locals.clear();
                self.statements(finally);
                self.exact_locals.clear();
            }
        }
    }

    pub(super) fn evaluate(&mut self, expression: &Expression) {
        let value = self.expression(expression);
        self.steps.evaluate(value);
    }

    fn returned(&mut self, expression: &Expression) {
        self.record_yield(expression);
        let returns = self.returns.clone();
        let value = self.expression_expecting(expression, returns.as_ref());
        let value = match returns {
            Some(type_ref) => expecting(value, type_ref),
            None => value,
        };
        if self.method.is_private {
            self.steps.give_back(value);
        } else {
            self.steps.evaluate(value);
        }
    }

    fn local(&mut self, local: &LocalVariable) {
        let declared = self.type_ref(&local.declared_type);
        self.locals.insert(local.name.clone(), declared.clone());
        self.exact_locals.remove(&local.name);
        let Some(initializer) = &local.initializer else {
            return;
        };
        let value = self.expression_expecting(initializer, declared.as_ref());
        self.record_local_identity(&local.name, declared.as_ref(), initializer);
        self.steps.assign(&local.name, declared, value);
    }

    fn catch(&mut self, caught: &Catch) {
        let declared = match caught.types.as_slice() {
            [only] => self.type_ref_or_object(only),
            _ => TypeRef::class("java.lang.Throwable"),
        };
        let value = Value::parameter(&caught.name, declared.clone());
        self.locals
            .insert(caught.name.clone(), Some(declared.clone()));
        self.steps.assign(&caught.name, Some(declared), value);
        self.statements(&caught.body);
    }
}
