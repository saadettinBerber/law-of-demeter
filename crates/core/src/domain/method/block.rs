use super::call::Call;
use super::expression::Expression;
use super::reason::{Reason, Verdict};
use super::subject::Context;
use crate::domain::type_shape::TypeRef;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Step {
    Call(Call),
    Assign {
        local: String,
        declared: Option<TypeRef>,
        value: Expression,
    },
    ForEach {
        variable: String,
        iterable: Expression,
    },
    Evaluate(Expression),
}

impl Step {
    pub(super) fn judge(&self, context: &mut Context) -> Vec<Verdict> {
        match self {
            Self::Call(call) => call.judge(context).verdicts,
            Self::Assign {
                local,
                declared,
                value,
            } => {
                let judged = value.judge(context);
                let subject = judged
                    .subject
                    .through(|origin| Reason::local(local, origin));
                context.bind(local, subject.declared_as(declared.as_ref()));
                judged.verdicts
            }
            Self::Evaluate(value) => value.judge(context).verdicts,
            Self::ForEach { variable, iterable } => {
                let judged = iterable.judge(context);
                context.bind(
                    variable,
                    judged
                        .subject
                        .element(|feed| Reason::loop_variable(variable, feed)),
                );
                judged.verdicts
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct Block {
    steps: Vec<Step>,
}

impl Block {
    pub(super) fn call(mut self, call: Call) -> Self {
        self.steps.push(Step::Call(call));
        self
    }

    pub(super) fn assign(mut self, local: &str, type_ref: TypeRef, value: Expression) -> Self {
        self.steps.push(Step::Assign {
            local: local.to_owned(),
            declared: Some(type_ref),
            value,
        });
        self
    }

    pub(super) fn assign_var(mut self, local: &str, value: Expression) -> Self {
        self.steps.push(Step::Assign {
            local: local.to_owned(),
            declared: None,
            value,
        });
        self
    }

    pub(super) fn for_each(mut self, variable: &str, iterable: Expression) -> Self {
        self.steps.push(Step::ForEach {
            variable: variable.to_owned(),
            iterable,
        });
        self
    }

    pub(super) fn evaluate(mut self, value: Expression) -> Self {
        self.steps.push(Step::Evaluate(value));
        self
    }

    pub(super) fn judge(&self, context: &mut Context) -> Vec<Verdict> {
        self.steps
            .iter()
            .flat_map(|step| step.judge(context))
            .collect()
    }
}
