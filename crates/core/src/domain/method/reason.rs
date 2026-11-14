use crate::domain::classification::Classification;
use crate::domain::type_shape::TypeRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    This,
    Literal,
    Field(String),
    FieldOf(String),
    Parameter(String),
    ReturnOf(String),
    UnassignedLocal(String),
    UnresolvedName(String),
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
    UnresolvedType {
        type_ref: Option<TypeRef>,
        source: Box<Reason>,
    },
}

impl Reason {
    pub(super) fn is_held(&self) -> bool {
        match self {
            Self::Field(_) | Self::Parameter(_) => true,
            Self::Local { origin, .. } => origin.is_held(),
            _ => false,
        }
    }

    pub fn field(name: &str) -> Self {
        Self::Field(name.to_owned())
    }

    pub fn field_of(name: &str) -> Self {
        Self::FieldOf(name.to_owned())
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

    pub fn unresolved_type(type_ref: Option<TypeRef>, source: Reason) -> Self {
        Self::UnresolvedType {
            type_ref,
            source: Box::new(source),
        }
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

    pub fn unresolved_name(name: &str) -> Self {
        Self::UnresolvedName(name.to_owned())
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
