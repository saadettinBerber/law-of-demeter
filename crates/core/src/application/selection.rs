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

    fn selects(&self, method: &Method) -> bool {
        method.class_name() == self.class
            && self
                .method
                .as_deref()
                .is_none_or(|name| name == method.name())
    }
}
