#![allow(dead_code)]

use std::path::Path;

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::Verdict;
use law_of_demeter_java_source::{java_platform, map_source};

const JDK_TYPES: &[&str] = &[
    "java.lang.String",
    "java.lang.StringBuilder",
    "java.lang.IllegalStateException",
    "java.time.Duration",
    "java.time.Instant",
    "java.time.temporal.ChronoUnit",
    "java.util.List",
    "java.util.Map",
    "java.util.concurrent.locks.ReentrantReadWriteLock",
];

pub fn verdicts_of(source: &str, class_name: &str, method: &str, types: &Types) -> Vec<Verdict> {
    let mapped = map_source(Path::new("C.java"), source, &|name| {
        JDK_TYPES.contains(&name) || types.shape(name).is_some()
    })
    .unwrap();
    let found: Vec<_> = mapped
        .methods
        .iter()
        .filter(|candidate| candidate.class_name() == class_name && candidate.name() == method)
        .collect();
    assert!(!found.is_empty(), "{class_name}.{method} was not mapped");
    let types = types.clone().with_platform(java_platform());
    found
        .into_iter()
        .flat_map(|candidate| candidate.verdicts(&types))
        .collect()
}

pub fn verdicts_in_c(
    imports: &str,
    members: &str,
    snippet: &str,
    method: &str,
    types: &Types,
) -> Vec<Verdict> {
    verdicts_of(
        &format!("{imports} class C {{ {members}\n{snippet}\n}}"),
        "C",
        method,
        types,
    )
}
