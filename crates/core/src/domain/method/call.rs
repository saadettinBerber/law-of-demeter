use super::arguments::{Links, judge_arguments, judge_values, value_types};
use super::expression::{Argument, Expression};
use super::reason::{Reason, Verdict};
use super::subject::{Context, Judged, Standing, Subject, Target};
use crate::domain::classification::{
    Classification, Invocation, Types, is_factory_method, is_query_on_repository,
    is_repository_query,
};
use crate::domain::type_shape::TypeRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    receiver: Expression,
    name: String,
    line: usize,
    arguments: Vec<Argument>,
    expected: Option<TypeRef>,
    resolved: Option<TypeRef>,
    flows: Vec<usize>,
    keyed_by: Option<usize>,
    reads_keys: bool,
}

impl Call {
    pub fn new(receiver: Expression, name: &str, line: usize) -> Self {
        Self {
            receiver,
            name: name.to_owned(),
            line,
            arguments: Vec::new(),
            expected: None,
            resolved: None,
            flows: Vec::new(),
            keyed_by: None,
            reads_keys: false,
        }
    }

    pub fn expecting(mut self, type_ref: TypeRef) -> Self {
        self.expected = Some(type_ref);
        self
    }

    pub fn returning(mut self, type_ref: TypeRef) -> Self {
        self.resolved = Some(type_ref);
        self
    }

    pub fn returning_from(mut self, arguments: &[usize]) -> Self {
        self.flows = arguments.to_vec();
        self
    }

    pub fn keyed_by(mut self, argument: usize) -> Self {
        self.keyed_by = Some(argument);
        self
    }

    pub fn yielding_keys(mut self) -> Self {
        self.reads_keys = true;
        self
    }

    pub fn with(mut self, argument: impl Into<Argument>) -> Self {
        self.arguments.push(argument.into());
        self
    }

    pub fn then(self, name: &str, line: usize) -> Self {
        Self::new(Expression::return_of(self), name, line)
    }

    pub(super) fn judge(&self, context: &Context) -> Judged {
        self.judge_expecting(context, None)
    }

    pub(super) fn judge_expecting(&self, context: &Context, target: Option<Target>) -> Judged {
        let receiver = self.receiver.judge(context);
        let mut verdicts = receiver.verdicts;
        let outer = target
            .and_then(|target| target.elements)
            .unwrap_or(&receiver.subject);
        let values = judge_values(context, &self.arguments, outer);
        let expected = self
            .expected
            .as_ref()
            .or(target.and_then(|target| target.type_ref));
        let invocation = self.invocation(
            context.types,
            &receiver.subject,
            &value_types(&values),
            expected,
        );
        let judged = judge_arguments(
            context,
            &self.arguments,
            values,
            &Links {
                flows: &self.flows,
                keyed_by: self.keyed_by,
            },
            (&receiver.subject, outer),
        );
        verdicts.extend(judged.verdicts);
        verdicts.push(verdict_on(
            context,
            &receiver.subject,
            &Called {
                name: &self.name,
                line: self.line,
                argument_count: self.arguments.len(),
            },
        ));
        let type_ref = self.resolved.clone().or_else(|| invocation.returns());
        let mapped = judged.mapped.filter(|_| {
            Self::is_container(context.types, receiver.subject.type_ref.as_ref())
                && Self::is_container(context.types, type_ref.as_ref())
        });
        let read_keys = receiver.subject.keys.clone().filter(|_| self.reads_keys);
        let standing = read_keys.or(mapped).unwrap_or_else(|| {
            standing_after_call(
                context.types,
                &receiver.subject,
                &self.name,
                self.arguments.len(),
                matches!(self.receiver, Expression::StaticType(_)),
            )
        });
        let subject = Subject {
            keys: judged.keys,
            ..Subject::new(standing, type_ref)
        };
        Judged { verdicts, subject }
    }

    fn is_container(types: &Types, type_ref: Option<&TypeRef>) -> bool {
        type_ref.is_some_and(|type_ref| types.classify(type_ref) == Classification::Container)
    }

    fn invocation(
        &self,
        types: &Types,
        receiver: &Subject,
        value_types: &[Option<&TypeRef>],
        expected: Option<&TypeRef>,
    ) -> Invocation {
        receiver
            .type_ref
            .as_ref()
            .map_or_else(Invocation::unknown, |receiver_type| {
                types.invocation(receiver_type, &self.name, value_types)
            })
            .expecting(types, expected)
    }
}

pub(super) struct Called<'a> {
    pub(super) name: &'a str,
    pub(super) line: usize,
    pub(super) argument_count: usize,
}

pub(super) fn verdict_on(context: &Context, receiver: &Subject, called: &Called) -> Verdict {
    match &receiver.standing {
        Standing::Friend(reason) => Verdict::allowed(called.name, called.line, reason.clone()),
        Standing::Stranger(source) => {
            stranger_verdict(context, called, receiver.type_ref.as_ref(), source)
        }
    }
}

fn stranger_verdict(
    context: &Context,
    called: &Called,
    type_ref: Option<&TypeRef>,
    source: &Reason,
) -> Verdict {
    let types = context.types;
    let (name, line) = (called.name, called.line);
    let classified = type_ref.map(|type_ref| (types.classify(type_ref), type_ref));
    match classified {
        Some((Classification::Container, type_ref)) if types.is_mutator(name) => {
            Verdict::violation(
                name,
                line,
                Reason::container_mutation(type_ref.clone(), source.clone()),
            )
        }
        Some((classification @ (Classification::Value | Classification::Container), type_ref)) => {
            Verdict::allowed(
                name,
                line,
                Reason::classified(classification, type_ref.clone()),
            )
        }
        Some((_, type_ref)) if is_library(context, type_ref) => {
            Verdict::allowed(name, line, Reason::library(type_ref.clone()))
        }
        Some((Classification::DataStructure, type_ref)) => Verdict::allowed(
            name,
            line,
            Reason::classified(Classification::DataStructure, type_ref.clone()),
        ),
        Some((_, type_ref)) if is_fluent(types, type_ref, name, called.argument_count) => {
            Verdict::allowed(name, line, Reason::fluent(type_ref.clone()))
        }
        Some((Classification::Unknown, _)) | None => Verdict::violation(
            name,
            line,
            Reason::unresolved_type(type_ref.cloned(), source.clone()),
        ),
        Some(_) => Verdict::violation(name, line, source.clone()),
    }
}

pub(super) fn standing_after_call(
    types: &Types,
    receiver: &Subject,
    name: &str,
    argument_count: usize,
    on_type: bool,
) -> Standing {
    let standing = &receiver.standing;
    let keeps_standing = receiver.type_ref.as_ref().is_some_and(|type_ref| {
        is_fluent(types, type_ref, name, argument_count)
            || types.classify(type_ref) == Classification::Container
    });
    if on_type {
        Standing::Friend(Reason::static_result(name))
    } else if keeps_standing {
        standing.clone()
    } else if is_factory_method(name) && matches!(standing, Standing::Friend(_)) {
        Standing::Friend(Reason::factory_result(name))
    } else if is_query(receiver, name) {
        Standing::Friend(Reason::query_result(name))
    } else {
        Standing::Stranger(Reason::return_of(name))
    }
}

fn is_query(receiver: &Subject, name: &str) -> bool {
    let on_repository = receiver
        .type_ref
        .as_ref()
        .and_then(class_name)
        .is_some_and(|owner| is_query_on_repository(owner, name));
    receiver.standing.is_held() && (is_repository_query(name) || on_repository)
}

fn is_fluent(types: &Types, type_ref: &TypeRef, name: &str, argument_count: usize) -> bool {
    class_name(type_ref).is_some_and(|owner| types.is_fluent(owner, name, argument_count))
}

fn is_library(context: &Context, type_ref: &TypeRef) -> bool {
    let types = context.types;
    let project_context =
        class_name(&context.this_type).is_some_and(|name| types.is_project_type(name));
    project_context
        && class_name(type_ref)
            .is_some_and(|name| types.shape(name).is_some() && !types.is_project_type(name))
}

fn class_name(type_ref: &TypeRef) -> Option<&str> {
    match type_ref {
        TypeRef::Class { name, .. } => Some(name),
        TypeRef::Primitive(_) | TypeRef::Array(_) | TypeRef::Variable(_) => None,
    }
}
