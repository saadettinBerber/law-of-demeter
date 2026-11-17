use std::collections::HashMap;

use super::Method;
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
    pub(super) keys: Option<Standing>,
    pub(super) values: Option<Standing>,
}

impl Subject {
    pub(super) fn new(standing: Standing, type_ref: Option<TypeRef>) -> Self {
        Self {
            standing,
            type_ref,
            keys: None,
            values: None,
        }
    }

    pub(super) fn friend(reason: Reason, type_ref: TypeRef) -> Self {
        Self::new(Standing::Friend(reason), Some(type_ref))
    }

    pub(super) fn through(self, wrap: impl FnOnce(Reason) -> Reason) -> Self {
        Self {
            standing: self.standing.through(wrap),
            ..self
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
            keys: self.keys.clone(),
            values: self.values.clone(),
            ..Self::new(
                self.standing.clone().through(wrap),
                self.type_ref
                    .as_ref()
                    .and_then(|type_ref| types.element_type_of(type_ref)),
            )
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Target<'a> {
    pub(super) type_ref: Option<&'a TypeRef>,
    pub(super) elements: Option<&'a Subject>,
}

impl<'a> Target<'a> {
    pub(super) fn of(type_ref: &'a TypeRef) -> Self {
        Self {
            type_ref: Some(type_ref),
            elements: None,
        }
    }

    pub(super) fn elements_of(elements: &'a Subject) -> Self {
        Self {
            type_ref: None,
            elements: Some(elements),
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
    methods: &'a [Method],
    running: Vec<usize>,
    given_back: Vec<Standing>,
}

impl<'a> Context<'a> {
    pub(super) fn new(types: &'a Types, class_name: &str, methods: &'a [Method]) -> Self {
        Self {
            types,
            this_type: TypeRef::class(class_name),
            locals: HashMap::new(),
            methods,
            running: Vec::new(),
            given_back: Vec::new(),
        }
    }

    pub(super) fn give_back(&mut self, standing: Standing) {
        self.given_back.push(standing);
    }

    pub(super) fn given_back(self) -> Vec<Standing> {
        self.given_back
    }

    pub(super) fn helper_result(&self, name: &str, line: usize) -> Option<Standing> {
        if self.running.contains(&line) {
            return None;
        }
        let helper = self.methods.iter().find(|method| {
            method.name() == name
                && method.line() == line
                && TypeRef::class(method.class_name()) == self.this_type
        })?;
        let inner = Self {
            locals: HashMap::new(),
            running: [self.running.as_slice(), &[line]].concat(),
            given_back: Vec::new(),
            ..self.clone()
        };
        helper.handed_back(inner)
    }

    pub(super) fn bind(&mut self, name: &str, subject: Subject) {
        self.locals.insert(name.to_owned(), subject);
    }

    pub(super) fn local(&self, name: &str) -> Subject {
        self.locals.get(name).cloned().unwrap_or_else(|| {
            Subject::new(Standing::Stranger(Reason::unassigned_local(name)), None)
        })
    }
}
