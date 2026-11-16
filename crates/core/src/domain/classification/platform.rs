use std::collections::{HashMap, HashSet};

use super::Classification;

#[derive(Debug, Clone, Default)]
pub struct Platform {
    types: HashMap<String, Classification>,
    namespaces: Vec<(String, Classification)>,
    behaviorless_supertypes: HashSet<String>,
    root_supertype: Option<String>,
    object_methods: HashSet<(String, usize)>,
    equality_methods: Vec<(String, usize)>,
    constructor_name: Option<String>,
}

impl Platform {
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

    pub fn with_root_supertype(mut self, name: &str) -> Self {
        self.root_supertype = Some(name.to_owned());
        self
    }

    pub fn with_object_method(mut self, name: &str, parameter_count: usize) -> Self {
        self.object_methods
            .insert((name.to_owned(), parameter_count));
        self
    }

    pub fn with_equality_method(mut self, name: &str, parameter_count: usize) -> Self {
        self.equality_methods
            .push((name.to_owned(), parameter_count));
        self
    }

    pub fn with_constructor_name(mut self, name: &str) -> Self {
        self.constructor_name = Some(name.to_owned());
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

    pub(super) fn root_supertype(&self) -> Option<&str> {
        self.root_supertype.as_deref()
    }

    pub(super) fn is_object_method(&self, name: &str, parameter_count: usize) -> bool {
        self.object_methods
            .contains(&(name.to_owned(), parameter_count))
    }

    pub(super) fn equality_methods(&self) -> &[(String, usize)] {
        &self.equality_methods
    }

    pub(super) fn is_constructor(&self, name: &str) -> bool {
        self.constructor_name.as_deref() == Some(name)
    }

    pub(super) fn is_behaviorless(&self, name: &str) -> bool {
        self.behaviorless_supertypes.contains(name)
    }
}
