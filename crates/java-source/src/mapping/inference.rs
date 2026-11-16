use std::collections::HashMap;

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::type_shape::TypeRef;

use super::conversions::boxed;
use super::lineage::{
    ARRAY_SUPERTYPES, OBJECT, arguments_of, class_name, closure, has_variable, substitute,
};
use super::overloads::ArgumentType;

pub(super) type Bindings = HashMap<String, TypeRef>;

type Constraints = Vec<(String, TypeRef)>;

pub(super) fn instantiate(
    types: &Types,
    free: &[String],
    parameters: &[TypeRef],
    arguments: &[&ArgumentType],
) -> Option<(Vec<TypeRef>, Bindings)> {
    let mut constraints = Constraints::new();
    for (parameter, argument) in parameters.iter().zip(arguments) {
        if let ArgumentType::Typed(argument) = argument {
            from_argument(
                types,
                free,
                parameter,
                &reference(argument),
                &mut constraints,
            );
        }
    }
    let bindings = resolve(types, constraints);
    let instantiated: Vec<_> = parameters
        .iter()
        .map(|parameter| bound(parameter, &bindings))
        .collect();
    (!instantiated.iter().any(has_variable)).then_some((instantiated, bindings))
}

pub(super) fn bound(type_ref: &TypeRef, bindings: &Bindings) -> TypeRef {
    substitute(type_ref, &|name| bindings.get(name).cloned())
}

fn from_argument(
    types: &Types,
    free: &[String],
    parameter: &TypeRef,
    argument: &TypeRef,
    constraints: &mut Constraints,
) {
    match (parameter, argument) {
        (TypeRef::Variable(name), _) if free.contains(name) => {
            constraints.push((name.clone(), argument.clone()));
        }
        (TypeRef::Array(element), TypeRef::Array(actual)) if !is_primitive(actual) => {
            from_argument(types, free, element, actual, constraints);
        }
        (TypeRef::Class { name, arguments }, _) => {
            let actual = supertype_arguments(types, argument, name);
            for (declared, actual) in arguments.iter().zip(&actual) {
                from_equal(free, declared, actual, constraints);
            }
        }
        _ => {}
    }
}

fn from_equal(
    free: &[String],
    declared: &TypeRef,
    actual: &TypeRef,
    constraints: &mut Constraints,
) {
    match (declared, actual) {
        (TypeRef::Variable(name), _) if free.contains(name) => {
            constraints.push((name.clone(), actual.clone()));
        }
        (TypeRef::Array(declared), TypeRef::Array(actual)) => {
            from_equal(free, declared, actual, constraints);
        }
        (
            TypeRef::Class {
                name: declared_name,
                arguments: declared_arguments,
            },
            TypeRef::Class {
                name: actual_name,
                arguments: actual_arguments,
            },
        ) if declared_name == actual_name => {
            for (declared, actual) in declared_arguments.iter().zip(actual_arguments) {
                from_equal(free, declared, actual, constraints);
            }
        }
        _ => {}
    }
}

fn supertype_arguments(types: &Types, from: &TypeRef, name: &str) -> Vec<TypeRef> {
    closure(types, from)
        .iter()
        .find(|supertype| class_name(supertype) == Some(name))
        .map(|supertype| arguments_of(supertype).to_vec())
        .unwrap_or_default()
}

fn resolve(types: &Types, constraints: Constraints) -> Bindings {
    let mut grouped: HashMap<String, Vec<TypeRef>> = HashMap::new();
    for (name, type_ref) in constraints {
        grouped.entry(name).or_default().push(type_ref);
    }
    grouped
        .into_iter()
        .filter_map(|(name, candidates)| Some((name, widest(types, &candidates)?)))
        .collect()
}

fn widest(types: &Types, candidates: &[TypeRef]) -> Option<TypeRef> {
    candidates
        .iter()
        .find(|wide| candidates.iter().all(|narrow| covers(types, wide, narrow)))
        .cloned()
}

fn covers(types: &Types, wide: &TypeRef, narrow: &TypeRef) -> bool {
    wide == narrow
        || match (wide, narrow) {
            (TypeRef::Array(wide), TypeRef::Array(narrow)) => {
                !is_primitive(narrow) && covers(types, wide, narrow)
            }
            (TypeRef::Class { name, .. }, TypeRef::Array(_)) => {
                ARRAY_SUPERTYPES.contains(&name.as_str())
            }
            (TypeRef::Class { name, .. }, TypeRef::Class { .. }) => {
                name == OBJECT || closure(types, narrow).contains(wide)
            }
            _ => false,
        }
}

fn reference(type_ref: &TypeRef) -> TypeRef {
    match type_ref {
        TypeRef::Primitive(primitive) => {
            boxed(primitive).map_or_else(|| type_ref.clone(), TypeRef::class)
        }
        _ => type_ref.clone(),
    }
}

fn is_primitive(type_ref: &TypeRef) -> bool {
    matches!(type_ref, TypeRef::Primitive(_))
}
