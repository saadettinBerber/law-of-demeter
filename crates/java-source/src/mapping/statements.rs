use std::collections::HashSet;

use law_of_demeter_core::domain::method::Expression as Value;
use law_of_demeter_core::domain::type_shape::TypeRef;

use crate::{Catch, Expression, LocalVariable, Statement};

use super::MethodMapper;
use super::expressions::expecting;
use super::writes::assigned_in;

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
                let outer: HashSet<_> = self.locals.keys().cloned().collect();
                self.statements(init);
                let written = assigned_in(body, condition.iter().chain(update));
                let invariants = self.unwritten_locals(&written);
                self.exact_locals = invariants.clone();
                if let Some(condition) = condition {
                    self.evaluate(condition);
                }
                self.statements(body);
                for update in update {
                    self.evaluate(update);
                }
                self.exact_locals = invariants;
                self.exact_locals.retain(|name| outer.contains(name));
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
                self.exact_locals.remove(&variable.name);
                let invariants = self.unwritten_locals(&assigned_in(body, []));
                self.exact_locals = invariants.clone();
                self.statements(body);
                self.exact_locals = invariants;
            }
            Statement::While { condition, body } => {
                let invariants = self.unwritten_locals(&assigned_in(body, [condition]));
                self.exact_locals = invariants.clone();
                self.evaluate(condition);
                self.statements(body);
                self.exact_locals = invariants;
            }
            Statement::DoWhile { body, condition } => {
                let invariants = self.unwritten_locals(&assigned_in(body, [condition]));
                self.exact_locals = invariants.clone();
                self.statements(body);
                self.evaluate(condition);
                self.exact_locals = invariants;
            }
            Statement::Try {
                resources,
                body,
                catches,
                finally,
            } => {
                let outer: HashSet<_> = self.locals.keys().cloned().collect();
                let before = self.exact_locals.clone();
                let mut written = assigned_in(resources, []);
                written.extend(assigned_in(body, []));
                let catch_start: HashSet<_> = before.difference(&written).cloned().collect();
                self.statements(resources);
                self.statements(body);
                let mut continuing = self.exact_locals.clone();
                continuing.retain(|name| outer.contains(name));
                for caught in catches {
                    self.exact_locals = catch_start.clone();
                    self.catch(caught);
                    continuing.retain(|name| self.exact_locals.contains(name));
                    written.extend(assigned_in(&caught.body, []));
                }
                self.exact_locals = before.difference(&written).cloned().collect();
                self.statements(finally);
                let written = assigned_in(finally, []);
                continuing.retain(|name| !written.contains(name));
                continuing.extend(
                    self.exact_locals
                        .iter()
                        .filter(|name| written.contains(*name) && outer.contains(*name))
                        .cloned(),
                );
                self.exact_locals = continuing;
            }
        }
    }

    pub(super) fn evaluate(&mut self, expression: &Expression) {
        let value = self.expression(expression);
        self.steps.evaluate(value);
    }

    pub(super) fn unwritten_locals(&self, written: &HashSet<String>) -> HashSet<String> {
        self.exact_locals.difference(written).cloned().collect()
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
        self.exact_locals.remove(&caught.name);
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
