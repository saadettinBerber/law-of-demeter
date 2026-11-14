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
    pub kind: ClassKind,
    pub outer: Option<String>,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub declared_type: DeclaredType,
    pub is_static: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeName {
    Resolved(String),
    Unresolved(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclaredType {
    Var,
    Primitive(String),
    Class {
        name: TypeName,
        arguments: Vec<DeclaredType>,
    },
    Array(Box<DeclaredType>),
    Variable(String),
    Wildcard(Option<Box<DeclaredType>>),
}

impl DeclaredType {
    pub fn primitive(name: &str) -> Self {
        Self::Primitive(name.to_owned())
    }

    pub fn class(binary_name: &str) -> Self {
        Self::generic(binary_name, Vec::new())
    }

    pub fn generic(binary_name: &str, arguments: Vec<DeclaredType>) -> Self {
        Self::Class {
            name: TypeName::Resolved(binary_name.to_owned()),
            arguments,
        }
    }

    pub fn unresolved(name: &str) -> Self {
        Self::Class {
            name: TypeName::Unresolved(name.to_owned()),
            arguments: Vec::new(),
        }
    }

    pub fn array(element: DeclaredType) -> Self {
        Self::Array(Box::new(element))
    }

    pub fn variable(name: &str) -> Self {
        Self::Variable(name.to_owned())
    }

    pub fn wildcard(upper_bound: Option<DeclaredType>) -> Self {
        Self::Wildcard(upper_bound.map(Box::new))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassKind {
    Class,
    Interface,
    Enum,
    Record,
    Annotation,
}
