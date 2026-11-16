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

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Strict,
    Loose,
}

pub(super) fn select(
    types: &Types,
    candidates: &[Vec<TypeRef>],
    arguments: &[Option<ArgumentType>],
) -> Selection {
    if candidates.len() == 1 {
        return Selection::Chosen(0);
    }
    let Some(arguments) = arguments
        .iter()
        .map(Option::as_ref)
        .collect::<Option<Vec<_>>>()
    else {
        return Selection::Undecided;
    };
    if candidates.iter().flatten().any(has_variable) {
        return Selection::Undecided;
    }
    [Phase::Strict, Phase::Loose]
        .into_iter()
        .map(|phase| applicable(types, candidates, &arguments, phase))
        .find(|applicable| !applicable.is_empty())
        .map_or(Selection::Unresolvable, |applicable| {
            most_specific(types, candidates, &applicable)
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
    candidates: &[Vec<TypeRef>],
    arguments: &[&ArgumentType],
    phase: Phase,
) -> Vec<usize> {
    (0..candidates.len())
        .filter(|&index| {
            candidates[index]
                .iter()
                .zip(arguments)
                .all(|(parameter, argument)| accepts(types, parameter, argument, phase))
        })
        .collect()
}

fn accepts(types: &Types, parameter: &TypeRef, argument: &ArgumentType, phase: Phase) -> bool {
    match argument {
        ArgumentType::Null => !matches!(parameter, TypeRef::Primitive(_)),
        ArgumentType::Typed(argument) => {
            substitutable(types, argument, parameter)
                || (phase == Phase::Loose && converts_by_boxing(types, argument, parameter))
        }
    }
}

fn most_specific(types: &Types, candidates: &[Vec<TypeRef>], applicable: &[usize]) -> Selection {
    let specific: Vec<_> = applicable
        .iter()
        .copied()
        .filter(|&index| {
            applicable.iter().all(|&other| {
                index == other || more_specific(types, &candidates[index], &candidates[other])
            })
        })
        .collect();
    match specific[..] {
        [only] => Selection::Chosen(only),
        _ => Selection::Unresolvable,
    }
}

fn more_specific(types: &Types, parameters: &[TypeRef], others: &[TypeRef]) -> bool {
    parameters
        .iter()
        .zip(others)
        .all(|(parameter, other)| substitutable(types, parameter, other))
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
