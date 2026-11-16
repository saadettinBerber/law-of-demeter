use super::{Classification, KnownTypes, Types};

pub(crate) fn sample_types() -> Types {
    Types::new().with_known(sample_platform())
}

fn sample_platform() -> KnownTypes {
    let types = [
        (VALUES, Classification::Value),
        (CONTAINERS, Classification::Container),
        (DATA_STRUCTURES, Classification::DataStructure),
    ];
    let known = types
        .into_iter()
        .flat_map(|(names, classification)| names.iter().map(move |name| (name, classification)))
        .fold(KnownTypes::new(), |known, (name, classification)| {
            known.with_type(name, classification)
        });
    let known = known
        .with_namespace("java.time.", Classification::Value)
        .with_namespace("java.util.stream.", Classification::Container);
    BEHAVIORLESS_SUPERTYPES
        .iter()
        .fold(known, |known, name| known.with_behaviorless_supertype(name))
}

const VALUES: &[&str] = &[
    "java.lang.Class",
    "java.lang.Number",
    "java.lang.String",
    "java.lang.StringBuilder",
    "java.math.BigDecimal",
    "java.util.Date",
];

const CONTAINERS: &[&str] = &[
    "java.lang.Iterable",
    "java.util.ArrayList",
    "java.util.Collection",
    "java.util.HashMap",
    "java.util.Iterator",
    "java.util.List",
    "java.util.Map",
    "java.util.Optional",
    "java.util.Set",
];

const DATA_STRUCTURES: &[&str] = &["java.util.Map$Entry"];

const BEHAVIORLESS_SUPERTYPES: &[&str] =
    &["java.lang.Enum", "java.lang.Object", "java.lang.Record"];
