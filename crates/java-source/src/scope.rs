use crate::model::Expression;

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

pub(crate) struct Scope {
    pub(crate) class: String,
    type_variables: Vec<String>,
    frames: Vec<Vec<(String, Binding)>>,
}

impl Scope {
    pub(crate) fn new(class: &str, type_variables: &[String]) -> Self {
        Self {
            class: class.to_owned(),
            type_variables: type_variables.to_vec(),
            frames: vec![Vec::new()],
        }
    }

    pub(crate) fn with_type_variables(mut self, variables: Vec<String>) -> Self {
        self.type_variables.extend(variables);
        self
    }

    pub(crate) fn is_type_variable(&self, name: &str) -> bool {
        self.type_variables.iter().any(|variable| variable == name)
    }

    pub(crate) fn open_frame(&mut self) {
        self.frames.push(Vec::new());
    }

    pub(crate) fn close_frame(&mut self) {
        self.frames.pop();
    }

    pub(crate) fn declare(&mut self, name: &str, binding: Binding) {
        if let Some(frame) = self.frames.last_mut() {
            frame.push((name.to_owned(), binding));
        }
    }

    pub(crate) fn lookup(&self, name: &str) -> Option<Expression> {
        self.frames
            .iter()
            .rev()
            .flat_map(|frame| frame.iter().rev())
            .find(|(declared, _)| declared == name)
            .map(|(_, binding)| binding.expression(name))
    }
}
