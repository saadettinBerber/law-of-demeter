use law_of_demeter_core::domain::type_shape::TypeRef;

const WIDENINGS: &[(&str, &[&str])] = &[
    ("byte", &["short", "int", "long", "float", "double"]),
    ("short", &["int", "long", "float", "double"]),
    ("char", &["int", "long", "float", "double"]),
    ("int", &["long", "float", "double"]),
    ("long", &["float", "double"]),
    ("float", &["double"]),
];

const BOXES: &[(&str, &str)] = &[
    ("boolean", "java.lang.Boolean"),
    ("byte", "java.lang.Byte"),
    ("short", "java.lang.Short"),
    ("char", "java.lang.Character"),
    ("int", "java.lang.Integer"),
    ("long", "java.lang.Long"),
    ("float", "java.lang.Float"),
    ("double", "java.lang.Double"),
];

pub(super) fn widens(from: &str, to: &str) -> bool {
    WIDENINGS
        .iter()
        .any(|(narrow, wider)| *narrow == from && wider.contains(&to))
}

pub(super) fn boxed(primitive: &str) -> Option<&'static str> {
    BOXES
        .iter()
        .find_map(|(name, class)| (*name == primitive).then_some(*class))
}

pub(super) fn unboxed(class: &str) -> Option<&'static str> {
    BOXES
        .iter()
        .find_map(|(name, boxed)| (*boxed == class).then_some(*name))
}

pub(super) fn is_primitive(type_ref: &TypeRef) -> bool {
    matches!(type_ref, TypeRef::Primitive(_))
}
