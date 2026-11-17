mod arguments;
mod block;
mod call;
mod expression;
mod reason;
mod subject;

pub use call::Call;
pub use expression::{Argument, Creation, Expression, Lambda, MethodReference};
pub use reason::{Reason, Verdict};

use block::Block;
use subject::{Context, Standing};

use super::classification::Types;
use super::type_shape::TypeRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Method {
    class_name: String,
    name: String,
    line: usize,
    body: Block,
    declared_within: Vec<(String, String)>,
}

impl Method {
    pub fn new(class_name: &str, name: &str, line: usize) -> Self {
        Self {
            class_name: class_name.to_owned(),
            name: name.to_owned(),
            line,
            body: Block::default(),
            declared_within: Vec::new(),
        }
    }

    pub fn class_name(&self) -> &str {
        &self.class_name
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn declared_within(mut self, class: &str, method: &str) -> Self {
        self.declared_within
            .push((class.to_owned(), method.to_owned()));
        self
    }

    pub(crate) fn is_declared_within(&self, class: &str, method: &str) -> bool {
        self.declared_within
            .iter()
            .any(|(outer, enclosing)| outer == class && enclosing == method)
    }

    pub fn call(mut self, call: Call) -> Self {
        self.body = self.body.call(call);
        self
    }

    pub fn assign(mut self, local: &str, type_ref: TypeRef, value: Expression) -> Self {
        self.body = self.body.assign(local, type_ref, value);
        self
    }

    pub fn assign_var(mut self, local: &str, value: Expression) -> Self {
        self.body = self.body.assign_var(local, value);
        self
    }

    pub fn for_each(mut self, variable: &str, iterable: Expression) -> Self {
        self.body = self.body.for_each(variable, iterable);
        self
    }

    pub fn evaluate(mut self, value: Expression) -> Self {
        self.body = self.body.evaluate(value);
        self
    }

    pub fn give_back(mut self, value: Expression) -> Self {
        self.body = self.body.give_back(value);
        self
    }

    pub fn verdicts(&self, types: &Types) -> Vec<Verdict> {
        self.verdicts_among(types, &[])
    }

    pub fn verdicts_among(&self, types: &Types, methods: &[Method]) -> Vec<Verdict> {
        self.body
            .judge(&mut Context::new(types, &self.class_name, methods))
    }

    fn handed_back(&self, mut context: Context) -> Option<Standing> {
        self.body.judge(&mut context);
        let mut reasons = context
            .given_back()
            .into_iter()
            .map(|standing| match standing {
                Standing::Friend(reason) if !reason.rests_on_parameter() => {
                    Some(reason.without_locals())
                }
                _ => None,
            });
        let first = reasons.next().flatten()?;
        reasons
            .all(|reason| reason.is_some())
            .then_some(Standing::Friend(first))
    }
}

#[cfg(test)]
mod compound;
#[cfg(test)]
mod false_alarms;
#[cfg(test)]
mod fluent_varargs;
#[cfg(test)]
mod functional_values;
#[cfg(test)]
mod links;
#[cfg(test)]
mod nested;
#[cfg(test)]
mod references;
#[cfg(test)]
mod resolved;
#[cfg(test)]
mod rule;
#[cfg(test)]
mod scenarios;
#[cfg(test)]
mod stream_standing;
#[cfg(test)]
mod tests;
