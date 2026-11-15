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
    is_final: bool,
}

impl FieldShape {
    pub fn new(name: &str, type_ref: TypeRef) -> Self {
        Self {
            name: name.to_owned(),
            type_ref,
            is_static: false,
            is_final: false,
        }
    }

    pub fn as_static(mut self) -> Self {
        self.is_static = true;
        self
    }

    pub fn as_final(mut self) -> Self {
        self.is_final = true;
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

    pub fn is_final(&self) -> bool {
        self.is_final
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
    is_abstract: bool,
    type_parameters: Vec<String>,
    reads_field: Option<String>,
    writes_field: Option<String>,
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
            is_abstract: false,
            type_parameters: Vec::new(),
            reads_field: None,
            writes_field: None,
        }
    }

    pub fn with_type_parameter(mut self, name: &str) -> Self {
        self.type_parameters.push(name.to_owned());
        self
    }

    pub fn as_abstract(mut self) -> Self {
        self.is_abstract = true;
        self
    }

    pub fn reading(mut self, field: &str) -> Self {
        self.reads_field = Some(field.to_owned());
        self
    }

    pub fn writing(mut self, field: &str) -> Self {
        self.writes_field = Some(field.to_owned());
        self
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

    pub fn is_abstract(&self) -> bool {
        self.is_abstract
    }

    pub fn type_parameters(&self) -> &[String] {
        &self.type_parameters
    }

    pub fn reads_field(&self) -> Option<&str> {
        self.reads_field.as_deref()
    }

    pub fn writes_field(&self) -> Option<&str> {
        self.writes_field.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeShape {
    name: String,
    kind: TypeKind,
    type_parameters: Vec<String>,
    superclass: Option<TypeRef>,
    supertypes: Vec<TypeRef>,
    fields: Vec<FieldShape>,
    methods: Vec<MethodShape>,
}

impl TypeShape {
    pub fn new(name: &str, kind: TypeKind) -> Self {
        Self {
            name: name.to_owned(),
            kind,
            type_parameters: Vec::new(),
            superclass: None,
            supertypes: Vec::new(),
            fields: Vec::new(),
            methods: Vec::new(),
        }
    }

    pub fn with_type_parameter(mut self, name: &str) -> Self {
        self.type_parameters.push(name.to_owned());
        self
    }

    pub fn with_superclass(mut self, superclass: TypeRef) -> Self {
        self.supertypes.push(superclass.clone());
        self.superclass = Some(superclass);
        self
    }

    pub fn extending(mut self, supertype: TypeRef) -> Self {
        self.supertypes.push(supertype);
        self
    }

    pub fn type_parameters(&self) -> &[String] {
        &self.type_parameters
    }

    pub fn superclass(&self) -> Option<&TypeRef> {
        self.superclass.as_ref()
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
