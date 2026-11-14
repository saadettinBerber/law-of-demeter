use cafebabe::attributes::{AttributeData, AttributeInfo};
use cafebabe::{ClassAccessFlags, ClassFile, FieldAccessFlags, FieldInfo, parse_class};
use law_of_demeter_core::domain::type_shape::{FieldShape, TypeKind, TypeRef, TypeShape};

use crate::signature;

pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Option<TypeShape>, String> {
    let class = parse_class(bytes).map_err(|error| error.to_string())?;
    if class
        .access_flags
        .intersects(ClassAccessFlags::SYNTHETIC | ClassAccessFlags::MODULE)
    {
        return Ok(None);
    }
    let mut shape = TypeShape::new(&binary_name(&class.this_class), kind_of(&class));
    for supertype in supertypes_of(&class)? {
        shape = shape.extending(supertype);
    }
    for field in &class.fields {
        if let Some(field) = field_of(field)? {
            shape = shape.with_field(field);
        }
    }
    Ok(Some(shape))
}

fn binary_name(internal_name: &str) -> String {
    internal_name.replace('/', ".")
}

fn kind_of(class: &ClassFile) -> TypeKind {
    if class.access_flags.contains(ClassAccessFlags::INTERFACE) {
        TypeKind::Interface
    } else if class.access_flags.contains(ClassAccessFlags::ENUM) {
        TypeKind::Enum
    } else if class.super_class.as_deref() == Some("java/lang/Record") {
        TypeKind::Record
    } else {
        TypeKind::Class
    }
}

fn supertypes_of(class: &ClassFile) -> Result<Vec<TypeRef>, String> {
    let supertypes = match signature_of(&class.attributes) {
        Some(text) => signature::class_supertypes(text)?,
        None => class
            .super_class
            .iter()
            .chain(&class.interfaces)
            .map(|name| TypeRef::class(&binary_name(name)))
            .collect(),
    };
    let object = TypeRef::class("java.lang.Object");
    Ok(supertypes
        .into_iter()
        .filter(|supertype| *supertype != object)
        .collect())
}

fn field_of(field: &FieldInfo) -> Result<Option<FieldShape>, String> {
    if field.access_flags.contains(FieldAccessFlags::SYNTHETIC) {
        return Ok(None);
    }
    let descriptor = field.descriptor.to_string();
    let text = signature_of(&field.attributes).unwrap_or(&descriptor);
    let type_ref =
        signature::field_type(text).map_err(|error| format!("field {}: {error}", field.name))?;
    let shape = FieldShape::new(&field.name, type_ref);
    Ok(Some(
        if field.access_flags.contains(FieldAccessFlags::STATIC) {
            shape.as_static()
        } else {
            shape
        },
    ))
}

fn signature_of<'a>(attributes: &'a [AttributeInfo]) -> Option<&'a str> {
    attributes
        .iter()
        .find_map(|attribute| match &attribute.data {
            AttributeData::Signature(text) => Some(&**text),
            _ => None,
        })
}
