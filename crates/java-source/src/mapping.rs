mod conversions;
mod expressions;
mod inference;
mod lambdas;
mod lineage;
mod overloads;
mod patterns;
mod returns;
mod statements;
mod steps;
mod types;

use std::collections::HashMap;
use std::path::Path;

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::Method;
use law_of_demeter_core::domain::type_shape::TypeRef;

use crate::platform::CONSTRUCTOR_NAME;
use crate::{Class, DeclaredType, Method as ParsedMethod, ParseError, SourceFile, parse_with};

use steps::Steps;
use types::{Bounds, object};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappedFile {
    pub classes: Vec<String>,
    pub methods: Vec<Method>,
}

pub fn map_source(path: &Path, source: &str, types: &Types) -> Result<MappedFile, ParseError> {
    let file = parse_with(&path.to_string_lossy(), source, &|name| {
        types.shape(name).is_some()
    })?;
    Ok(map_file(&file, types))
}

fn map_file(file: &SourceFile, types: &Types) -> MappedFile {
    MappedFile {
        classes: file
            .classes
            .iter()
            .map(|class| class.name.clone())
            .collect(),
        methods: file
            .classes
            .iter()
            .flat_map(|class| {
                class
                    .methods
                    .iter()
                    .map(|method| MethodMapper::new(file, types, class, method).map())
            })
            .collect(),
    }
}

struct MethodMapper<'a> {
    file: &'a SourceFile,
    types: &'a Types,
    class: &'a Class,
    method: &'a ParsedMethod,
    bounds: Bounds,
    parameters: HashMap<&'a str, TypeRef>,
    locals: HashMap<String, Option<TypeRef>>,
    lambda_parameters: HashMap<String, TypeRef>,
    returns: Option<TypeRef>,
    steps: Steps,
}

impl<'a> MethodMapper<'a> {
    fn new(
        file: &'a SourceFile,
        types: &'a Types,
        class: &'a Class,
        method: &'a ParsedMethod,
    ) -> Self {
        let bounds = bounds_of(file, class, method);
        Self {
            file,
            types,
            class,
            method,
            parameters: method
                .parameters
                .iter()
                .map(|parameter| {
                    (
                        parameter.name.as_str(),
                        types::type_ref_or_object(&parameter.declared_type, &bounds),
                    )
                })
                .collect(),
            locals: HashMap::new(),
            lambda_parameters: HashMap::new(),
            returns: method
                .returns
                .as_ref()
                .and_then(|returns| types::type_ref(returns, &bounds)),
            bounds,
            steps: Steps::default(),
        }
    }

    fn map(mut self) -> Method {
        let name = if self.method.is_constructor() {
            CONSTRUCTOR_NAME
        } else {
            &self.method.name
        };
        let method = Method::new(&self.class.name, name, self.method.line);
        if let Some(body) = &self.method.body {
            self.statements(body);
        }
        self.steps.into_method(method)
    }

    fn type_ref(&self, declared: &DeclaredType) -> Option<TypeRef> {
        types::type_ref(declared, &self.bounds)
    }

    fn type_ref_or_object(&self, declared: &DeclaredType) -> TypeRef {
        types::type_ref_or_object(declared, &self.bounds)
    }

    fn parameter_type(&self, name: &str) -> TypeRef {
        self.parameters.get(name).cloned().unwrap_or_else(object)
    }

    fn field_type(&self, class_name: &str, field: &str) -> TypeRef {
        self.file
            .classes
            .iter()
            .find(|class| class.name == class_name)
            .and_then(|class| {
                class
                    .fields
                    .iter()
                    .find(|candidate| candidate.name == field)
            })
            .map_or_else(object, |field| {
                self.type_ref_or_object(&field.declared_type)
            })
    }

    fn captured_type(&self, name: &str) -> TypeRef {
        self.class
            .captured
            .iter()
            .rfind(|variable| variable.name == name)
            .map_or_else(object, |variable| {
                self.type_ref_or_object(&variable.declared_type)
            })
    }
}

fn bounds_of(file: &SourceFile, class: &Class, method: &ParsedMethod) -> Bounds {
    let mut enclosing = Vec::new();
    let mut current = Some(class);
    while let Some(found) = current {
        enclosing.push(found);
        current = found.outer.as_ref().and_then(|outer| {
            file.classes
                .iter()
                .find(|candidate| &candidate.name == outer)
        });
    }
    enclosing
        .iter()
        .rev()
        .flat_map(|found| &found.type_parameters)
        .chain(&method.type_parameters)
        .fold(Bounds::new(), |mut bounds, parameter| {
            match &parameter.bound {
                Some(bound) => {
                    bounds.insert(
                        parameter.name.clone(),
                        types::type_ref_or_object(bound, &Bounds::new()),
                    );
                }
                None => {
                    bounds.remove(&parameter.name);
                }
            }
            bounds
        })
}
