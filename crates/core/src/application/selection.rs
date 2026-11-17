use crate::domain::method::Method;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Selection {
    #[default]
    Everything,
    Only(Vec<Target>),
}

impl Selection {
    pub(super) fn selects(&self, method: &Method) -> bool {
        match self {
            Self::Everything => true,
            Self::Only(targets) => targets.iter().any(|target| target.selects(method)),
        }
    }

    pub(super) fn covers(&self, classes: &[String]) -> bool {
        match self {
            Self::Everything => true,
            Self::Only(targets) => classes
                .iter()
                .any(|class| targets.iter().any(|target| target.names(class))),
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

    fn names(&self, class: &str) -> bool {
        class == self.class
    }

    fn selects(&self, method: &Method) -> bool {
        self.names(method.class_name())
            && self
                .method
                .as_deref()
                .is_none_or(|name| name == method.name())
    }
}
