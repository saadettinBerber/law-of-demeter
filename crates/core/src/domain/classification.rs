mod jdk;
mod naming;

pub use naming::{is_factory_method, is_repository_query};

use std::collections::HashMap;

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
        self.inherited_jdk_classification(shape)
            .unwrap_or_else(|| classify_kind(shape))
    }

    fn inherited_jdk_classification(&self, shape: &TypeShape) -> Option<Classification> {
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

fn classify_kind(shape: &TypeShape) -> Classification {
    match shape.kind() {
        TypeKind::Record => Classification::DataStructure,
        TypeKind::Class => classify_class(shape),
        TypeKind::Enum => Classification::Value,
        TypeKind::Interface => Classification::Object,
    }
}

fn classify_class(shape: &TypeShape) -> Classification {
    let exposed = || shape.methods().iter().filter(|method| is_exposed(method));
    let has_setter = exposed().any(is_setter);
    let has_behavior = exposed().any(|method| !is_getter(method) && !is_setter(method));
    match (has_behavior, has_setter) {
        (true, true) => Classification::Hybrid,
        (true, false) => Classification::Object,
        (false, _) => Classification::DataStructure,
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

fn is_getter(method: &MethodShape) -> bool {
    method.reads_field().is_some() && method.parameters().is_empty()
}

fn is_setter(method: &MethodShape) -> bool {
    method.writes_field().is_some() && method.parameters().len() == 1
}

#[cfg(test)]
mod tests;
