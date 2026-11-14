#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub path: String,
    pub package: Option<String>,
    pub imports: Vec<Import>,
    pub classes: Vec<Class>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    pub name: String,
    pub is_static: bool,
    pub is_wildcard: bool,
}

impl Import {
    pub fn single(name: &str) -> Self {
        Self::new(name, false, false)
    }

    pub fn wildcard(name: &str) -> Self {
        Self::new(name, false, true)
    }

    pub fn static_single(name: &str) -> Self {
        Self::new(name, true, false)
    }

    pub fn static_wildcard(name: &str) -> Self {
        Self::new(name, true, true)
    }

    fn new(name: &str, is_static: bool, is_wildcard: bool) -> Self {
        Self {
            name: name.to_owned(),
            is_static,
            is_wildcard,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Class {
    pub name: String,
}
