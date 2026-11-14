use std::collections::HashSet;

use crate::model::{Import, TypeName};

pub(crate) struct TypeResolver<'a> {
    package: Option<String>,
    single_imports: Vec<String>,
    on_demand_prefixes: Vec<String>,
    declared: HashSet<String>,
    type_exists: &'a dyn Fn(&str) -> bool,
}

impl<'a> TypeResolver<'a> {
    pub(crate) fn new(
        package: Option<&str>,
        imports: &[Import],
        declared: HashSet<String>,
        type_exists: &'a dyn Fn(&str) -> bool,
    ) -> Self {
        let plain_imports = imports.iter().filter(|import| !import.is_static);
        Self {
            package: package.map(str::to_owned),
            single_imports: plain_imports
                .clone()
                .filter(|import| !import.is_wildcard)
                .map(|import| import.name.clone())
                .collect(),
            on_demand_prefixes: plain_imports
                .filter(|import| import.is_wildcard)
                .map(|import| on_demand_prefix(&import.name))
                .collect(),
            declared,
            type_exists,
        }
    }

    pub(crate) fn resolve(&self, name: &str, enclosing: &str) -> TypeName {
        let mut segments = name.split('.');
        let head = segments.next().unwrap_or_default();
        let nested: Vec<&str> = segments.collect();
        match self.resolve_simple(head, enclosing) {
            Some(binary_name) => TypeName::Resolved(with_nested(binary_name, &nested)),
            None => match qualified_binary_name(name) {
                Some(binary_name) if name.contains('.') && !starts_with_uppercase(head) => {
                    TypeName::Resolved(binary_name)
                }
                _ => TypeName::Unresolved(name.to_owned()),
            },
        }
    }

    pub(crate) fn resolve_simple(&self, name: &str, enclosing: &str) -> Option<String> {
        self.declared_in_file(name, enclosing)
            .or_else(|| self.single_import(name))
            .or_else(|| self.known(&self.in_package(name)))
            .or_else(|| self.on_demand(name))
    }

    fn declared_in_file(&self, name: &str, enclosing: &str) -> Option<String> {
        let mut scope = enclosing;
        while !scope.is_empty() {
            let candidate = format!("{scope}${name}");
            if self.declared.contains(&candidate) {
                return Some(candidate);
            }
            scope = scope.rsplit_once('$').map_or("", |(outer, _)| outer);
        }
        Some(self.in_package(name)).filter(|candidate| self.declared.contains(candidate))
    }

    fn single_import(&self, name: &str) -> Option<String> {
        self.single_imports
            .iter()
            .find(|import| import.rsplit('.').next() == Some(name))
            .and_then(|import| qualified_binary_name(import))
    }

    fn on_demand(&self, name: &str) -> Option<String> {
        let mut found: Vec<String> = self
            .on_demand_prefixes
            .iter()
            .map(|prefix| format!("{prefix}{name}"))
            .chain([format!("java.lang.{name}")])
            .filter(|candidate| (self.type_exists)(candidate))
            .collect();
        found.sort();
        found.dedup();
        match found.as_slice() {
            [only] => Some(only.clone()),
            _ => None,
        }
    }

    fn known(&self, candidate: &str) -> Option<String> {
        (self.type_exists)(candidate).then(|| candidate.to_owned())
    }

    fn in_package(&self, name: &str) -> String {
        match &self.package {
            Some(package) => format!("{package}.{name}"),
            None => name.to_owned(),
        }
    }
}

fn on_demand_prefix(import: &str) -> String {
    match qualified_binary_name(import) {
        Some(binary_name) if import.rsplit('.').next().is_some_and(starts_with_uppercase) => {
            format!("{binary_name}$")
        }
        _ => format!("{import}."),
    }
}

fn with_nested(binary_name: String, nested: &[&str]) -> String {
    nested
        .iter()
        .fold(binary_name, |outer, inner| format!("{outer}${inner}"))
}

fn qualified_binary_name(name: &str) -> Option<String> {
    let segments: Vec<&str> = name.split('.').collect();
    let class_index = segments
        .iter()
        .position(|segment| starts_with_uppercase(segment))
        .unwrap_or(segments.len().saturating_sub(1));
    let (package, classes) = segments.split_at(class_index);
    let class = with_nested(classes.first()?.to_string(), &classes[1..]);
    Some(if package.is_empty() {
        class
    } else {
        format!("{}.{class}", package.join("."))
    })
}

fn starts_with_uppercase(segment: &str) -> bool {
    segment.chars().next().is_some_and(char::is_uppercase)
}
