use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::type_shape::TypeRef;

use crate::LiteralType;

use super::conversions::{boxed, is_primitive, unboxed, widens};
use super::inference::{Bindings, apply_bindings, instantiate};
use super::lambdas::functional_method;
use super::lineage::{ARRAY_SUPERTYPES, OBJECT, class_name, closure};

pub(super) enum ArgumentType {
    Null,
    Typed(TypeRef),
    Foreign(TypeRef),
    Lambda(usize),
    Reference,
    Open(TypeRef),
    Unknown,
}

impl ArgumentType {
    pub(super) fn of_type(types: &Types, type_ref: TypeRef) -> Self {
        if is_known(types, &type_ref) {
            Self::Typed(type_ref)
        } else {
            Self::Foreign(type_ref)
        }
    }

    fn is_unsure(&self) -> bool {
        matches!(self, Self::Foreign(_) | Self::Unknown)
    }

    pub(super) fn of_literal(literal: LiteralType) -> Self {
        match literal {
            LiteralType::Null => Self::Null,
            LiteralType::String => Self::Typed(TypeRef::class("java.lang.String")),
            LiteralType::Int => Self::Typed(TypeRef::primitive("int")),
            LiteralType::Long => Self::Typed(TypeRef::primitive("long")),
            LiteralType::Float => Self::Typed(TypeRef::primitive("float")),
            LiteralType::Double => Self::Typed(TypeRef::primitive("double")),
            LiteralType::Boolean => Self::Typed(TypeRef::primitive("boolean")),
            LiteralType::Char => Self::Typed(TypeRef::primitive("char")),
        }
    }
}

pub(super) enum Selection {
    Chosen {
        index: usize,
        bindings: Bindings,
        parameters: Vec<TypeRef>,
    },
    Ambiguous(Vec<usize>),
}

pub(super) struct Signature {
    pub(super) parameters: Vec<TypeRef>,
    pub(super) variable_arity: bool,
    pub(super) free: Vec<String>,
}

struct Applicant {
    index: usize,
    bindings: Bindings,
    parameters: Vec<TypeRef>,
}

impl Signature {
    pub(super) fn takes(&self, count: usize) -> bool {
        self.parameters.len() == count
            || (self.variable_arity && self.parameters.len() <= count + 1)
    }

    fn parameters_for(&self, phase: Phase, length: usize) -> Option<Vec<TypeRef>> {
        match phase {
            Phase::VariableArity => self.expanded(length),
            Phase::Strict | Phase::Loose => {
                (self.parameters.len() == length).then(|| self.parameters.clone())
            }
        }
    }

    fn expanded(&self, length: usize) -> Option<Vec<TypeRef>> {
        let (TypeRef::Array(element), fixed) = self.parameters.split_last()? else {
            return None;
        };
        if !self.variable_arity || fixed.len() > length {
            return None;
        }
        let repeated = std::iter::repeat_n(element.as_ref().clone(), length - fixed.len());
        Some(fixed.iter().cloned().chain(repeated).collect())
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Strict,
    Loose,
    VariableArity,
}

pub(super) fn select(
    types: &Types,
    signatures: &[Signature],
    arguments: &[Option<ArgumentType>],
) -> Selection {
    let arguments: Vec<_> = arguments
        .iter()
        .map(|argument| argument.as_ref().unwrap_or(&ArgumentType::Unknown))
        .collect();
    for phase in [Phase::Strict, Phase::Loose, Phase::VariableArity] {
        let applicants = applicable(types, signatures, &arguments, phase);
        if applicants.len() > 1 && arguments.iter().any(|argument| argument.is_unsure()) {
            return Selection::Ambiguous(indices_of(&applicants));
        }
        if !applicants.is_empty() {
            return most_specific(types, signatures, &applicants, phase, arguments.len());
        }
    }
    Selection::Ambiguous((0..signatures.len()).collect())
}

pub(super) fn is_known(types: &Types, type_ref: &TypeRef) -> bool {
    match type_ref {
        TypeRef::Primitive(_) => true,
        TypeRef::Array(element) => is_known(types, element),
        TypeRef::Variable(_) => false,
        TypeRef::Class { .. } => closure(types, type_ref)
            .iter()
            .all(|supertype| class_name(supertype).is_some_and(|name| types.shape(name).is_some())),
    }
}

fn applicable(
    types: &Types,
    signatures: &[Signature],
    arguments: &[&ArgumentType],
    phase: Phase,
) -> Vec<Applicant> {
    let mut applicants = Vec::new();
    for (index, signature) in signatures.iter().enumerate() {
        let Some(parameters) = signature.parameters_for(phase, arguments.len()) else {
            continue;
        };
        let (parameters, bindings) = instantiate(types, &signature.free, &parameters, arguments);
        let fits = parameters
            .iter()
            .zip(arguments)
            .all(|(parameter, argument)| accepts(types, parameter, argument, phase));
        if fits {
            applicants.push(Applicant {
                index,
                bindings,
                parameters,
            });
        }
    }
    applicants
}

fn accepts(types: &Types, parameter: &TypeRef, argument: &ArgumentType, phase: Phase) -> bool {
    if matches!(parameter, TypeRef::Variable(_)) {
        return true;
    }
    match argument {
        ArgumentType::Unknown => true,
        ArgumentType::Foreign(_) => {
            matches!(parameter, TypeRef::Class { .. } | TypeRef::Variable(_))
        }
        ArgumentType::Null => !matches!(parameter, TypeRef::Primitive(_)),
        ArgumentType::Lambda(arity) => {
            matches!(parameter, TypeRef::Variable(_))
                || functional_method(types, parameter)
                    .is_some_and(|functional| functional.parameters.len() == *arity)
        }
        ArgumentType::Reference => {
            matches!(parameter, TypeRef::Variable(_))
                || functional_method(types, parameter).is_some()
        }
        ArgumentType::Open(returned) => match returned {
            TypeRef::Variable(_) => phase != Phase::Strict || !is_primitive(parameter),
            _ => substitutable(types, returned, parameter),
        },
        ArgumentType::Typed(argument) => {
            substitutable(types, argument, parameter)
                || (phase != Phase::Strict && converts_by_boxing(types, argument, parameter))
        }
    }
}

fn most_specific(
    types: &Types,
    signatures: &[Signature],
    applicants: &[Applicant],
    phase: Phase,
    count: usize,
) -> Selection {
    let specific: Vec<_> = applicants
        .iter()
        .filter(|applicant| {
            applicants.iter().all(|other| {
                applicant.index == other.index
                    || more_specific(types, phase, count, signatures, (applicant, other))
            })
        })
        .collect();
    match specific[..] {
        [only] => Selection::Chosen {
            index: only.index,
            bindings: only.bindings.clone(),
            parameters: only.parameters.clone(),
        },
        _ => Selection::Ambiguous(indices_of(applicants)),
    }
}

fn indices_of(applicants: &[Applicant]) -> Vec<usize> {
    applicants.iter().map(|applicant| applicant.index).collect()
}

fn more_specific(
    types: &Types,
    phase: Phase,
    count: usize,
    signatures: &[Signature],
    (applicant, other): (&Applicant, &Applicant),
) -> bool {
    let signature = &signatures[applicant.index];
    let other_signature = &signatures[other.index];
    let length = count
        .max(signature.parameters.len())
        .max(other_signature.parameters.len());
    match (
        instance(signature, phase, length, &applicant.bindings),
        instance(other_signature, phase, length, &other.bindings),
    ) {
        (Some(parameters), Some(others)) => parameters
            .iter()
            .zip(&others)
            .all(|(parameter, other)| substitutable(types, parameter, other)),
        _ => false,
    }
}

fn instance(
    signature: &Signature,
    phase: Phase,
    length: usize,
    bindings: &Bindings,
) -> Option<Vec<TypeRef>> {
    let parameters = signature.parameters_for(phase, length)?;
    Some(
        parameters
            .iter()
            .map(|parameter| apply_bindings(parameter, bindings))
            .collect(),
    )
}

fn substitutable(types: &Types, from: &TypeRef, to: &TypeRef) -> bool {
    match (from, to) {
        (TypeRef::Primitive(from), TypeRef::Primitive(to)) => from == to || widens(from, to),
        (TypeRef::Primitive(_), _) | (_, TypeRef::Primitive(_)) => false,
        _ => is_subtype(types, from, to),
    }
}

fn is_subtype(types: &Types, from: &TypeRef, to: &TypeRef) -> bool {
    match (from, to) {
        (TypeRef::Array(from), TypeRef::Array(to)) => match (from.as_ref(), to.as_ref()) {
            (TypeRef::Primitive(from), TypeRef::Primitive(to)) => from == to,
            (TypeRef::Primitive(_), _) | (_, TypeRef::Primitive(_)) => false,
            (from, to) => is_subtype(types, from, to),
        },
        (TypeRef::Array(_), TypeRef::Class { name, .. }) => {
            ARRAY_SUPERTYPES.contains(&name.as_str())
        }
        (TypeRef::Class { .. }, TypeRef::Class { name, .. }) => {
            name == OBJECT
                || closure(types, from)
                    .iter()
                    .any(|supertype| class_name(supertype) == Some(name))
        }
        _ => false,
    }
}

fn converts_by_boxing(types: &Types, from: &TypeRef, to: &TypeRef) -> bool {
    match (from, to) {
        (TypeRef::Primitive(primitive), TypeRef::Class { .. }) => {
            boxed(primitive).is_some_and(|class| is_subtype(types, &TypeRef::class(class), to))
        }
        (TypeRef::Class { name, .. }, TypeRef::Primitive(target)) => {
            unboxed(name).is_some_and(|primitive| primitive == target || widens(primitive, target))
        }
        _ => false,
    }
}
