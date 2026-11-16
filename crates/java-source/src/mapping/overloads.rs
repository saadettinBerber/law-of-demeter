use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::type_shape::TypeRef;

use crate::LiteralType;

use super::conversions::{boxed, unboxed, widens};
use super::lineage::{OBJECT, class_name, closure, has_variable};

const ARRAY_SUPERTYPES: [&str; 3] = [OBJECT, "java.lang.Cloneable", "java.io.Serializable"];

pub(super) enum ArgumentType {
    Null,
    Typed(TypeRef),
}

impl ArgumentType {
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
    Chosen(usize),
    Unresolvable,
    Undecided,
}

pub(super) struct Signature {
    pub(super) parameters: Vec<TypeRef>,
    pub(super) variable_arity: bool,
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
    let Some(arguments) = arguments
        .iter()
        .map(Option::as_ref)
        .collect::<Option<Vec<_>>>()
    else {
        return Selection::Undecided;
    };
    if signatures
        .iter()
        .flat_map(|signature| &signature.parameters)
        .any(has_variable)
    {
        return Selection::Undecided;
    }
    [Phase::Strict, Phase::Loose, Phase::VariableArity]
        .into_iter()
        .map(|phase| (phase, applicable(types, signatures, &arguments, phase)))
        .find(|(_, applicable)| !applicable.is_empty())
        .map_or(Selection::Unresolvable, |(phase, applicable)| {
            most_specific(types, signatures, &applicable, phase, arguments.len())
        })
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
) -> Vec<usize> {
    (0..signatures.len())
        .filter(|&index| {
            signatures[index]
                .parameters_for(phase, arguments.len())
                .is_some_and(|parameters| {
                    parameters
                        .iter()
                        .zip(arguments)
                        .all(|(parameter, argument)| accepts(types, parameter, argument, phase))
                })
        })
        .collect()
}

fn accepts(types: &Types, parameter: &TypeRef, argument: &ArgumentType, phase: Phase) -> bool {
    match argument {
        ArgumentType::Null => !matches!(parameter, TypeRef::Primitive(_)),
        ArgumentType::Typed(argument) => {
            substitutable(types, argument, parameter)
                || (phase != Phase::Strict && converts_by_boxing(types, argument, parameter))
        }
    }
}

fn most_specific(
    types: &Types,
    signatures: &[Signature],
    applicable: &[usize],
    phase: Phase,
    count: usize,
) -> Selection {
    let specific: Vec<_> = applicable
        .iter()
        .copied()
        .filter(|&index| {
            applicable.iter().all(|&other| {
                index == other
                    || more_specific(types, phase, count, &signatures[index], &signatures[other])
            })
        })
        .collect();
    match specific[..] {
        [only] => Selection::Chosen(only),
        _ => Selection::Unresolvable,
    }
}

fn more_specific(
    types: &Types,
    phase: Phase,
    count: usize,
    signature: &Signature,
    other: &Signature,
) -> bool {
    let length = count
        .max(signature.parameters.len())
        .max(other.parameters.len());
    match (
        signature.parameters_for(phase, length),
        other.parameters_for(phase, length),
    ) {
        (Some(parameters), Some(others)) => parameters
            .iter()
            .zip(&others)
            .all(|(parameter, other)| substitutable(types, parameter, other)),
        _ => false,
    }
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
