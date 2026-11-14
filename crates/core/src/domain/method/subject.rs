use std::collections::HashMap;

use super::reason::{Reason, Verdict};
use crate::domain::classification::Types;
use crate::domain::type_shape::TypeRef;

#[derive(Debug, Clone)]
pub(super) enum Standing {
    Friend(Reason),
    Stranger(Reason),
}

impl Standing {
    pub(super) fn is_held(&self) -> bool {
        matches!(self, Self::Friend(reason) if reason.is_held())
    }

    pub(super) fn through(self, wrap: impl FnOnce(Reason) -> Reason) -> Self {
        match self {
            Self::Friend(reason) => Self::Friend(wrap(reason)),
            Self::Stranger(reason) => Self::Stranger(wrap(reason)),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct Subject {
    pub(super) standing: Standing,
    pub(super) type_ref: Option<TypeRef>,
}

impl Subject {
    pub(super) fn friend(reason: Reason, type_ref: TypeRef) -> Self {
        Self {
            standing: Standing::Friend(reason),
            type_ref: Some(type_ref),
        }
    }

    pub(super) fn through(self, wrap: impl FnOnce(Reason) -> Reason) -> Self {
        Self {
            standing: self.standing.through(wrap),
            type_ref: self.type_ref,
        }
    }

    pub(super) fn declared_as(self, declared: Option<&TypeRef>) -> Self {
        Self {
            type_ref: declared.cloned().or(self.type_ref),
            ..self
        }
    }

    pub(super) fn element(&self, types: &Types, wrap: impl FnOnce(Reason) -> Reason) -> Self {
        Self {
            standing: self.standing.clone().through(wrap),
            type_ref: self
                .type_ref
                .as_ref()
                .and_then(|type_ref| types.element_type_of(type_ref)),
        }
    }
}

pub(super) struct Judged {
    pub(super) verdicts: Vec<Verdict>,
    pub(super) subject: Subject,
}

impl Judged {
    pub(super) fn quiet(subject: Subject) -> Self {
        Self {
            verdicts: Vec::new(),
            subject,
        }
    }
}

#[derive(Clone)]
pub(super) struct Context<'a> {
    pub(super) types: &'a Types,
    pub(super) this_type: TypeRef,
    locals: HashMap<String, Subject>,
}

impl<'a> Context<'a> {
    pub(super) fn new(types: &'a Types, class_name: &str) -> Self {
        Self {
            types,
            this_type: TypeRef::class(class_name),
            locals: HashMap::new(),
        }
    }

    pub(super) fn bind(&mut self, name: &str, subject: Subject) {
        self.locals.insert(name.to_owned(), subject);
    }

    pub(super) fn local(&self, name: &str) -> Subject {
        self.locals.get(name).cloned().unwrap_or_else(|| Subject {
            standing: Standing::Stranger(Reason::unassigned_local(name)),
            type_ref: None,
        })
    }
}
