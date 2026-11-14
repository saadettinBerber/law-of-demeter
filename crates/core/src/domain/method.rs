use std::collections::HashMap;

use super::classification::Types;
use super::type_shape::TypeRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    This,
    Field { name: String, type_ref: TypeRef },
    Parameter { name: String, type_ref: TypeRef },
    ReturnOf(Box<Call>),
    Local(String),
    New(Creation),
    StaticType(TypeRef),
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

    pub fn return_of(call: Call) -> Self {
        Self::ReturnOf(Box::new(call))
    }

    fn verdicts(&self, scope: &Scope) -> Vec<Verdict> {
        match self {
            Self::ReturnOf(call) => call.verdicts(scope),
            Self::New(creation) => creation.verdicts(scope),
            Self::This
            | Self::Field { .. }
            | Self::Parameter { .. }
            | Self::Local(_)
            | Self::StaticType(_) => Vec::new(),
        }
    }

    fn standing(&self, scope: &Scope) -> Standing {
        match self {
            Self::This => Standing::Friend(Reason::This),
            Self::Field { name, .. } => Standing::Friend(Reason::field(name)),
            Self::Parameter { name, .. } => Standing::Friend(Reason::parameter(name)),
            Self::ReturnOf(call) => Standing::Stranger(Reason::return_of(&call.name)),
            Self::New(creation) => Standing::Friend(Reason::created(creation.type_ref.clone())),
            Self::StaticType(type_ref) => Standing::Friend(Reason::static_type(type_ref.clone())),
            Self::Local(name) => scope
                .get(name)
                .cloned()
                .unwrap_or_else(|| Standing::Stranger(Reason::unassigned_local(name))),
        }
    }
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

    fn verdicts(&self, scope: &Scope) -> Vec<Verdict> {
        let feed = Standing::Friend(Reason::created(self.type_ref.clone()));
        self.arguments
            .iter()
            .flat_map(|argument| argument.verdicts(scope, &feed))
            .collect()
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
    fn verdicts(&self, scope: &Scope, feed: &Standing) -> Vec<Verdict> {
        match self {
            Self::Value(value) => value.verdicts(scope),
            Self::Lambda(lambda) => lambda.verdicts(scope, feed),
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

    pub fn assign(mut self, local: &str, value: Expression) -> Self {
        self.body = self.body.assign(local, value);
        self
    }

    fn verdicts(&self, scope: &Scope, feed: &Standing) -> Vec<Verdict> {
        let mut scope = scope.clone();
        for parameter in &self.parameters {
            let standing = feed
                .clone()
                .through(|feed| Reason::lambda_parameter(parameter, feed));
            scope.insert(parameter.clone(), standing);
        }
        self.body.verdicts(&mut scope)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    receiver: Expression,
    name: String,
    line: usize,
    arguments: Vec<Argument>,
}

impl Call {
    pub fn new(receiver: Expression, name: &str, line: usize) -> Self {
        Self {
            receiver,
            name: name.to_owned(),
            line,
            arguments: Vec::new(),
        }
    }

    pub fn with(mut self, argument: impl Into<Argument>) -> Self {
        self.arguments.push(argument.into());
        self
    }

    pub fn then(self, name: &str, line: usize) -> Self {
        Self::new(Expression::return_of(self), name, line)
    }

    fn verdicts(&self, scope: &Scope) -> Vec<Verdict> {
        let receiver = self.receiver.standing(scope);
        let mut verdicts = self.receiver.verdicts(scope);
        verdicts.extend(
            self.arguments
                .iter()
                .flat_map(|argument| argument.verdicts(scope, &receiver)),
        );
        verdicts.push(receiver.verdict(&self.name, self.line));
        verdicts
    }
}

type Scope = HashMap<String, Standing>;

#[derive(Debug, Clone)]
enum Standing {
    Friend(Reason),
    Stranger(Reason),
}

impl Standing {
    fn verdict(self, call: &str, line: usize) -> Verdict {
        match self {
            Self::Friend(reason) => Verdict::allowed(call, line, reason),
            Self::Stranger(reason) => Verdict::violation(call, line, reason),
        }
    }

    fn through(self, wrap: impl FnOnce(Reason) -> Reason) -> Self {
        match self {
            Self::Friend(reason) => Self::Friend(wrap(reason)),
            Self::Stranger(reason) => Self::Stranger(wrap(reason)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    This,
    Field(String),
    Parameter(String),
    ReturnOf(String),
    UnassignedLocal(String),
    Created(TypeRef),
    StaticType(TypeRef),
    Local { name: String, origin: Box<Reason> },
    LambdaParameter { name: String, feed: Box<Reason> },
    LoopVariable { name: String, feed: Box<Reason> },
}

impl Reason {
    pub fn field(name: &str) -> Self {
        Self::Field(name.to_owned())
    }

    pub fn parameter(name: &str) -> Self {
        Self::Parameter(name.to_owned())
    }

    pub fn return_of(call: &str) -> Self {
        Self::ReturnOf(call.to_owned())
    }

    pub fn local(name: &str, origin: Reason) -> Self {
        Self::Local {
            name: name.to_owned(),
            origin: Box::new(origin),
        }
    }

    pub fn lambda_parameter(name: &str, feed: Reason) -> Self {
        Self::LambdaParameter {
            name: name.to_owned(),
            feed: Box::new(feed),
        }
    }

    pub fn loop_variable(name: &str, feed: Reason) -> Self {
        Self::LoopVariable {
            name: name.to_owned(),
            feed: Box::new(feed),
        }
    }

    pub fn created(type_ref: TypeRef) -> Self {
        Self::Created(type_ref)
    }

    pub fn static_type(type_ref: TypeRef) -> Self {
        Self::StaticType(type_ref)
    }

    pub fn unassigned_local(name: &str) -> Self {
        Self::UnassignedLocal(name.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Allowed {
        call: String,
        line: usize,
        reason: Reason,
    },
    Violation {
        call: String,
        line: usize,
        reason: Reason,
    },
}

impl Verdict {
    pub fn allowed(call: &str, line: usize, reason: Reason) -> Self {
        Self::Allowed {
            call: call.to_owned(),
            line,
            reason,
        }
    }

    pub fn violation(call: &str, line: usize, reason: Reason) -> Self {
        Self::Violation {
            call: call.to_owned(),
            line,
            reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Step {
    Call(Call),
    Assign {
        local: String,
        value: Expression,
    },
    ForEach {
        variable: String,
        iterable: Expression,
    },
}

impl Step {
    fn verdicts(&self, scope: &mut Scope) -> Vec<Verdict> {
        match self {
            Self::Call(call) => call.verdicts(scope),
            Self::Assign { local, value } => {
                let verdicts = value.verdicts(scope);
                let standing = value
                    .standing(scope)
                    .through(|origin| Reason::local(local, origin));
                scope.insert(local.clone(), standing);
                verdicts
            }
            Self::ForEach { variable, iterable } => {
                let verdicts = iterable.verdicts(scope);
                let standing = iterable
                    .standing(scope)
                    .through(|feed| Reason::loop_variable(variable, feed));
                scope.insert(variable.clone(), standing);
                verdicts
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Block {
    steps: Vec<Step>,
}

impl Block {
    fn call(mut self, call: Call) -> Self {
        self.steps.push(Step::Call(call));
        self
    }

    fn assign(mut self, local: &str, value: Expression) -> Self {
        self.steps.push(Step::Assign {
            local: local.to_owned(),
            value,
        });
        self
    }

    fn for_each(mut self, variable: &str, iterable: Expression) -> Self {
        self.steps.push(Step::ForEach {
            variable: variable.to_owned(),
            iterable,
        });
        self
    }

    fn verdicts(&self, scope: &mut Scope) -> Vec<Verdict> {
        self.steps
            .iter()
            .flat_map(|step| step.verdicts(scope))
            .collect()
    }
}

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

    pub fn assign(mut self, local: &str, value: Expression) -> Self {
        self.body = self.body.assign(local, value);
        self
    }

    pub fn for_each(mut self, variable: &str, iterable: Expression) -> Self {
        self.body = self.body.for_each(variable, iterable);
        self
    }

    pub fn verdicts(&self, _types: &Types) -> Vec<Verdict> {
        self.body.verdicts(&mut Scope::new())
    }
}

#[cfg(test)]
mod tests;
