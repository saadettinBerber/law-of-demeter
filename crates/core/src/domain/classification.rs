use std::collections::HashMap;

use super::type_shape::{TypeKind, TypeRef, TypeShape};

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
        TypeKind::Record | TypeKind::Class => Classification::DataStructure,
        TypeKind::Enum | TypeKind::Interface => Classification::Unknown,
    }
}

#[cfg(test)]
mod tests;
