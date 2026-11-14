use crate::declarations::Declaration;
use crate::model::{Class, DeclaredType, Expression, Parameter};

#[derive(Clone, Copy)]
pub(crate) enum Binding {
    Parameter,
    Local,
    LambdaParameter,
}

impl Binding {
    fn expression(self, name: &str) -> Expression {
        let name = name.to_owned();
        match self {
            Self::Parameter => Expression::Parameter(name),
            Self::Local => Expression::Local(name),
            Self::LambdaParameter => Expression::LambdaParameter(name),
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct Capture {
    variables: Vec<(String, DeclaredType)>,
    types: Vec<(String, String)>,
}

impl Capture {
    pub(crate) fn variables(&self) -> Vec<Parameter> {
        self.variables
            .iter()
            .map(|(name, declared_type)| Parameter {
                name: name.clone(),
                declared_type: declared_type.clone(),
            })
            .collect()
    }
}

#[derive(Default)]
struct Frame {
    variables: Vec<(String, Binding, DeclaredType)>,
    types: Vec<(String, String)>,
}

pub(crate) struct Scope {
    pub(crate) class: String,
    type_variables: Vec<String>,
    frames: Vec<Frame>,
    captured: Capture,
    nested: Vec<Class>,
}

impl Scope {
    pub(crate) fn for_class(declaration: &Declaration) -> Self {
        Self {
            class: declaration.name.clone(),
            type_variables: declaration.type_variables.clone(),
            frames: vec![Frame::default()],
            captured: declaration.capture.clone(),
            nested: Vec::new(),
        }
    }

    pub(crate) fn with_type_variables(mut self, variables: Vec<String>) -> Self {
        self.type_variables.extend(variables);
        self
    }

    pub(crate) fn type_variables(&self) -> &[String] {
        &self.type_variables
    }

    pub(crate) fn is_type_variable(&self, name: &str) -> bool {
        self.type_variables.iter().any(|variable| variable == name)
    }

    pub(crate) fn open_frame(&mut self) {
        self.frames.push(Frame::default());
    }

    pub(crate) fn close_frame(&mut self) {
        self.frames.pop();
    }

    pub(crate) fn declare(&mut self, name: &str, binding: Binding, declared_type: DeclaredType) {
        if let Some(frame) = self.frames.last_mut() {
            frame
                .variables
                .push((name.to_owned(), binding, declared_type));
        }
    }

    pub(crate) fn declare_type(&mut self, name: &str, binary_name: &str) {
        if let Some(frame) = self.frames.last_mut() {
            frame.types.push((name.to_owned(), binary_name.to_owned()));
        }
    }

    pub(crate) fn lookup(&self, name: &str) -> Option<Expression> {
        self.frames
            .iter()
            .rev()
            .flat_map(|frame| frame.variables.iter().rev())
            .find(|(declared, _, _)| declared == name)
            .map(|(_, binding, _)| binding.expression(name))
    }

    pub(crate) fn lookup_type(&self, name: &str) -> Option<&str> {
        self.frames
            .iter()
            .rev()
            .flat_map(|frame| frame.types.iter().rev())
            .chain(self.captured.types.iter().rev())
            .find(|(declared, _)| declared == name)
            .map(|(_, binary_name)| binary_name.as_str())
    }

    pub(crate) fn is_captured(&self, name: &str) -> bool {
        self.captured
            .variables
            .iter()
            .any(|(captured, _)| captured == name)
    }

    pub(crate) fn capture(&self) -> Capture {
        let mut capture = self.captured.clone();
        for frame in &self.frames {
            capture.variables.extend(
                frame
                    .variables
                    .iter()
                    .map(|(name, _, declared_type)| (name.clone(), declared_type.clone())),
            );
            capture.types.extend(frame.types.iter().cloned());
        }
        capture
    }

    pub(crate) fn add_nested(&mut self, classes: Vec<Class>) {
        self.nested.extend(classes);
    }

    pub(crate) fn take_nested(&mut self) -> Vec<Class> {
        std::mem::take(&mut self.nested)
    }
}
