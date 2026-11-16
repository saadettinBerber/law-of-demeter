use law_of_demeter_core::domain::classification::{Classification, Types};
use law_of_demeter_core::domain::method::{Lambda, MethodReference};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeRef, TypeShape};

use super::lambdas::functional_method;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Feed {
    Elements,
    Receiver,
}

#[derive(Clone, Default)]
pub(super) struct ArgumentLinks {
    pub(super) feeds: Vec<Feed>,
    pub(super) inherited: Vec<String>,
}

impl ArgumentLinks {
    pub(super) fn on_lambda(&self, lambda: Lambda, names: &[&str]) -> Lambda {
        names
            .iter()
            .zip(&self.feeds)
            .fold(lambda, |lambda, (name, feed)| match feed {
                Feed::Elements => lambda.fed_by_elements(name),
                Feed::Receiver => lambda.fed_by_receiver(name),
            })
    }

    pub(super) fn on_reference(&self, reference: MethodReference) -> MethodReference {
        match self.feeds.first() {
            Some(Feed::Elements) => reference.fed_by_elements(),
            Some(Feed::Receiver) => reference.fed_by_receiver(),
            None => reference,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Flow {
    None,
    Return,
    Keys,
}

#[derive(Clone)]
pub(super) struct Declared {
    parameters: Vec<TypeRef>,
    returned: Option<TypeRef>,
    free: Vec<String>,
    elements: Option<String>,
    yields_keys: bool,
}

impl Declared {
    pub(super) fn of(
        types: &Types,
        method: &MethodShape,
        owner: &TypeShape,
        argument_count: usize,
    ) -> Self {
        let owner_parameters = owner.type_parameters();
        let owner_type = TypeRef::generic(
            owner.name(),
            owner_parameters
                .iter()
                .map(|name| TypeRef::variable(name))
                .collect(),
        );
        let elements = match types.element_type(&owner_type) {
            Some(TypeRef::Variable(name)) => Some(name.clone()),
            _ => None,
        };
        let yields_keys = match (
            method
                .returns()
                .and_then(|returned| types.element_type(returned)),
            owner_parameters,
        ) {
            (Some(TypeRef::Variable(name)), [key, _]) => name == key,
            _ => false,
        };
        Self {
            parameters: expanded(method, argument_count),
            returned: method.returns().cloned(),
            free: method.type_parameters().to_vec(),
            elements,
            yields_keys,
        }
    }

    pub(super) fn yields_keys(&self) -> bool {
        self.yields_keys
    }

    pub(super) fn produces_keys(&self, types: &Types) -> bool {
        self.returned
            .as_ref()
            .is_some_and(|returned| !key_variables(types, returned).is_empty())
    }

    pub(super) fn links(&self, types: &Types, index: usize, inherited: &[String]) -> ArgumentLinks {
        let fed = self.fed(inherited);
        let feeds = self
            .parameters
            .get(index)
            .and_then(|parameter| functional_method(types, parameter))
            .map(|functional| {
                functional
                    .parameters
                    .iter()
                    .map(|input| match input {
                        TypeRef::Variable(name) if fed.contains(name) => Feed::Elements,
                        _ => Feed::Receiver,
                    })
                    .collect()
            })
            .unwrap_or_default();
        ArgumentLinks {
            feeds,
            inherited: Vec::new(),
        }
    }

    pub(super) fn inherited_by(
        &self,
        index: usize,
        nested: &Declared,
        inherited: &[String],
    ) -> Vec<String> {
        let fed = self.fed(inherited);
        let mut aligned = Vec::new();
        if let Some((parameter, returned)) =
            self.parameters.get(index).zip(nested.returned.as_ref())
        {
            align(returned, parameter, &fed, &nested.free, &mut aligned);
        }
        aligned
    }

    pub(super) fn flow(&self, types: &Types, index: usize, functional: bool) -> Flow {
        let Some((parameter, returned)) = self.parameters.get(index).zip(self.returned.as_ref())
        else {
            return Flow::None;
        };
        let result = if functional {
            functional_method(types, parameter).and_then(|method| method.returns)
        } else {
            Some(parameter.clone())
        };
        let shared: Vec<_> = result
            .iter()
            .flat_map(variables)
            .filter(|name| self.free.contains(name) && variables(returned).contains(name))
            .collect();
        let keys = key_variables(types, returned);
        let carries_keys = shared.iter().any(|name| keys.contains(name))
            || (!functional
                && shared
                    .iter()
                    .any(|name| *returned == TypeRef::variable(name)));
        if carries_keys {
            Flow::Keys
        } else if shared.is_empty() {
            Flow::None
        } else {
            Flow::Return
        }
    }

    fn fed(&self, inherited: &[String]) -> Vec<String> {
        self.elements.iter().chain(inherited).cloned().collect()
    }
}

fn align(
    returned: &TypeRef,
    parameter: &TypeRef,
    fed: &[String],
    free: &[String],
    aligned: &mut Vec<String>,
) {
    match (returned, parameter) {
        (TypeRef::Variable(own), TypeRef::Variable(outer))
            if free.contains(own) && fed.contains(outer) =>
        {
            aligned.push(own.clone());
        }
        (
            TypeRef::Class {
                name: returned_name,
                arguments: returned_arguments,
            },
            TypeRef::Class {
                name: parameter_name,
                arguments: parameter_arguments,
            },
        ) if returned_name == parameter_name => {
            for (returned, parameter) in returned_arguments.iter().zip(parameter_arguments) {
                align(returned, parameter, fed, free, aligned);
            }
        }
        _ => {}
    }
}

fn key_variables(types: &Types, type_ref: &TypeRef) -> Vec<String> {
    match type_ref {
        TypeRef::Class { arguments, .. } => {
            let own = match arguments.as_slice() {
                [key, _] if types.classify(type_ref) == Classification::Container => variables(key),
                _ => Vec::new(),
            };
            own.into_iter()
                .chain(
                    arguments
                        .iter()
                        .flat_map(|argument| key_variables(types, argument)),
                )
                .collect()
        }
        TypeRef::Array(element) => key_variables(types, element),
        TypeRef::Variable(_) | TypeRef::Primitive(_) => Vec::new(),
    }
}

fn variables(type_ref: &TypeRef) -> Vec<String> {
    match type_ref {
        TypeRef::Variable(name) => vec![name.clone()],
        TypeRef::Class { arguments, .. } => arguments.iter().flat_map(variables).collect(),
        TypeRef::Array(element) => variables(element),
        TypeRef::Primitive(_) => Vec::new(),
    }
}

fn expanded(method: &MethodShape, argument_count: usize) -> Vec<TypeRef> {
    let parameters = method.parameters();
    match parameters.split_last() {
        Some((TypeRef::Array(element), fixed))
            if method.is_varargs() && parameters.len() != argument_count =>
        {
            fixed
                .iter()
                .cloned()
                .chain(std::iter::repeat((**element).clone()))
                .take(argument_count)
                .collect()
        }
        _ => parameters.to_vec(),
    }
}
