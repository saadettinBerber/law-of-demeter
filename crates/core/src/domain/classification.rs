mod jdk;
mod naming;

pub use naming::{is_factory_method, is_repository_query};

use std::collections::{HashMap, HashSet};

use super::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classification {
    DataStructure,
    Object,
    Hybrid,
    Value,
    Container,
    Unknown,
}

#[derive(Debug, Clone, Default)]
pub struct Types {
    shapes: HashMap<String, TypeShape>,
}

impl Types {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_shape(mut self, shape: TypeShape) -> Self {
        self.shapes.insert(shape.name().to_owned(), shape);
        self
    }

    pub fn shape(&self, name: &str) -> Option<&TypeShape> {
        self.shapes.get(name)
    }

    pub fn return_type(
        &self,
        type_name: &str,
        method: &str,
        parameter_count: usize,
    ) -> Option<&TypeRef> {
        let overloads = self.overloads(type_name, method, parameter_count);
        unambiguous_return(&overloads)
    }

    pub fn return_type_on(
        &self,
        receiver: &TypeRef,
        method: &str,
        parameter_count: usize,
    ) -> Option<TypeRef> {
        let (declaring, overloads) = self.overloads_on(receiver, method, parameter_count)?;
        let returned = unambiguous_return(&overloads)?;
        let (_, bindings) = self.bound(&declaring)?;
        Some(substitute(returned, &bindings))
    }

    fn overloads(
        &self,
        type_name: &str,
        method: &str,
        parameter_count: usize,
    ) -> Vec<&MethodShape> {
        self.overloads_on(&TypeRef::class(type_name), method, parameter_count)
            .map(|(_, overloads)| overloads)
            .unwrap_or_default()
    }

    fn overloads_on(
        &self,
        receiver: &TypeRef,
        method: &str,
        parameter_count: usize,
    ) -> Option<(TypeRef, Vec<&MethodShape>)> {
        self.ancestors(receiver).into_iter().find_map(|ancestor| {
            let shape = self.shape(class_name(&ancestor)?)?;
            let declared: Vec<_> = shape
                .methods()
                .iter()
                .filter(|candidate| {
                    candidate.name() == method && candidate.parameters().len() == parameter_count
                })
                .collect();
            (!declared.is_empty()).then_some((ancestor, declared))
        })
    }

    fn ancestors(&self, receiver: &TypeRef) -> Vec<TypeRef> {
        let mut found = vec![receiver.clone()];
        let mut index = 0;
        while let Some(current) = found.get(index) {
            index += 1;
            let Some((shape, bindings)) = self.bound(current) else {
                continue;
            };
            let unseen: Vec<_> = shape
                .supertypes()
                .iter()
                .map(|supertype| substitute(supertype, &bindings))
                .filter(|supertype| {
                    !found
                        .iter()
                        .any(|known| class_name(known) == class_name(supertype))
                })
                .collect();
            found.extend(unseen);
        }
        found
    }

    fn bound(&self, type_ref: &TypeRef) -> Option<(&TypeShape, HashMap<String, TypeRef>)> {
        let TypeRef::Class { name, arguments } = type_ref else {
            return None;
        };
        let shape = self.shape(name)?;
        let bindings = if shape.type_parameters().len() == arguments.len() {
            shape
                .type_parameters()
                .iter()
                .cloned()
                .zip(arguments.iter().cloned())
                .collect()
        } else {
            HashMap::new()
        };
        Some((shape, bindings))
    }

    pub fn is_fluent(&self, type_name: &str, method: &str, parameter_count: usize) -> bool {
        let overloads = self.overloads(type_name, method, parameter_count);
        !overloads.is_empty()
            && overloads.iter().all(|overload| {
                !overload.is_static() && overload.returns().and_then(class_name) == Some(type_name)
            })
    }

    pub fn hybrid_class(&self, name: &str) -> Option<HybridClass> {
        let shape = self.shape(name)?;
        (self.classify_named(name) == Classification::Hybrid).then(|| {
            let methods = self.class_members(shape).methods;
            HybridClass::new(
                name,
                distinct_names(methods.iter().filter(|method| is_behavior(method))),
                distinct_names(methods.iter().filter(|method| is_setter(method))),
            )
        })
    }

    pub fn classify(&self, type_ref: &TypeRef) -> Classification {
        match type_ref {
            TypeRef::Class { name, .. } => self.classify_named(name),
            TypeRef::Primitive(_) => Classification::Value,
            TypeRef::Array(_) => Classification::Container,
            TypeRef::Variable(_) => Classification::Unknown,
        }
    }

    fn classify_named(&self, name: &str) -> Classification {
        jdk::classify(name).unwrap_or_else(|| {
            self.shapes
                .get(name)
                .map_or(Classification::Unknown, |shape| self.classify_shape(shape))
        })
    }

    fn classify_shape(&self, shape: &TypeShape) -> Classification {
        self.inherited_value_or_container(shape)
            .unwrap_or_else(|| match shape.kind() {
                TypeKind::Record => Classification::DataStructure,
                TypeKind::Class => self.classify_class(shape),
                TypeKind::Enum => Classification::Value,
                TypeKind::Interface => Classification::Object,
            })
    }

    fn class_members<'a>(&'a self, shape: &'a TypeShape) -> Members<'a> {
        let mut chain = vec![shape];
        let mut complete = true;
        let mut index = 0;
        while let Some(&current) = chain.get(index) {
            index += 1;
            let Some(name) = current.superclass().and_then(class_name) else {
                continue;
            };
            if jdk::is_behaviorless(name) {
                continue;
            }
            match self.shape(name) {
                Some(superclass) => {
                    if !chain.iter().any(|known| known.name() == superclass.name()) {
                        chain.push(superclass);
                    }
                }
                None => complete &= jdk::classify(name).is_some(),
            }
        }
        let mut methods: Vec<&MethodShape> = Vec::new();
        let mut hidden: HashSet<(&str, usize)> = HashSet::new();
        for class in chain {
            let declared: Vec<_> = class
                .methods()
                .iter()
                .filter(|method| is_exposed(method))
                .collect();
            methods.extend(
                declared
                    .iter()
                    .filter(|method| !hidden.contains(&signature(method))),
            );
            hidden.extend(declared.iter().map(|method| signature(method)));
        }
        Members { methods, complete }
    }

    fn classify_class(&self, shape: &TypeShape) -> Classification {
        let members = self.class_members(shape);
        let has_setter = members.methods.iter().any(|method| is_setter(method));
        let has_behavior = members.methods.iter().any(|method| is_behavior(method));
        match (has_behavior, has_setter, members.complete) {
            (true, true, _) => Classification::Hybrid,
            (true, false, _) => Classification::Object,
            (false, _, true) => Classification::DataStructure,
            (false, _, false) => Classification::Unknown,
        }
    }

    fn inherited_value_or_container(&self, shape: &TypeShape) -> Option<Classification> {
        self.ancestors(&TypeRef::class(shape.name()))
            .iter()
            .skip(1)
            .filter_map(class_name)
            .filter_map(jdk::classify)
            .find(|classification| {
                matches!(
                    classification,
                    Classification::Value | Classification::Container
                )
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HybridClass {
    class: String,
    behavior_methods: Vec<String>,
    setters: Vec<String>,
}

impl HybridClass {
    fn new(class: &str, behavior_methods: Vec<String>, setters: Vec<String>) -> Self {
        Self {
            class: class.to_owned(),
            behavior_methods,
            setters,
        }
    }

    pub fn class(&self) -> &str {
        &self.class
    }

    pub fn behavior_methods(&self) -> &[String] {
        &self.behavior_methods
    }

    pub fn setters(&self) -> &[String] {
        &self.setters
    }
}

struct Members<'a> {
    methods: Vec<&'a MethodShape>,
    complete: bool,
}

pub fn element_type(type_ref: &TypeRef) -> Option<&TypeRef> {
    match type_ref {
        TypeRef::Array(element) => Some(element),
        TypeRef::Class { name, arguments }
            if jdk::classify(name) == Some(Classification::Container) =>
        {
            arguments.last()
        }
        TypeRef::Class { .. } | TypeRef::Primitive(_) | TypeRef::Variable(_) => None,
    }
}

fn unambiguous_return<'a>(overloads: &[&'a MethodShape]) -> Option<&'a TypeRef> {
    let (first, rest) = overloads.split_first()?;
    rest.iter()
        .all(|overload| overload.returns() == first.returns())
        .then(|| first.returns())
        .flatten()
}

fn substitute(type_ref: &TypeRef, bindings: &HashMap<String, TypeRef>) -> TypeRef {
    match type_ref {
        TypeRef::Variable(name) => bindings
            .get(name)
            .cloned()
            .unwrap_or_else(|| type_ref.clone()),
        TypeRef::Class { name, arguments } => TypeRef::Class {
            name: name.clone(),
            arguments: arguments
                .iter()
                .map(|argument| substitute(argument, bindings))
                .collect(),
        },
        TypeRef::Array(element) => TypeRef::Array(Box::new(substitute(element, bindings))),
        TypeRef::Primitive(_) => type_ref.clone(),
    }
}

fn class_name(type_ref: &TypeRef) -> Option<&str> {
    match type_ref {
        TypeRef::Class { name, .. } => Some(name),
        TypeRef::Primitive(_) | TypeRef::Array(_) | TypeRef::Variable(_) => None,
    }
}

fn is_exposed(method: &MethodShape) -> bool {
    let is_object_method = matches!(
        (method.name(), method.parameters().len()),
        ("equals", 1) | ("hashCode" | "toString", 0)
    );
    method.is_public() && !method.is_static() && !is_object_method && method.name() != "<init>"
}

fn signature(method: &MethodShape) -> (&str, usize) {
    (method.name(), method.parameters().len())
}

fn is_behavior(method: &MethodShape) -> bool {
    !is_getter(method) && !is_setter(method)
}

fn distinct_names<'a>(methods: impl Iterator<Item = &'a &'a MethodShape>) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for method in methods {
        if !names.iter().any(|name| name == method.name()) {
            names.push(method.name().to_owned());
        }
    }
    names
}

fn is_getter(method: &MethodShape) -> bool {
    method.reads_field().is_some() && method.parameters().is_empty()
}

fn is_setter(method: &MethodShape) -> bool {
    method.writes_field().is_some() && method.parameters().len() == 1
}

#[cfg(test)]
mod tests;
