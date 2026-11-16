use std::collections::HashMap;

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::type_shape::TypeRef;

use super::conversions::boxed;
use super::lineage::{
    ARRAY_SUPERTYPES, OBJECT, arguments_of, class_name, closure, has_variable, substitute,
};
use super::overloads::ArgumentType;

pub(super) type Bindings = HashMap<String, TypeRef>;

type Pairs = Vec<(String, TypeRef)>;

#[derive(Default)]
struct Constraints {
    lower: Pairs,
    equal: Pairs,
}

pub(super) fn instantiate(
    types: &Types,
    free: &[String],
    parameters: &[TypeRef],
    arguments: &[&ArgumentType],
) -> Option<(Vec<TypeRef>, Bindings)> {
    let mut constraints = Constraints::default();
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
    let bindings = resolve(types, [constraints.lower, constraints.equal].concat());
    let instantiated: Vec<_> = parameters
        .iter()
        .map(|parameter| apply_bindings(parameter, &bindings))
        .collect();
    let settled = instantiated
        .iter()
        .zip(arguments)
        .all(|(parameter, argument)| argument.is_deferred() || !has_variable(parameter));
    settled.then_some((instantiated, bindings))
}

pub(super) fn complete(
    types: &Types,
    free: &[String],
    returned: &TypeRef,
    from_arguments: &Bindings,
    expected: &TypeRef,
) -> Option<Bindings> {
    let mut constraints = Constraints::default();
    from_target(
        types,
        free,
        returned,
        &reference(expected),
        &mut constraints,
    );
    let mut completed = from_arguments.clone();
    for (name, wanted) in resolve(types, constraints.equal) {
        let fits = completed
            .get(&name)
            .is_none_or(|argument| covers(types, &wanted, argument));
        if !fits {
            return None;
        }
        completed.insert(name, wanted);
    }
    for (name, wanted) in resolve(types, constraints.lower) {
        completed.entry(name).or_insert(wanted);
    }
    Some(completed)
}

pub(super) fn bind_results(
    types: &Types,
    free: &[String],
    bound: &Bindings,
    results: &[(TypeRef, TypeRef)],
) -> Bindings {
    let mut constraints = Constraints::default();
    for (returned, result) in results {
        from_argument(types, free, returned, &reference(result), &mut constraints);
    }
    let mut bindings = bound.clone();
    bindings.extend(resolve(
        types,
        [constraints.lower, constraints.equal].concat(),
    ));
    bindings
}

pub(super) fn common_result(types: &Types, results: &[Option<TypeRef>]) -> Option<TypeRef> {
    let known: Option<Vec<TypeRef>> = results
        .iter()
        .map(|result| result.as_ref().map(reference))
        .collect();
    widest(types, &known?)
}

pub(super) fn apply_bindings(type_ref: &TypeRef, bindings: &Bindings) -> TypeRef {
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
            constraints.lower.push((name.clone(), argument.clone()));
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

fn from_target(
    types: &Types,
    free: &[String],
    returned: &TypeRef,
    expected: &TypeRef,
    constraints: &mut Constraints,
) {
    match (returned, expected) {
        (TypeRef::Variable(name), _) if free.contains(name) => {
            constraints.lower.push((name.clone(), expected.clone()));
        }
        (TypeRef::Array(element), TypeRef::Array(target)) if !is_primitive(target) => {
            from_target(types, free, element, target, constraints);
        }
        (TypeRef::Class { .. }, TypeRef::Class { name, arguments }) => {
            let declared = supertype_arguments(types, returned, name);
            for (declared, target) in declared.iter().zip(arguments) {
                from_equal(free, declared, target, constraints);
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
            constraints.equal.push((name.clone(), actual.clone()));
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

fn resolve(types: &Types, constraints: Pairs) -> Bindings {
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
