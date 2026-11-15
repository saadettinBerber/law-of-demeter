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
    pub origin: ClassOrigin,
    pub outer: Option<String>,
    pub captured: Vec<Parameter>,
    pub annotations: Vec<TypeName>,
    pub supertypes: Vec<DeclaredType>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Method {
    pub name: String,
    pub line: usize,
    pub is_static: bool,
    pub parameters: Vec<Parameter>,
    pub returns: Option<DeclaredType>,
    pub body: Option<Vec<Statement>>,
}

pub const STATIC_INITIALIZER: &str = "<static-init>";
pub const INSTANCE_INITIALIZER: &str = "<instance-init>";

impl Method {
    pub fn is_constructor(&self) -> bool {
        self.returns.is_none()
    }

    pub fn is_initializer(&self) -> bool {
        self.name == STATIC_INITIALIZER || self.name == INSTANCE_INITIALIZER
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub declared_type: DeclaredType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalVariable {
    pub name: String,
    pub declared_type: DeclaredType,
    pub initializer: Option<Expression>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Local(LocalVariable),
    Expression(Expression),
    Return(Option<Expression>),
    Block(Vec<Statement>),
    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Vec<Statement>,
    },
    For {
        init: Vec<Statement>,
        condition: Option<Expression>,
        update: Vec<Expression>,
        body: Vec<Statement>,
    },
    ForEach {
        variable: LocalVariable,
        iterable: Expression,
        body: Vec<Statement>,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    DoWhile {
        body: Vec<Statement>,
        condition: Expression,
    },
    Try {
        resources: Vec<Statement>,
        body: Vec<Statement>,
        catches: Vec<Catch>,
        finally: Vec<Statement>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catch {
    pub name: String,
    pub types: Vec<DeclaredType>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    This,
    Super,
    Parameter(String),
    Local(String),
    LambdaParameter(String),
    Captured(String),
    Field(String),
    OuterField {
        owner: String,
        name: String,
    },
    Type(TypeName),
    StaticImport {
        owner: TypeName,
        name: String,
    },
    Unresolved(String),
    FieldAccess {
        target: Box<Expression>,
        name: String,
    },
    Assignment {
        target: Box<Expression>,
        value: Box<Expression>,
    },
    Call(Call),
    New(New),
    Lambda(Lambda),
    MethodReference {
        target: Box<Expression>,
        name: String,
    },
    Cast {
        target: DeclaredType,
        operand: Box<Expression>,
    },
    InstanceOf {
        operand: Box<Expression>,
        tested: DeclaredType,
        bindings: Vec<PatternBinding>,
    },
    Switch(Switch),
    ClassLiteral(DeclaredType),
    Literal,
    Operation(Vec<Expression>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    pub receiver: Option<Box<Expression>>,
    pub implicit_target: Option<ImplicitTarget>,
    pub name: String,
    pub arguments: Vec<Expression>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImplicitTarget {
    OwnMethod,
    OuterMethod { owner: String },
    StaticImport { owner: TypeName },
    OnDemandStaticImport { candidates: Vec<TypeName> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct New {
    pub created: DeclaredType,
    pub arguments: Vec<Expression>,
    pub anonymous_class: Option<String>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lambda {
    pub parameters: Vec<LambdaParameter>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LambdaParameter {
    pub name: String,
    pub declared_type: Option<DeclaredType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Switch {
    pub selector: Box<Expression>,
    pub branches: Vec<SwitchBranch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchBranch {
    pub bindings: Vec<PatternBinding>,
    pub guard: Option<Expression>,
    pub body: Vec<Statement>,
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
pub enum ClassOrigin {
    Member,
    Anonymous,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassKind {
    Class,
    Interface,
    Enum,
    Record,
    Annotation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternBinding {
    pub name: String,
    pub declared_type: DeclaredType,
    pub is_component: bool,
}
