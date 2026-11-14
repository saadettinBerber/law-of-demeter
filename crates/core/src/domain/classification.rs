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
        let (first, rest) = overloads.split_first()?;
        rest.iter()
            .all(|overload| overload.returns() == first.returns())
            .then(|| first.returns())
            .flatten()
    }

    fn overloads(
        &self,
        type_name: &str,
        method: &str,
        parameter_count: usize,
    ) -> Vec<&MethodShape> {
        let Some(shape) = self.shape(type_name) else {
            return Vec::new();
        };
        let declared: Vec<_> = shape
            .methods()
            .iter()
            .filter(|candidate| {
                candidate.name() == method && candidate.parameters().len() == parameter_count
            })
            .collect();
        if !declared.is_empty() {
            return declared;
        }
        shape
            .supertypes()
            .iter()
            .filter_map(class_name)
            .map(|supertype| self.overloads(supertype, method, parameter_count))
            .find(|inherited| !inherited.is_empty())
            .unwrap_or_default()
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
            for name in current.supertypes().iter().filter_map(class_name) {
                match self.shape(name) {
                    Some(superclass) if superclass.kind() == TypeKind::Interface => {}
                    Some(superclass) => {
                        if !chain.iter().any(|known| known.name() == superclass.name()) {
                            chain.push(superclass);
                        }
                    }
                    None => complete &= jdk::is_known_supertype(name),
                }
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
        shape
            .supertypes()
            .iter()
            .map(|supertype| self.classify(supertype))
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
