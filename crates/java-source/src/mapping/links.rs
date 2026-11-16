use law_of_demeter_core::domain::classification::Types;
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

#[derive(Clone)]
pub(super) struct Declared {
    parameters: Vec<TypeRef>,
    returned: Option<TypeRef>,
    free: Vec<String>,
    elements: Option<String>,
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
        Self {
            parameters: expanded(method, argument_count),
            returned: method.returns().cloned(),
            free: method.type_parameters().to_vec(),
            elements,
        }
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

    pub(super) fn feeds_return(&self, types: &Types, index: usize) -> bool {
        let Some((parameter, returned)) = self.parameters.get(index).zip(self.returned.as_ref())
        else {
            return false;
        };
        let result = functional_method(types, parameter).and_then(|method| method.returns);
        let returned_variables = variables(returned);
        result
            .iter()
            .flat_map(variables)
            .any(|name| self.free.contains(&name) && returned_variables.contains(&name))
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
