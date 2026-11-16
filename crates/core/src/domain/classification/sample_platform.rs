use super::{Classification, Platform, Types};

pub(crate) fn sample_types() -> Types {
    Types::new().with_platform(sample_platform())
}

fn sample_platform() -> Platform {
    let types = [
        (VALUES, Classification::Value),
        (CONTAINERS, Classification::Container),
        (DATA_STRUCTURES, Classification::DataStructure),
    ];
    let platform = types
        .into_iter()
        .flat_map(|(names, classification)| names.iter().map(move |name| (name, classification)))
        .fold(Platform::new(), |platform, (name, classification)| {
            platform.with_type(name, classification)
        });
    let platform = platform
        .with_namespace("java.time.", Classification::Value)
        .with_namespace("java.util.stream.", Classification::Container);
    BEHAVIORLESS_SUPERTYPES
        .iter()
        .fold(platform, |platform, name| {
            platform.with_behaviorless_supertype(name)
        })
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
