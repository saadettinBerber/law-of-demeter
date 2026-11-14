#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeRef {
    Primitive(String),
    Class {
        name: String,
        arguments: Vec<TypeRef>,
    },
    Array(Box<TypeRef>),
    Variable(String),
}

impl TypeRef {
    pub fn primitive(name: &str) -> Self {
        Self::Primitive(name.to_owned())
    }

    pub fn class(name: &str) -> Self {
        Self::generic(name, Vec::new())
    }

    pub fn generic(name: &str, arguments: Vec<TypeRef>) -> Self {
        Self::Class {
            name: name.to_owned(),
            arguments,
        }
    }

    pub fn array(element: TypeRef) -> Self {
        Self::Array(Box::new(element))
    }

    pub fn variable(name: &str) -> Self {
        Self::Variable(name.to_owned())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKind {
    Class,
    Record,
    Enum,
    Interface,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldShape {
    name: String,
    type_ref: TypeRef,
    is_static: bool,
}

impl FieldShape {
    pub fn new(name: &str, type_ref: TypeRef) -> Self {
        Self {
            name: name.to_owned(),
            type_ref,
            is_static: false,
        }
    }

    pub fn as_static(mut self) -> Self {
        self.is_static = true;
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_ref(&self) -> &TypeRef {
        &self.type_ref
    }

    pub fn is_static(&self) -> bool {
        self.is_static
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodShape {
    name: String,
    parameters: Vec<TypeRef>,
    returns: Option<TypeRef>,
    is_static: bool,
    is_public: bool,
    is_generated: bool,
}

impl MethodShape {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            parameters: Vec::new(),
            returns: None,
            is_static: false,
            is_public: true,
            is_generated: false,
        }
    }

    pub fn with_parameters(mut self, parameters: Vec<TypeRef>) -> Self {
        self.parameters = parameters;
        self
    }

    pub fn returning(mut self, type_ref: TypeRef) -> Self {
        self.returns = Some(type_ref);
        self
    }

    pub fn as_static(mut self) -> Self {
        self.is_static = true;
        self
    }

    pub fn not_public(mut self) -> Self {
        self.is_public = false;
        self
    }

    pub fn generated(mut self) -> Self {
        self.is_generated = true;
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn parameters(&self) -> &[TypeRef] {
        &self.parameters
    }

    pub fn returns(&self) -> Option<&TypeRef> {
        self.returns.as_ref()
    }

    pub fn is_static(&self) -> bool {
        self.is_static
    }

    pub fn is_public(&self) -> bool {
        self.is_public
    }

    pub fn is_generated(&self) -> bool {
        self.is_generated
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeShape {
    name: String,
    kind: TypeKind,
    supertypes: Vec<TypeRef>,
    fields: Vec<FieldShape>,
    methods: Vec<MethodShape>,
}

impl TypeShape {
    pub fn new(name: &str, kind: TypeKind) -> Self {
        Self {
            name: name.to_owned(),
            kind,
            supertypes: Vec::new(),
            fields: Vec::new(),
            methods: Vec::new(),
        }
    }

    pub fn extending(mut self, supertype: TypeRef) -> Self {
        self.supertypes.push(supertype);
        self
    }

    pub fn with_field(mut self, field: FieldShape) -> Self {
        self.fields.push(field);
        self
    }

    pub fn with_method(mut self, method: MethodShape) -> Self {
        self.methods.push(method);
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> TypeKind {
        self.kind
    }

    pub fn supertypes(&self) -> &[TypeRef] {
        &self.supertypes
    }

    pub fn fields(&self) -> &[FieldShape] {
        &self.fields
    }

    pub fn methods(&self) -> &[MethodShape] {
        &self.methods
    }
}
