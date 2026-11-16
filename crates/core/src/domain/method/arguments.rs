use super::expression::Argument;
use super::reason::Verdict;
use super::subject::{Context, Judged, Standing, Subject, Target};

#[derive(Default)]
pub(super) struct Links<'a> {
    pub(super) flows: &'a [usize],
    pub(super) keyed_by: Option<usize>,
    pub(super) valued_by: Option<usize>,
}

pub(super) struct Judgement {
    pub(super) verdicts: Vec<Verdict>,
    pub(super) mapped: Option<Standing>,
    pub(super) keys: Option<Standing>,
    pub(super) values: Option<Standing>,
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

pub(super) fn judge_arguments(
    context: &Context,
    arguments: &[Argument],
    values: Vec<Option<Judged>>,
    links: &Links,
    (feed, outer): (&Subject, &Subject),
) -> Judgement {
    let siblings: Vec<Option<Subject>> = values
        .iter()
        .map(|value| value.as_ref().map(|judged| judged.subject.clone()))
        .collect();
    let mut verdicts = Vec::new();
    let mut mapped = None;
    let mut keys = None;
    let mut valued = None;
    for (index, (argument, value)) in arguments.iter().zip(values).enumerate() {
        match value {
            Some(judged) => {
                verdicts.extend(judged.verdicts);
                if links.keyed_by == Some(index) {
                    keys = keys.or(judged.subject.keys);
                }
                if links.valued_by == Some(index) {
                    valued = valued.or(judged.subject.values);
                }
            }
            None => {
                let produced = argument.judge(context, (feed, outer), &siblings);
                verdicts.extend(produced.verdicts);
                if links.keyed_by == Some(index) {
                    keys = keys.or(produced.standing.clone());
                }
                if links.valued_by == Some(index) {
                    valued = valued.or(produced.standing.clone());
                }
                if links.flows.contains(&index) {
                    mapped = mapped.or(produced.standing);
                }
            }
        }
    }
    Judgement {
        verdicts,
        mapped,
        keys,
        values: valued,
    }
}
