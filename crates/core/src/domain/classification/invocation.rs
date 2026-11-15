use std::collections::HashMap;

use super::{FunctionalMethod, Types, class_name, substitute};
use crate::domain::type_shape::{MethodShape, TypeRef};

pub(crate) struct Invocation {
    parameters: Vec<TypeRef>,
    returns: Option<TypeRef>,
    free: Vec<String>,
    bindings: HashMap<String, TypeRef>,
}

impl Invocation {
    pub(crate) fn unknown() -> Self {
        Self {
            parameters: Vec::new(),
            returns: None,
            free: Vec::new(),
            bindings: HashMap::new(),
        }
    }

    pub(super) fn ambiguous(returns: Option<TypeRef>) -> Self {
        Self {
            returns,
            ..Self::unknown()
        }
    }

    pub(super) fn of(
        types: &Types,
        method: &MethodShape,
        class_bindings: &HashMap<String, TypeRef>,
        values: &[Option<&TypeRef>],
    ) -> Self {
        let free = method.type_parameters().to_vec();
        let class_bindings: HashMap<_, _> = class_bindings
            .iter()
            .filter(|(name, _)| !free.contains(name))
            .map(|(name, type_ref)| (name.clone(), type_ref.clone()))
            .collect();
        let parameters: Vec<_> = method
            .parameters()
            .iter()
            .map(|parameter| substitute(parameter, &class_bindings))
            .collect();
        let mut invocation = Self {
            parameters,
            returns: method
                .returns()
                .map(|returned| substitute(returned, &class_bindings)),
            free,
            bindings: HashMap::new(),
        };
        for (index, value) in values.iter().enumerate() {
            if let (Some(parameter), Some(value)) =
                (invocation.parameters.get(index).cloned(), value)
            {
                invocation.unify(types, &parameter, value);
            }
        }
        invocation
    }

    pub(crate) fn expecting(mut self, types: &Types, expected: Option<&TypeRef>) -> Self {
        if let Some((returns, expected)) = self.returns.clone().zip(expected) {
            self.unify(types, &returns, expected);
        }
        self
    }

    pub(crate) fn yielded(&mut self, types: &Types, index: usize, result: Option<&TypeRef>) {
        let returned = self
            .functional_method(types, index)
            .and_then(|functional| functional.returns);
        if let Some((returned, result)) = returned.zip(result) {
            self.unify(types, &returned, result);
        }
    }

    pub(crate) fn functional_method(
        &self,
        types: &Types,
        index: usize,
    ) -> Option<FunctionalMethod> {
        let parameter = self.parameters.get(index)?;
        types.functional_method(&substitute(parameter, &self.bindings))
    }

    pub(crate) fn returns(&self) -> Option<TypeRef> {
        self.returns
            .as_ref()
            .map(|returned| substitute(returned, &self.bindings))
    }

    fn unify(&mut self, types: &Types, declared: &TypeRef, actual: &TypeRef) {
        match (declared, actual) {
            (TypeRef::Variable(name), _) if self.free.contains(name) => {
                self.bindings
                    .entry(name.clone())
                    .or_insert_with(|| actual.clone());
            }
            (TypeRef::Array(declared), TypeRef::Array(actual)) => {
                self.unify(types, declared, actual);
            }
            (TypeRef::Class { .. }, TypeRef::Class { .. }) => {
                if let Some((declared_arguments, actual_arguments)) =
                    common_arguments(types, declared, actual)
                {
                    for (declared, actual) in declared_arguments.iter().zip(&actual_arguments) {
                        self.unify(types, declared, actual);
                    }
                }
            }
            _ => {}
        }
    }
}

pub(super) fn applicable<'a>(
    types: &Types,
    overloads: Vec<&'a MethodShape>,
    class_bindings: &HashMap<String, TypeRef>,
    values: &[Option<&TypeRef>],
) -> Vec<&'a MethodShape> {
    if overloads.len() < 2 {
        return overloads;
    }
    let fitting: Vec<_> = overloads
        .iter()
        .copied()
        .filter(|method| {
            method
                .parameters()
                .iter()
                .zip(values)
                .all(|(parameter, value)| {
                    value.is_none_or(|value| {
                        accepts(types, &substitute(parameter, class_bindings), value)
                    })
                })
        })
        .collect();
    if fitting.is_empty() {
        overloads
    } else {
        fitting
    }
}

fn accepts(types: &Types, parameter: &TypeRef, value: &TypeRef) -> bool {
    match (parameter, value) {
        (TypeRef::Variable(_), _) | (_, TypeRef::Variable(_)) => true,
        (TypeRef::Array(parameter), TypeRef::Array(value)) => accepts(types, parameter, value),
        (TypeRef::Primitive(parameter), TypeRef::Primitive(value)) => parameter == value,
        (TypeRef::Class { name, .. }, TypeRef::Array(_))
        | (TypeRef::Array(_), TypeRef::Class { name, .. }) => name == "java.lang.Object",
        (TypeRef::Primitive(_), TypeRef::Array(_) | TypeRef::Class { .. })
        | (TypeRef::Array(_) | TypeRef::Class { .. }, TypeRef::Primitive(_)) => false,
        (TypeRef::Class { name, .. }, TypeRef::Class { .. }) => {
            let ancestors = types.ancestors(value);
            name == "java.lang.Object"
                || ancestors
                    .iter()
                    .filter_map(class_name)
                    .any(|ancestor| ancestor == name || types.shape(ancestor).is_none())
        }
    }
}

fn common_arguments(
    types: &Types,
    declared: &TypeRef,
    actual: &TypeRef,
) -> Option<(Vec<TypeRef>, Vec<TypeRef>)> {
    let (declared_name, declared_arguments) = class_parts(declared)?;
    let (actual_name, actual_arguments) = class_parts(actual)?;
    match arguments_in_ancestors(types, actual, declared_name) {
        Some(actual_arguments) => Some((declared_arguments.to_vec(), actual_arguments)),
        None => arguments_in_ancestors(types, declared, actual_name)
            .map(|declared_arguments| (declared_arguments, actual_arguments.to_vec())),
    }
}

fn class_parts(type_ref: &TypeRef) -> Option<(&str, &[TypeRef])> {
    match type_ref {
        TypeRef::Class { name, arguments } => Some((name, arguments)),
        TypeRef::Primitive(_) | TypeRef::Array(_) | TypeRef::Variable(_) => None,
    }
}

fn arguments_in_ancestors(types: &Types, type_ref: &TypeRef, name: &str) -> Option<Vec<TypeRef>> {
    types
        .ancestors(type_ref)
        .into_iter()
        .find_map(|ancestor| match ancestor {
            TypeRef::Class {
                name: ancestor_name,
                arguments,
            } if ancestor_name == name => Some(arguments),
            _ => None,
        })
}
