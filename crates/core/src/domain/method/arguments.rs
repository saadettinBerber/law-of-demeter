use super::expression::Argument;
use super::reason::Verdict;
use super::subject::{Context, Judged, Standing, Subject, Target};
use crate::domain::classification::{Invocation, has_variable};
use crate::domain::type_shape::TypeRef;

#[derive(Default)]
pub(super) struct Links<'a> {
    pub(super) flows: Option<&'a [usize]>,
    pub(super) keyed_by: Option<usize>,
}

pub(super) struct Judgement {
    pub(super) verdicts: Vec<Verdict>,
    pub(super) mapped: Option<Standing>,
    pub(super) keys: Option<Standing>,
}

pub(super) fn judge_values(
    context: &Context,
    arguments: &[Argument],
    outer: &Subject,
) -> Vec<Option<Judged>> {
    arguments
        .iter()
        .map(|argument| argument.judge_value_expecting(context, Some(Target::elements_of(outer))))
        .collect()
}

pub(super) fn value_types(values: &[Option<Judged>]) -> Vec<Option<&TypeRef>> {
    values
        .iter()
        .map(|value| {
            value
                .as_ref()
                .and_then(|judged| judged.subject.type_ref.as_ref())
                .filter(|type_ref| !has_variable(type_ref))
        })
        .collect()
}

pub(super) fn judge_arguments(
    context: &Context,
    arguments: &[Argument],
    values: Vec<Option<Judged>>,
    invocation: &mut Invocation,
    links: &Links,
    (feed, outer): (&Subject, &Subject),
) -> Judgement {
    let mut verdicts = Vec::new();
    let mut mapped = None;
    let mut keys = None;
    for (index, (argument, value)) in arguments.iter().zip(values).enumerate() {
        match value {
            Some(judged) => {
                let judged = settled(context, argument, index, judged, invocation, outer);
                verdicts.extend(judged.verdicts);
                if links.keyed_by == Some(index) {
                    keys = keys.or(judged.subject.keys);
                }
            }
            None => {
                let functional = invocation.functional_method(context.types, index);
                let produced = argument.judge(context, feed, functional.as_ref(), outer);
                verdicts.extend(produced.verdicts);
                invocation.yielded(context.types, index, produced.result.as_ref());
                let feeds_return = match links.flows {
                    Some(flows) => flows.contains(&index),
                    None => invocation.feeds_return(context.types, index),
                };
                if links.keyed_by == Some(index) {
                    keys = keys.or(produced.standing.clone());
                }
                if feeds_return {
                    mapped = mapped.or(produced.standing);
                }
            }
        }
    }
    Judgement {
        verdicts,
        mapped,
        keys,
    }
}

fn settled(
    context: &Context,
    argument: &Argument,
    index: usize,
    judged: Judged,
    invocation: &mut Invocation,
    outer: &Subject,
) -> Judged {
    if !judged.subject.type_ref.as_ref().is_some_and(has_variable) {
        return judged;
    }
    let retried = invocation.parameter_type(index).and_then(|expected| {
        argument.judge_value_expecting(context, Some(Target::among(&expected, Some(outer))))
    });
    let judged = retried.unwrap_or(judged);
    invocation.argument(context.types, index, judged.subject.type_ref.as_ref());
    judged
}
