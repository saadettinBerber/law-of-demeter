use cafebabe::{ClassAccessFlags, ClassFile, parse_class};
use law_of_demeter_core::domain::type_shape::{TypeKind, TypeShape};

pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Option<TypeShape>, String> {
    let class = parse_class(bytes).map_err(|error| error.to_string())?;
    if class
        .access_flags
        .intersects(ClassAccessFlags::SYNTHETIC | ClassAccessFlags::MODULE)
    {
        return Ok(None);
    }
    Ok(Some(TypeShape::new(
        &binary_name(&class.this_class),
        kind_of(&class),
    )))
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
