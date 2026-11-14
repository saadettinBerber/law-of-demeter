use cafebabe::attributes::{AttributeData, AttributeInfo};
use cafebabe::descriptors::FieldType;
use cafebabe::{
    ClassAccessFlags, ClassFile, FieldAccessFlags, FieldInfo, MethodAccessFlags, MethodInfo,
    ParseOptions, parse_class_with_options,
};
use law_of_demeter_core::domain::type_shape::{
    FieldShape, MethodShape, TypeKind, TypeRef, TypeShape,
};

use crate::accessor::{self, Access};
use crate::signature;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Scope {
    All,
    PublicApi,
}

pub(crate) fn from_bytes(bytes: &[u8], scope: Scope) -> Result<Option<TypeShape>, String> {
    let mut options = ParseOptions::default();
    options.parse_bytecode(scope == Scope::All);
    let class = parse_class_with_options(bytes, &options).map_err(|error| error.to_string())?;
    if class
        .access_flags
        .intersects(ClassAccessFlags::SYNTHETIC | ClassAccessFlags::MODULE)
    {
        return Ok(None);
    }
    if scope == Scope::PublicApi && !class.access_flags.contains(ClassAccessFlags::PUBLIC) {
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
    for method in &class.methods {
        if let Some(method) = method_of(&class, method)? {
            shape = shape.with_method(method);
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

fn method_of(class: &ClassFile, method: &MethodInfo) -> Result<Option<MethodShape>, String> {
    let skipped = MethodAccessFlags::SYNTHETIC | MethodAccessFlags::BRIDGE;
    if method.access_flags.intersects(skipped) || ["<init>", "<clinit>"].contains(&&*method.name) {
        return Ok(None);
    }
    let descriptor = method.descriptor.to_string();
    let text = signature_of(&method.attributes).unwrap_or(&descriptor);
    let (parameters, returns) = signature::method_types(text)
        .map_err(|error| format!("method {}: {error}", method.name))?;
    let mut shape = MethodShape::new(&method.name).with_parameters(parameters);
    if let Some(returns) = returns {
        shape = shape.returning(returns);
    }
    if method.access_flags.contains(MethodAccessFlags::STATIC) {
        shape = shape.as_static();
    }
    if !method.access_flags.contains(MethodAccessFlags::PUBLIC) {
        shape = shape.not_public();
    }
    if is_lombok_generated(&method.attributes) {
        shape = shape.generated();
    }
    shape = match accessor::of(class, method) {
        Some(Access::Read(field)) => shape.reading(&field),
        Some(Access::Write(field)) => shape.writing(&field),
        None => shape,
    };
    Ok(Some(shape))
}

fn is_lombok_generated(attributes: &[AttributeInfo]) -> bool {
    attributes.iter().any(|attribute| match &attribute.data {
        AttributeData::RuntimeInvisibleAnnotations(annotations) => {
            annotations.iter().any(|annotation| {
                matches!(
                    &annotation.type_descriptor.field_type,
                    FieldType::Object(name) if &**name == "lombok/Generated"
                )
            })
        }
        _ => false,
    })
}

fn signature_of<'a>(attributes: &'a [AttributeInfo]) -> Option<&'a str> {
    attributes
        .iter()
        .find_map(|attribute| match &attribute.data {
            AttributeData::Signature(text) => Some(&**text),
            _ => None,
        })
}
