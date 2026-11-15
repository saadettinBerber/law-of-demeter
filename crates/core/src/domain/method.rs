mod block;
mod call;
mod expression;
mod reason;
mod subject;

pub use call::Call;
pub use expression::{Argument, Creation, Expression, Lambda, MethodReference};
pub use reason::{Reason, Verdict};

use block::Block;
use subject::Context;

use super::classification::Types;
use super::type_shape::TypeRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Method {
    class_name: String,
    name: String,
    line: usize,
    body: Block,
}

impl Method {
    pub fn new(class_name: &str, name: &str, line: usize) -> Self {
        Self {
            class_name: class_name.to_owned(),
            name: name.to_owned(),
            line,
            body: Block::default(),
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

    pub fn verdicts(&self, types: &Types) -> Vec<Verdict> {
        self.body.judge(&mut Context::new(types, &self.class_name))
    }
}

#[cfg(test)]
mod calibration;
#[cfg(test)]
mod compound;
#[cfg(test)]
mod inference;
#[cfg(test)]
mod scenarios;
#[cfg(test)]
mod tests;
