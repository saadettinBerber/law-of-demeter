mod expressions;
mod patterns;
mod statements;
mod steps;
mod types;

use std::collections::HashMap;
use std::path::Path;

use law_of_demeter_core::domain::method::Method;
use law_of_demeter_core::domain::type_shape::TypeRef;

use crate::{Class, Method as ParsedMethod, ParseError, SourceFile, parse_with};

use steps::Steps;
use types::{object, type_ref_or_object};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappedFile {
    pub classes: Vec<String>,
    pub methods: Vec<Method>,
}

pub fn map_source(
    path: &Path,
    source: &str,
    type_exists: &dyn Fn(&str) -> bool,
) -> Result<MappedFile, ParseError> {
    let file = parse_with(&path.to_string_lossy(), source, type_exists)?;
    Ok(map_file(&file))
}

fn map_file(file: &SourceFile) -> MappedFile {
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
                    .map(|method| MethodMapper::new(file, class, method).map())
            })
            .collect(),
    }
}

struct MethodMapper<'a> {
    file: &'a SourceFile,
    class: &'a Class,
    method: &'a ParsedMethod,
    parameters: HashMap<&'a str, TypeRef>,
    locals: HashMap<String, Option<TypeRef>>,
    steps: Steps,
}

impl<'a> MethodMapper<'a> {
    fn new(file: &'a SourceFile, class: &'a Class, method: &'a ParsedMethod) -> Self {
        Self {
            file,
            class,
            method,
            parameters: method
                .parameters
                .iter()
                .map(|parameter| {
                    (
                        parameter.name.as_str(),
                        type_ref_or_object(&parameter.declared_type),
                    )
                })
                .collect(),
            locals: HashMap::new(),
            steps: Steps::default(),
        }
    }

    fn map(mut self) -> Method {
        let name = if self.method.is_constructor() {
            "<init>"
        } else {
            &self.method.name
        };
        let method = Method::new(&self.class.name, name, self.method.line);
        if let Some(body) = &self.method.body {
            self.statements(body);
        }
        self.steps.into_method(method)
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
            .map_or_else(object, |field| type_ref_or_object(&field.declared_type))
    }

    fn captured_type(&self, name: &str) -> TypeRef {
        self.class
            .captured
            .iter()
            .rfind(|variable| variable.name == name)
            .map_or_else(object, |variable| {
                type_ref_or_object(&variable.declared_type)
            })
    }
}
