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

    pub fn classify(&self, type_ref: &TypeRef) -> Classification {
        match type_ref {
            TypeRef::Class { name, .. } => self
                .shapes
                .get(name)
                .map_or(Classification::Unknown, classify_shape),
            TypeRef::Primitive(_) | TypeRef::Array(_) | TypeRef::Variable(_) => {
                Classification::Unknown
            }
        }
    }
}

fn classify_shape(shape: &TypeShape) -> Classification {
    match shape.kind() {
        TypeKind::Record => Classification::DataStructure,
        TypeKind::Class => classify_class(shape),
        TypeKind::Enum | TypeKind::Interface => Classification::Unknown,
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
