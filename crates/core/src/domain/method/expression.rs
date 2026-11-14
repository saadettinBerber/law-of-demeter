use std::collections::HashSet;

use super::block::Block;
use super::call::Call;
use super::reason::{Reason, Verdict};
use super::subject::{Context, Judged, Standing, Subject};
use crate::domain::classification::Types;
use crate::domain::type_shape::TypeRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    This,
    Field {
        name: String,
        type_ref: TypeRef,
    },
    Parameter {
        name: String,
        type_ref: TypeRef,
    },
    ReturnOf(Box<Call>),
    Local(String),
    New(Creation),
    StaticType(TypeRef),
    Literal,
    Operation(Vec<Expression>),
    Cast {
        type_ref: TypeRef,
        operand: Box<Expression>,
    },
    FieldOf {
        target: Box<Expression>,
        name: String,
    },
}

impl Expression {
    pub fn field(name: &str, type_ref: TypeRef) -> Self {
        Self::Field {
            name: name.to_owned(),
            type_ref,
        }
    }

    pub fn parameter(name: &str, type_ref: TypeRef) -> Self {
        Self::Parameter {
            name: name.to_owned(),
            type_ref,
        }
    }

    pub fn static_type(type_ref: TypeRef) -> Self {
        Self::StaticType(type_ref)
    }

    pub fn created(creation: Creation) -> Self {
        Self::New(creation)
    }

    pub fn local(name: &str) -> Self {
        Self::Local(name.to_owned())
    }

    pub fn literal() -> Self {
        Self::Literal
    }

    pub fn return_of(call: Call) -> Self {
        Self::ReturnOf(Box::new(call))
    }

    pub fn operation(operands: Vec<Expression>) -> Self {
        Self::Operation(operands)
    }

    pub fn field_of(target: Expression, name: &str) -> Self {
        Self::FieldOf {
            target: Box::new(target),
            name: name.to_owned(),
        }
    }

    pub fn cast(type_ref: TypeRef, operand: Expression) -> Self {
        Self::Cast {
            type_ref,
            operand: Box::new(operand),
        }
    }

    pub(super) fn judge(&self, context: &Context) -> Judged {
        match self {
            Self::This => Judged::quiet(Subject::friend(Reason::This, context.this_type.clone())),
            Self::Field { name, type_ref } => {
                Judged::quiet(Subject::friend(Reason::field(name), type_ref.clone()))
            }
            Self::Parameter { name, type_ref } => {
                Judged::quiet(Subject::friend(Reason::parameter(name), type_ref.clone()))
            }
            Self::ReturnOf(call) => call.judge(context),
            Self::Local(name) => Judged::quiet(context.local(name)),
            Self::New(creation) => creation.judge(context),
            Self::StaticType(type_ref) => Judged::quiet(Subject::friend(
                Reason::static_type(type_ref.clone()),
                type_ref.clone(),
            )),
            Self::Literal => Judged::quiet(Subject {
                standing: Standing::Friend(Reason::Literal),
                type_ref: None,
            }),
            Self::Operation(operands) => Judged {
                verdicts: operands
                    .iter()
                    .flat_map(|operand| operand.judge(context).verdicts)
                    .collect(),
                subject: Subject {
                    standing: Standing::Friend(Reason::Literal),
                    type_ref: None,
                },
            },
            Self::FieldOf { target, name } => {
                let judged = target.judge(context);
                let standing = if matches!(**target, Self::StaticType(_)) {
                    Standing::Friend(Reason::static_result(name))
                } else {
                    Standing::Stranger(Reason::field_of(name))
                };
                let type_ref = judged
                    .subject
                    .type_ref
                    .as_ref()
                    .and_then(class_name)
                    .and_then(|owner| field_type(context.types, owner, name));
                Judged {
                    verdicts: judged.verdicts,
                    subject: Subject { standing, type_ref },
                }
            }
            Self::Cast { type_ref, operand } => {
                let judged = operand.judge(context);
                Judged {
                    verdicts: judged.verdicts,
                    subject: judged.subject.declared_as(Some(type_ref)),
                }
            }
        }
    }
}

fn class_name(type_ref: &TypeRef) -> Option<&str> {
    match type_ref {
        TypeRef::Class { name, .. } => Some(name),
        TypeRef::Primitive(_) | TypeRef::Array(_) | TypeRef::Variable(_) => None,
    }
}

fn field_type(types: &Types, owner: &str, name: &str) -> Option<TypeRef> {
    let mut pending = vec![owner];
    let mut seen = HashSet::new();
    while let Some(current) = pending.pop() {
        if !seen.insert(current) {
            continue;
        }
        let Some(shape) = types.shape(current) else {
            continue;
        };
        if let Some(field) = shape.fields().iter().find(|field| field.name() == name) {
            return Some(field.type_ref().clone());
        }
        pending.extend(shape.supertypes().iter().filter_map(class_name));
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Creation {
    type_ref: TypeRef,
    arguments: Vec<Argument>,
}

impl Creation {
    pub fn new(type_ref: TypeRef) -> Self {
        Self {
            type_ref,
            arguments: Vec::new(),
        }
    }

    pub fn with(mut self, argument: impl Into<Argument>) -> Self {
        self.arguments.push(argument.into());
        self
    }

    pub(super) fn judge(&self, context: &Context) -> Judged {
        let created = Subject::friend(
            Reason::created(self.type_ref.clone()),
            self.type_ref.clone(),
        );
        let verdicts = self
            .arguments
            .iter()
            .flat_map(|argument| argument.judge(context, &created))
            .collect();
        Judged {
            verdicts,
            subject: created,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Argument {
    Value(Expression),
    Lambda(Lambda),
}

impl From<Expression> for Argument {
    fn from(value: Expression) -> Self {
        Self::Value(value)
    }
}

impl From<Lambda> for Argument {
    fn from(lambda: Lambda) -> Self {
        Self::Lambda(lambda)
    }
}

impl Argument {
    pub(super) fn judge(&self, context: &Context, feed: &Subject) -> Vec<Verdict> {
        match self {
            Self::Value(value) => value.judge(context).verdicts,
            Self::Lambda(lambda) => lambda.judge(context, feed),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lambda {
    parameters: Vec<String>,
    body: Block,
}

impl Lambda {
    pub fn new(parameters: &[&str]) -> Self {
        Self {
            parameters: parameters.iter().map(|name| (*name).to_owned()).collect(),
            body: Block::default(),
        }
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

    pub fn evaluate(mut self, value: Expression) -> Self {
        self.body = self.body.evaluate(value);
        self
    }

    pub fn for_each(mut self, variable: &str, iterable: Expression) -> Self {
        self.body = self.body.for_each(variable, iterable);
        self
    }

    pub(super) fn judge(&self, context: &Context, feed: &Subject) -> Vec<Verdict> {
        let mut inner = context.clone();
        for parameter in &self.parameters {
            inner.bind(
                parameter,
                feed.element(|feed| Reason::lambda_parameter(parameter, feed)),
            );
        }
        self.body.judge(&mut inner)
    }
}
