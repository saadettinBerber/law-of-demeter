use law_of_demeter_core::domain::type_shape::TypeRef;

use crate::{DeclaredType, TypeName};

pub(super) fn type_ref(declared: &DeclaredType) -> Option<TypeRef> {
    match declared {
        DeclaredType::Var => None,
        DeclaredType::Primitive(name) => Some(TypeRef::primitive(name)),
        DeclaredType::Class { name, arguments } => Some(TypeRef::generic(
            type_name(name),
            arguments.iter().map(type_ref_or_object).collect(),
        )),
        DeclaredType::Array(element) => Some(TypeRef::array(type_ref_or_object(element))),
        DeclaredType::Variable(name) => Some(TypeRef::variable(name)),
        DeclaredType::Wildcard(upper_bound) => Some(
            upper_bound
                .as_deref()
                .map_or_else(object, type_ref_or_object),
        ),
    }
}

pub(super) fn type_ref_or_object(declared: &DeclaredType) -> TypeRef {
    type_ref(declared).unwrap_or_else(object)
}

pub(super) fn named_type(name: &TypeName) -> TypeRef {
    TypeRef::class(type_name(name))
}

pub(super) fn object() -> TypeRef {
    TypeRef::class("java.lang.Object")
}

fn type_name(name: &TypeName) -> &str {
    match name {
        TypeName::Resolved(name) | TypeName::Unresolved(name) => name,
    }
}
