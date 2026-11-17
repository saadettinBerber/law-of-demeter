use super::ports::SourceFile;
use crate::domain::classification::Types;
use crate::domain::method::Method;

#[derive(Debug)]
pub struct Selection(Choice);

#[derive(Debug)]
enum Choice {
    Everything,
    Only(Vec<Target>),
}

impl Selection {
    pub fn everything() -> Self {
        Self(Choice::Everything)
    }

    pub fn of(targets: Vec<Target>) -> Self {
        if targets.is_empty() {
            Self::everything()
        } else {
            Self(Choice::Only(targets))
        }
    }

    pub(super) fn selects(&self, method: &Method, types: &Types) -> bool {
        match &self.0 {
            Choice::Everything => true,
            Choice::Only(targets) => targets.iter().any(|target| target.selects(method, types)),
        }
    }

    pub(super) fn selects_class(&self, class: &str, types: &Types) -> bool {
        match &self.0 {
            Choice::Everything => true,
            Choice::Only(targets) => targets
                .iter()
                .any(|target| target.selects_class(class, types)),
        }
    }

    pub(super) fn unmatched<'f>(
        &self,
        files: impl Iterator<Item = &'f SourceFile> + Clone,
        types: &Types,
    ) -> Vec<Target> {
        match &self.0 {
            Choice::Everything => Vec::new(),
            Choice::Only(targets) => targets
                .iter()
                .filter(|target| !files.clone().any(|file| target.is_found_in(file, types)))
                .cloned()
                .collect(),
        }
    }

    pub(super) fn covers(&self, classes: &[String], types: &Types) -> bool {
        match &self.0 {
            Choice::Everything => true,
            Choice::Only(targets) => classes
                .iter()
                .any(|class| targets.iter().any(|target| target.contains(class, types))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    class: String,
    method: Option<String>,
}

impl Target {
    pub fn class(name: &str) -> Self {
        Self {
            class: name.to_owned(),
            method: None,
        }
    }

    pub fn method(class: &str, method: &str) -> Self {
        Self {
            class: class.to_owned(),
            method: Some(method.to_owned()),
        }
    }

    pub fn class_name(&self) -> &str {
        &self.class
    }

    pub fn method_name(&self) -> Option<&str> {
        self.method.as_deref()
    }

    fn contains(&self, class: &str, types: &Types) -> bool {
        types.is_part_of(class, &self.class)
    }

    fn selects(&self, method: &Method, types: &Types) -> bool {
        match &self.method {
            None => self.contains(method.class_name(), types),
            Some(name) => {
                (method.class_name() == self.class && method.name() == name)
                    || method.is_declared_within(&self.class, name)
            }
        }
    }

    fn selects_class(&self, class: &str, types: &Types) -> bool {
        self.method.is_none() && self.contains(class, types)
    }

    fn is_found_in(&self, file: &SourceFile, types: &Types) -> bool {
        file.methods()
            .iter()
            .any(|method| self.selects(method, types))
            || file
                .classes()
                .iter()
                .any(|class| self.selects_class(class, types))
    }
}
