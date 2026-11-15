use super::expression::{Argument, Expression};
use super::reason::{Reason, Verdict};
use super::subject::{Context, Judged, Standing, Subject};
use crate::domain::classification::{
    Classification, Invocation, Types, is_factory_method, is_repository_query,
};
use crate::domain::type_shape::TypeRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    receiver: Expression,
    name: String,
    line: usize,
    arguments: Vec<Argument>,
    expected: Option<TypeRef>,
}

impl Call {
    pub fn new(receiver: Expression, name: &str, line: usize) -> Self {
        Self {
            receiver,
            name: name.to_owned(),
            line,
            arguments: Vec::new(),
            expected: None,
        }
    }

    pub fn expecting(mut self, type_ref: TypeRef) -> Self {
        self.expected = Some(type_ref);
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

    pub(super) fn judge_expecting(&self, context: &Context, expected: Option<&TypeRef>) -> Judged {
        let receiver = self.receiver.judge(context);
        let mut verdicts = receiver.verdicts;
        let values: Vec<_> = self
            .arguments
            .iter()
            .map(|argument| argument.judge_value(context))
            .collect();
        let expected = self.expected.as_ref().or(expected);
        let invocation = self.invocation(context.types, &receiver.subject, &values, expected);
        for (index, (argument, value)) in self.arguments.iter().zip(values).enumerate() {
            match value {
                Some(judged) => verdicts.extend(judged.verdicts),
                None => {
                    let functional = invocation.functional_method(context.types, index);
                    verdicts.extend(argument.judge(
                        context,
                        &receiver.subject,
                        functional.as_ref(),
                    ));
                }
            }
        }
        verdicts.push(self.verdict(context, &receiver.subject));
        let subject = Subject {
            standing: self.result_standing(context.types, &receiver.subject),
            type_ref: invocation.returns(),
        };
        Judged { verdicts, subject }
    }

    fn invocation(
        &self,
        types: &Types,
        receiver: &Subject,
        values: &[Option<Judged>],
        expected: Option<&TypeRef>,
    ) -> Invocation {
        let value_types: Vec<_> = values
            .iter()
            .map(|value| {
                value
                    .as_ref()
                    .and_then(|judged| judged.subject.type_ref.as_ref())
            })
            .collect();
        receiver
            .type_ref
            .as_ref()
            .map_or_else(Invocation::unknown, |receiver_type| {
                types.invocation(receiver_type, &self.name, &value_types)
            })
            .expecting(types, expected)
    }

    fn verdict(&self, context: &Context, receiver: &Subject) -> Verdict {
        match &receiver.standing {
            Standing::Friend(reason) => Verdict::allowed(&self.name, self.line, reason.clone()),
            Standing::Stranger(source) => {
                self.stranger_verdict(context, receiver.type_ref.as_ref(), source)
            }
        }
    }

    fn stranger_verdict(
        &self,
        context: &Context,
        type_ref: Option<&TypeRef>,
        source: &Reason,
    ) -> Verdict {
        let types = context.types;
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
                classification @ (Classification::Value | Classification::Container),
                type_ref,
            )) => Verdict::allowed(
                &self.name,
                self.line,
                Reason::classified(classification, type_ref.clone()),
            ),
            Some((_, type_ref)) if is_library(context, type_ref) => {
                Verdict::allowed(&self.name, self.line, Reason::library(type_ref.clone()))
            }
            Some((Classification::DataStructure, type_ref)) => Verdict::allowed(
                &self.name,
                self.line,
                Reason::classified(Classification::DataStructure, type_ref.clone()),
            ),
            Some((_, type_ref)) if self.is_fluent_on(types, type_ref) => {
                Verdict::allowed(&self.name, self.line, Reason::fluent(type_ref.clone()))
            }
            Some((Classification::Unknown, _)) | None => Verdict::violation(
                &self.name,
                self.line,
                Reason::unresolved_type(type_ref.cloned(), source.clone()),
            ),
            Some(_) => Verdict::violation(&self.name, self.line, source.clone()),
        }
    }

    fn is_fluent_on(&self, types: &Types, type_ref: &TypeRef) -> bool {
        class_name(type_ref)
            .is_some_and(|name| types.is_fluent(name, &self.name, self.arguments.len()))
    }

    fn result_standing(&self, types: &Types, receiver: &Subject) -> Standing {
        let standing = &receiver.standing;
        let keeps_standing = receiver.type_ref.as_ref().is_some_and(|type_ref| {
            self.is_fluent_on(types, type_ref)
                || types.classify(type_ref) == Classification::Container
        });
        if matches!(self.receiver, Expression::StaticType(_)) {
            Standing::Friend(Reason::static_result(&self.name))
        } else if keeps_standing {
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

fn is_library(context: &Context, type_ref: &TypeRef) -> bool {
    let types = context.types;
    let project_context =
        class_name(&context.this_type).is_some_and(|name| types.is_project_type(name));
    project_context
        && class_name(type_ref)
            .is_some_and(|name| types.shape(name).is_some() && !types.is_project_type(name))
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
