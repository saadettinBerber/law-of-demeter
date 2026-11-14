use std::collections::HashMap;

use super::classification::{
    Classification, Types, element_type, is_factory_method, is_repository_query,
};
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
    Literal,
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

    fn judge(&self, context: &Context) -> Judged {
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

    fn judge(&self, context: &Context) -> Judged {
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
    fn judge(&self, context: &Context, feed: &Subject) -> Vec<Verdict> {
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

    fn judge(&self, context: &Context, feed: &Subject) -> Vec<Verdict> {
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

    fn judge(&self, context: &Context) -> Judged {
        let receiver = self.receiver.judge(context);
        let mut verdicts = receiver.verdicts;
        for argument in &self.arguments {
            verdicts.extend(argument.judge(context, &receiver.subject));
        }
        verdicts.push(self.verdict(context.types, &receiver.subject));
        let subject = self.result(context.types, &receiver.subject);
        Judged { verdicts, subject }
    }

    fn verdict(&self, types: &Types, receiver: &Subject) -> Verdict {
        match &receiver.standing {
            Standing::Friend(reason) => Verdict::allowed(&self.name, self.line, reason.clone()),
            Standing::Stranger(source) => {
                self.stranger_verdict(types, receiver.type_ref.as_ref(), source)
            }
        }
    }

    fn stranger_verdict(
        &self,
        types: &Types,
        type_ref: Option<&TypeRef>,
        source: &Reason,
    ) -> Verdict {
        let classified = type_ref.map(|type_ref| (types.classify(type_ref), type_ref));
        match classified {
            Some((Classification::Container, type_ref)) if is_mutator(&self.name) => {
                Verdict::violation(
                    &self.name,
                    self.line,
                    Reason::container_mutation(type_ref.clone(), source.clone()),
                )
            }
            Some((
                classification @ (Classification::DataStructure
                | Classification::Value
                | Classification::Container),
                type_ref,
            )) => Verdict::allowed(
                &self.name,
                self.line,
                Reason::classified(classification, type_ref.clone()),
            ),
            Some((_, type_ref)) if self.is_fluent_on(types, type_ref) => {
                Verdict::allowed(&self.name, self.line, Reason::fluent(type_ref.clone()))
            }
            _ => Verdict::violation(&self.name, self.line, source.clone()),
        }
    }

    fn is_fluent_on(&self, types: &Types, type_ref: &TypeRef) -> bool {
        class_name(type_ref)
            .is_some_and(|name| types.is_fluent(name, &self.name, self.arguments.len()))
    }

    fn result(&self, types: &Types, receiver: &Subject) -> Subject {
        let type_ref = receiver
            .type_ref
            .as_ref()
            .and_then(class_name)
            .and_then(|name| types.return_type(name, &self.name, self.arguments.len()))
            .cloned();
        Subject {
            standing: self.result_standing(types, receiver),
            type_ref,
        }
    }

    fn result_standing(&self, types: &Types, receiver: &Subject) -> Standing {
        let standing = &receiver.standing;
        let is_fluent = receiver
            .type_ref
            .as_ref()
            .is_some_and(|type_ref| self.is_fluent_on(types, type_ref));
        if matches!(self.receiver, Expression::StaticType(_)) {
            Standing::Friend(Reason::static_result(&self.name))
        } else if is_fluent {
            standing.clone()
        } else if is_factory_method(&self.name) && matches!(standing, Standing::Friend(_)) {
            Standing::Friend(Reason::factory_result(&self.name))
        } else if is_repository_query(&self.name) && standing.is_held() {
            Standing::Friend(Reason::query_result(&self.name))
        } else {
            Standing::Stranger(Reason::return_of(&self.name))
        }
    }
}

const MUTATOR_WORDS: [&str; 14] = [
    "add", "remove", "retain", "clear", "put", "set", "push", "pop", "poll", "offer", "sort",
    "replace", "merge", "compute",
];

fn is_mutator(name: &str) -> bool {
    MUTATOR_WORDS.iter().any(|word| {
        name.strip_prefix(word)
            .is_some_and(|rest| !rest.starts_with(char::is_lowercase))
    })
}

fn class_name(type_ref: &TypeRef) -> Option<&str> {
    match type_ref {
        TypeRef::Class { name, .. } => Some(name),
        TypeRef::Primitive(_) | TypeRef::Array(_) | TypeRef::Variable(_) => None,
    }
}

#[derive(Debug, Clone)]
enum Standing {
    Friend(Reason),
    Stranger(Reason),
}

impl Standing {
    fn is_held(&self) -> bool {
        matches!(self, Self::Friend(reason) if reason.is_held())
    }

    fn through(self, wrap: impl FnOnce(Reason) -> Reason) -> Self {
        match self {
            Self::Friend(reason) => Self::Friend(wrap(reason)),
            Self::Stranger(reason) => Self::Stranger(wrap(reason)),
        }
    }
}

#[derive(Debug, Clone)]
struct Subject {
    standing: Standing,
    type_ref: Option<TypeRef>,
}

impl Subject {
    fn friend(reason: Reason, type_ref: TypeRef) -> Self {
        Self {
            standing: Standing::Friend(reason),
            type_ref: Some(type_ref),
        }
    }

    fn through(self, wrap: impl FnOnce(Reason) -> Reason) -> Self {
        Self {
            standing: self.standing.through(wrap),
            type_ref: self.type_ref,
        }
    }

    fn declared_as(self, declared: Option<&TypeRef>) -> Self {
        Self {
            type_ref: declared.cloned().or(self.type_ref),
            ..self
        }
    }

    fn element(&self, wrap: impl FnOnce(Reason) -> Reason) -> Self {
        Self {
            standing: self.standing.clone().through(wrap),
            type_ref: self.type_ref.as_ref().and_then(element_type).cloned(),
        }
    }
}

struct Judged {
    verdicts: Vec<Verdict>,
    subject: Subject,
}

impl Judged {
    fn quiet(subject: Subject) -> Self {
        Self {
            verdicts: Vec::new(),
            subject,
        }
    }
}

#[derive(Clone)]
struct Context<'a> {
    types: &'a Types,
    this_type: TypeRef,
    locals: HashMap<String, Subject>,
}

impl<'a> Context<'a> {
    fn new(types: &'a Types, class_name: &str) -> Self {
        Self {
            types,
            this_type: TypeRef::class(class_name),
            locals: HashMap::new(),
        }
    }

    fn bind(&mut self, name: &str, subject: Subject) {
        self.locals.insert(name.to_owned(), subject);
    }

    fn local(&self, name: &str) -> Subject {
        self.locals.get(name).cloned().unwrap_or_else(|| Subject {
            standing: Standing::Stranger(Reason::unassigned_local(name)),
            type_ref: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    This,
    Literal,
    Field(String),
    Parameter(String),
    ReturnOf(String),
    UnassignedLocal(String),
    StaticResult(String),
    FactoryResult(String),
    QueryResult(String),
    Fluent(TypeRef),
    ContainerMutation {
        type_ref: TypeRef,
        source: Box<Reason>,
    },
    Created(TypeRef),
    StaticType(TypeRef),
    Local {
        name: String,
        origin: Box<Reason>,
    },
    LambdaParameter {
        name: String,
        feed: Box<Reason>,
    },
    LoopVariable {
        name: String,
        feed: Box<Reason>,
    },
    Classified {
        classification: Classification,
        type_ref: TypeRef,
    },
}

impl Reason {
    fn is_held(&self) -> bool {
        match self {
            Self::Field(_) | Self::Parameter(_) => true,
            Self::Local { origin, .. } => origin.is_held(),
            _ => false,
        }
    }

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

    pub fn static_result(call: &str) -> Self {
        Self::StaticResult(call.to_owned())
    }

    pub fn factory_result(call: &str) -> Self {
        Self::FactoryResult(call.to_owned())
    }

    pub fn query_result(call: &str) -> Self {
        Self::QueryResult(call.to_owned())
    }

    pub fn container_mutation(type_ref: TypeRef, source: Reason) -> Self {
        Self::ContainerMutation {
            type_ref,
            source: Box::new(source),
        }
    }

    pub fn fluent(type_ref: TypeRef) -> Self {
        Self::Fluent(type_ref)
    }

    pub fn classified(classification: Classification, type_ref: TypeRef) -> Self {
        Self::Classified {
            classification,
            type_ref,
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
        declared: Option<TypeRef>,
        value: Expression,
    },
    ForEach {
        variable: String,
        iterable: Expression,
    },
}

impl Step {
    fn judge(&self, context: &mut Context) -> Vec<Verdict> {
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
struct Block {
    steps: Vec<Step>,
}

impl Block {
    fn call(mut self, call: Call) -> Self {
        self.steps.push(Step::Call(call));
        self
    }

    fn assign(mut self, local: &str, type_ref: TypeRef, value: Expression) -> Self {
        self.steps.push(Step::Assign {
            local: local.to_owned(),
            declared: Some(type_ref),
            value,
        });
        self
    }

    fn assign_var(mut self, local: &str, value: Expression) -> Self {
        self.steps.push(Step::Assign {
            local: local.to_owned(),
            declared: None,
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

    fn judge(&self, context: &mut Context) -> Vec<Verdict> {
        self.steps
            .iter()
            .flat_map(|step| step.judge(context))
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

    pub fn verdicts(&self, types: &Types) -> Vec<Verdict> {
        self.body.judge(&mut Context::new(types, &self.class_name))
    }
}

#[cfg(test)]
mod scenarios;
#[cfg(test)]
mod tests;
