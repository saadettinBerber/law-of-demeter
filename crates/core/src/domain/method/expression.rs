use std::collections::HashSet;

use super::arguments::{judge_arguments, judge_values, value_types};
use super::block::Block;
use super::call::{Call, Called, standing_after_call, verdict_on};
use super::reason::{Reason, Verdict};
use super::subject::{Context, Judged, Standing, Subject, Target};
use crate::domain::classification::{FunctionalMethod, Types};
use crate::domain::type_shape::{FieldShape, TypeRef};

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
    Unresolved {
        name: String,
        candidates: Vec<TypeRef>,
    },
    Lambda(Box<Lambda>),
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

    pub fn unresolved(name: &str) -> Self {
        Self::unresolved_among(name, Vec::new())
    }

    pub fn unresolved_among(name: &str, candidates: Vec<TypeRef>) -> Self {
        Self::Unresolved {
            name: name.to_owned(),
            candidates,
        }
    }

    pub fn field_of(target: Expression, name: &str) -> Self {
        Self::FieldOf {
            target: Box::new(target),
            name: name.to_owned(),
        }
    }

    pub fn lambda(lambda: Lambda) -> Self {
        Self::Lambda(Box::new(lambda))
    }

    pub fn cast(type_ref: TypeRef, operand: Expression) -> Self {
        Self::Cast {
            type_ref,
            operand: Box::new(operand),
        }
    }

    pub(super) fn judge(&self, context: &Context) -> Judged {
        self.judge_expecting(context, None)
    }

    pub(super) fn judge_expecting(&self, context: &Context, target: Option<Target>) -> Judged {
        match self {
            Self::This => Judged::quiet(Subject::friend(Reason::This, context.this_type.clone())),
            Self::Field { name, type_ref } => {
                Judged::quiet(Subject::friend(Reason::field(name), type_ref.clone()))
            }
            Self::Parameter { name, type_ref } => {
                Judged::quiet(Subject::friend(Reason::parameter(name), type_ref.clone()))
            }
            Self::ReturnOf(call) => call.judge_expecting(context, target),
            Self::Local(name) => Judged::quiet(context.local(name)),
            Self::New(creation) => creation.judge_expecting(context, target),
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
            Self::Unresolved { name, candidates } => judge_unresolved(context, name, candidates),
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
            Self::Lambda(lambda) => lambda.judge_value(context, target),
            Self::Cast { type_ref, operand } => {
                let judged = operand.judge_expecting(context, Some(Target::of(type_ref)));
                Judged {
                    verdicts: judged.verdicts,
                    subject: judged.subject.declared_as(Some(type_ref)),
                }
            }
        }
    }
}

fn element_feed<'a>(
    types: &Types,
    outer: &'a Subject,
    feed: &'a Subject,
    declared: Option<&TypeRef>,
) -> &'a Subject {
    let element = outer
        .type_ref
        .as_ref()
        .and_then(|type_ref| types.element_type_of(type_ref));
    match element.as_ref().zip(declared) {
        Some((element, declared)) if element == declared => outer,
        _ => feed,
    }
}

fn class_name(type_ref: &TypeRef) -> Option<&str> {
    match type_ref {
        TypeRef::Class { name, .. } => Some(name),
        TypeRef::Primitive(_) | TypeRef::Array(_) | TypeRef::Variable(_) => None,
    }
}

fn judge_unresolved(context: &Context, name: &str, candidates: &[TypeRef]) -> Judged {
    let own =
        class_name(&context.this_type).and_then(|owner| field_type(context.types, owner, name));
    if let Some(type_ref) = own {
        return Judged::quiet(Subject::friend(Reason::field(name), type_ref));
    }
    let declaring = candidates
        .iter()
        .find(|candidate| declares_static_field(context.types, candidate, name));
    match declaring {
        Some(owner) => {
            Expression::field_of(Expression::static_type(owner.clone()), name).judge(context)
        }
        None => Judged::quiet(Subject {
            standing: Standing::Stranger(Reason::unresolved_name(name)),
            type_ref: None,
        }),
    }
}

fn declares_static_field(types: &Types, owner: &TypeRef, name: &str) -> bool {
    class_name(owner)
        .and_then(|owner| find_field(types, owner, name))
        .is_some_and(FieldShape::is_static)
}

fn field_type(types: &Types, owner: &str, name: &str) -> Option<TypeRef> {
    find_field(types, owner, name).map(|field| field.type_ref().clone())
}

fn find_field<'t>(types: &'t Types, owner: &str, name: &str) -> Option<&'t FieldShape> {
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
            return Some(field);
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

    pub(super) fn judge_expecting(&self, context: &Context, target: Option<Target>) -> Judged {
        let created = Subject::friend(
            Reason::created(self.type_ref.clone()),
            self.type_ref.clone(),
        );
        let values = judge_values(context, &self.arguments);
        let mut invocation = context
            .types
            .construction(&self.type_ref, &value_types(&values))
            .expecting(context.types, target.map(|target| target.type_ref));
        let judged = judge_arguments(
            context,
            &self.arguments,
            values,
            &mut invocation,
            &created,
            &created,
        );
        Judged {
            verdicts: judged.verdicts,
            subject: created,
        }
    }
}

pub(super) struct Produced {
    pub(super) verdicts: Vec<Verdict>,
    pub(super) result: Option<TypeRef>,
    pub(super) standing: Option<Standing>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Argument {
    Value(Expression),
    Lambda(Lambda),
    MethodReference(MethodReference),
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

impl From<MethodReference> for Argument {
    fn from(reference: MethodReference) -> Self {
        Self::MethodReference(reference)
    }
}

impl Argument {
    pub(super) fn judge_value(&self, context: &Context) -> Option<Judged> {
        self.judge_value_expecting(context, None)
    }

    pub(super) fn judge_value_expecting(
        &self,
        context: &Context,
        target: Option<Target>,
    ) -> Option<Judged> {
        match self {
            Self::Value(value) => Some(value.judge_expecting(context, target)),
            Self::Lambda(_) | Self::MethodReference(_) => None,
        }
    }

    pub(super) fn judge(
        &self,
        context: &Context,
        feed: &Subject,
        functional: Option<&FunctionalMethod>,
        outer: &Subject,
    ) -> Produced {
        match self {
            Self::Value(value) => {
                let judged = value.judge(context);
                Produced {
                    verdicts: judged.verdicts,
                    result: judged.subject.type_ref,
                    standing: Some(judged.subject.standing),
                }
            }
            Self::Lambda(lambda) => lambda.judge(context, feed, functional, outer),
            Self::MethodReference(reference) => reference.judge(context, feed, functional, outer),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodReference {
    target: Expression,
    name: String,
    line: usize,
}

impl MethodReference {
    pub fn new(target: Expression, name: &str, line: usize) -> Self {
        Self {
            target,
            name: name.to_owned(),
            line,
        }
    }

    fn judge(
        &self,
        context: &Context,
        feed: &Subject,
        functional: Option<&FunctionalMethod>,
        outer: &Subject,
    ) -> Produced {
        let types = context.types;
        let target = self.target.judge(context);
        let mut verdicts = target.verdicts;
        let arity = functional.map_or(0, |functional| functional.parameters.len());
        let result = match &self.target {
            _ if self.name == "new" => functional.map(|_| self.constructed(types, &target.subject)),
            Expression::StaticType(owner) if types.declares_static(owner, &self.name) => {
                verdicts.push(verdict_on(context, &target.subject, &self.called(arity)));
                functional.map(|_| self.static_result(types, owner, arity))
            }
            Expression::StaticType(owner) => {
                let element = self.element(types, feed, outer, functional);
                let receiver = element
                    .clone()
                    .through(|feed| Reason::method_reference(owner.clone(), &self.name, feed));
                let arguments = arity.saturating_sub(1);
                verdicts.push(verdict_on(context, &receiver, &self.called(arguments)));
                functional.map(|_| self.on(types, &element, arguments))
            }
            _ => {
                verdicts.push(verdict_on(context, &target.subject, &self.called(arity)));
                functional.map(|_| self.on(types, &target.subject, arity))
            }
        };
        Produced {
            verdicts,
            result: result.as_ref().and_then(|subject| subject.type_ref.clone()),
            standing: result.map(|subject| subject.standing),
        }
    }

    fn called(&self, argument_count: usize) -> Called<'_> {
        Called {
            name: &self.name,
            line: self.line,
            argument_count,
        }
    }

    fn element(
        &self,
        types: &Types,
        feed: &Subject,
        outer: &Subject,
        functional: Option<&FunctionalMethod>,
    ) -> Subject {
        let known = functional.and_then(|functional| functional.parameters.first());
        element_feed(types, outer, feed, known)
            .element(types, |reason| reason)
            .declared_as(known)
    }

    fn constructed(&self, types: &Types, target: &Subject) -> Subject {
        match &self.target {
            Expression::StaticType(created) => {
                Subject::friend(Reason::created(created.clone()), created.clone())
            }
            _ => self.on(types, target, 0),
        }
    }

    fn static_result(&self, types: &Types, owner: &TypeRef, arguments: usize) -> Subject {
        types
            .return_type_on(owner, &self.name, arguments)
            .map(|type_ref| Subject::friend(Reason::static_result(&self.name), type_ref))
            .unwrap_or_else(|| self.unresolved())
    }

    fn on(&self, types: &Types, target: &Subject, arguments: usize) -> Subject {
        Subject {
            standing: standing_after_call(types, target, &self.name, arguments, false),
            type_ref: target
                .type_ref
                .as_ref()
                .and_then(|target_type| types.return_type_on(target_type, &self.name, arguments)),
        }
    }

    fn unresolved(&self) -> Subject {
        Subject {
            standing: Standing::Stranger(Reason::return_of(&self.name)),
            type_ref: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lambda {
    parameters: Vec<String>,
    body: Block,
    result: Option<Expression>,
    expected: Option<TypeRef>,
    declared: Vec<(String, TypeRef)>,
    resolved: Vec<(String, TypeRef)>,
}

impl Lambda {
    pub fn new(parameters: &[&str]) -> Self {
        Self {
            parameters: parameters.iter().map(|name| (*name).to_owned()).collect(),
            body: Block::default(),
            result: None,
            expected: None,
            declared: Vec::new(),
            resolved: Vec::new(),
        }
    }

    pub fn declaring(mut self, parameter: &str, type_ref: TypeRef) -> Self {
        self.declared.push((parameter.to_owned(), type_ref));
        self
    }

    pub fn resolving(mut self, parameter: &str, type_ref: TypeRef) -> Self {
        self.resolved.push((parameter.to_owned(), type_ref));
        self
    }

    pub fn expecting(mut self, type_ref: TypeRef) -> Self {
        self.expected = Some(type_ref);
        self
    }

    pub fn yielding(mut self, value: Expression) -> Self {
        self.result = Some(value);
        self
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

    pub(super) fn judge(
        &self,
        context: &Context,
        feed: &Subject,
        functional: Option<&FunctionalMethod>,
        outer: &Subject,
    ) -> Produced {
        let declared = functional
            .map(|functional| functional.parameters.as_slice())
            .unwrap_or_default();
        let mut inner = context.clone();
        for (index, parameter) in self.parameters.iter().enumerate() {
            let known = self.source_type(parameter).or(declared.get(index));
            let source = element_feed(context.types, outer, feed, known);
            let subject = source.element(context.types, |feed| {
                Reason::lambda_parameter(parameter, feed)
            });
            inner.bind(parameter, subject.declared_as(known));
        }
        let (verdicts, yielded) = self.judge_body(inner);
        Produced {
            verdicts,
            result: yielded
                .as_ref()
                .and_then(|subject| subject.type_ref.clone()),
            standing: yielded.map(|subject| subject.standing),
        }
    }

    fn source_type(&self, parameter: &str) -> Option<&TypeRef> {
        type_named(&self.declared, parameter).or_else(|| type_named(&self.resolved, parameter))
    }

    fn judge_body(&self, mut inner: Context) -> (Vec<Verdict>, Option<Subject>) {
        let mut verdicts = self.body.judge(&mut inner);
        let yielded = self.result.as_ref().map(|result| {
            let judged = result.judge(&inner);
            verdicts.extend(judged.verdicts);
            judged.subject
        });
        (verdicts, yielded)
    }

    fn judge_value(&self, context: &Context, target: Option<Target>) -> Judged {
        let expected = self
            .expected
            .as_ref()
            .or(target.map(|target| target.type_ref));
        let functional = expected.and_then(|expected| context.types.functional_method(expected));
        let declared = functional
            .as_ref()
            .map(|functional| functional.parameters.as_slice())
            .unwrap_or_default();
        let mut inner = context.clone();
        for (index, parameter) in self.parameters.iter().enumerate() {
            let type_ref = self
                .source_type(parameter)
                .or(declared.get(index))
                .cloned()
                .or_else(|| context.types.root_supertype());
            inner.bind(
                parameter,
                Subject {
                    standing: Standing::Friend(Reason::parameter(parameter)),
                    type_ref,
                },
            );
        }
        let (verdicts, _) = self.judge_body(inner);
        let subject = match expected {
            Some(expected) => Subject::friend(Reason::created(expected.clone()), expected.clone()),
            None => Subject {
                standing: Standing::Friend(Reason::Literal),
                type_ref: None,
            },
        };
        Judged { verdicts, subject }
    }
}

fn type_named<'a>(types: &'a [(String, TypeRef)], parameter: &str) -> Option<&'a TypeRef> {
    types
        .iter()
        .find(|(name, _)| name == parameter)
        .map(|(_, type_ref)| type_ref)
}
