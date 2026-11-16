use std::collections::{HashMap, HashSet};

use super::Classification;

#[derive(Debug, Clone, Default)]
pub struct KnownTypes {
    types: HashMap<String, Classification>,
    namespaces: Vec<(String, Classification)>,
    behaviorless_supertypes: HashSet<String>,
}

impl KnownTypes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_type(mut self, name: &str, classification: Classification) -> Self {
        self.types.insert(name.to_owned(), classification);
        self
    }

    pub fn with_namespace(mut self, prefix: &str, classification: Classification) -> Self {
        self.namespaces.push((prefix.to_owned(), classification));
        self
    }

    pub fn with_behaviorless_supertype(mut self, name: &str) -> Self {
        self.behaviorless_supertypes.insert(name.to_owned());
        self
    }

    pub(super) fn classify(&self, name: &str) -> Option<Classification> {
        self.types.get(name).copied().or_else(|| {
            self.namespaces
                .iter()
                .find(|(prefix, _)| name.starts_with(prefix.as_str()))
                .map(|(_, classification)| *classification)
        })
    }

    pub(super) fn is_behaviorless(&self, name: &str) -> bool {
        self.behaviorless_supertypes.contains(name)
    }
}
