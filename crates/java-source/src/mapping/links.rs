use law_of_demeter_core::domain::classification::{Classification, Types};
use law_of_demeter_core::domain::method::{Lambda, MethodReference};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeRef, TypeShape};

use super::lambdas::functional_method;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Feed {
    Elements,
    Receiver,
    Argument(usize),
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
                Feed::Argument(index) => lambda.fed_by_argument_elements(name, *index),
            })
    }

    pub(super) fn on_reference(&self, reference: MethodReference) -> MethodReference {
        match self.feeds.first() {
            Some(Feed::Elements) => reference.fed_by_elements(),
            Some(Feed::Receiver) => reference.fed_by_receiver(),
            Some(Feed::Argument(index)) => reference.fed_by_argument_elements(*index),
            None => reference,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Flow {
    None,
    Return,
    Keys,
    Values,
    Whole,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Yield {
    Nothing,
    Keys,
    Values,
    Entries,
}

#[derive(Clone)]
pub(super) struct Declared {
    parameters: Vec<TypeRef>,
    returned: Option<TypeRef>,
    free: Vec<String>,
    elements: Option<String>,
    yields: Yield,
    returns_receiver: bool,
    fixed_dispatch: bool,
    returns_exact_type: bool,
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
        let yields = yield_of(types, method, owner, elements.as_deref());
        Self {
            parameters: expanded(method, argument_count),
            returned: method.returns().cloned(),
            free: method.type_parameters().to_vec(),
            elements,
            yields,
            returns_receiver: method.returns_receiver(),
            fixed_dispatch: owner.is_final()
                || method.is_final()
                || method.is_private()
                || method.is_static(),
            returns_exact_type: method.returns_exact_type(),
        }
    }

    pub(super) fn yields(&self) -> Yield {
        self.yields
    }

    pub(super) fn returns_receiver(&self) -> bool {
        self.returns_receiver
    }

    pub(super) fn has_fixed_dispatch(&self) -> bool {
        self.fixed_dispatch
    }

    pub(super) fn returns_exact_type(&self) -> bool {
        self.returns_exact_type
    }

    pub(super) fn produces_keys(&self, types: &Types) -> bool {
        self.returned
            .as_ref()
            .is_some_and(|returned| !map_variables(types, returned, 0).is_empty())
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
                        TypeRef::Variable(name) => self
                            .argument_binding(types, name)
                            .map_or(Feed::Receiver, Feed::Argument),
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

    fn argument_binding(&self, types: &Types, name: &str) -> Option<usize> {
        let bound = TypeRef::variable(name);
        self.parameters
            .iter()
            .position(|parameter| sole_element(types, parameter) == Some(&bound))
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

    pub(super) fn result_flow(&self, types: &Types, index: usize) -> Flow {
        let result = self
            .parameters
            .get(index)
            .and_then(|parameter| functional_method(types, parameter))
            .and_then(|method| method.returns);
        self.flow(types, result, |_| false)
    }

    pub(super) fn value_flow(&self, types: &Types, index: usize) -> Flow {
        let returned = self.returned.as_ref();
        let whole = |name: &String| returned == Some(&TypeRef::variable(name));
        self.flow(types, self.parameters.get(index).cloned(), whole)
    }

    fn flow(
        &self,
        types: &Types,
        result: Option<TypeRef>,
        is_whole_return: impl Fn(&String) -> bool,
    ) -> Flow {
        let (Some(result), Some(returned)) = (result, self.returned.as_ref()) else {
            return Flow::None;
        };
        let returned_variables = variables(returned);
        let shared: Vec<_> = variables(&result)
            .into_iter()
            .filter(|name| self.free.contains(name) && returned_variables.contains(name))
            .collect();
        let positioned = |index| {
            let positions = map_variables(types, returned, index);
            move |name: &String| positions.contains(name)
        };
        let (keys, values) = (positioned(0), positioned(1));
        if shared.iter().any(is_whole_return) {
            Flow::Whole
        } else if shared.iter().any(keys) {
            Flow::Keys
        } else if shared.iter().any(values) {
            Flow::Values
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

fn sole_element<'t>(types: &Types, type_ref: &'t TypeRef) -> Option<&'t TypeRef> {
    match type_ref {
        TypeRef::Class { arguments, .. } if arguments.len() != 1 => None,
        _ => types.element_type(type_ref),
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

fn map_variables(types: &Types, type_ref: &TypeRef, position: usize) -> Vec<String> {
    match type_ref {
        TypeRef::Class { arguments, .. } => {
            let own = match arguments.as_slice() {
                [key, value] if types.classify(type_ref) == Classification::Container => {
                    variables(if position == 0 { key } else { value })
                }
                _ => Vec::new(),
            };
            own.into_iter()
                .chain(
                    arguments
                        .iter()
                        .flat_map(|argument| map_variables(types, argument, position)),
                )
                .collect()
        }
        TypeRef::Array(element) => map_variables(types, element, position),
        TypeRef::Variable(_) | TypeRef::Primitive(_) => Vec::new(),
    }
}

fn yield_of(
    types: &Types,
    method: &MethodShape,
    owner: &TypeShape,
    elements: Option<&str>,
) -> Yield {
    let returned = method.returns();
    let element = returned.and_then(|returned| types.element_type(returned));
    let read = match (returned, element) {
        (Some(TypeRef::Variable(name)), _) | (_, Some(TypeRef::Variable(name))) => Some(name),
        _ => None,
    };
    match (owner.type_parameters(), read, element) {
        ([key, _], Some(name), _) if name == key => Yield::Keys,
        ([_, value], Some(name), _) if name == value => Yield::Values,
        ([key, value], _, Some(TypeRef::Class { arguments, .. }))
            if arguments == &[TypeRef::variable(key), TypeRef::variable(value)] =>
        {
            Yield::Entries
        }
        (_, Some(name), _) if elements == Some(name.as_str()) => Yield::Entries,
        _ => Yield::Nothing,
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
