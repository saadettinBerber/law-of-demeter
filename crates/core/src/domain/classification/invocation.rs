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
            (
                TypeRef::Class {
                    name: declared_name,
                    arguments: declared_arguments,
                },
                TypeRef::Class { .. },
            ) => {
                let matching = types
                    .ancestors(actual)
                    .into_iter()
                    .find(|ancestor| class_name(ancestor) == Some(declared_name));
                if let Some(TypeRef::Class {
                    arguments: actual_arguments,
                    ..
                }) = matching
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
