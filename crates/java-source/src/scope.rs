pub(crate) struct Scope {
    pub(crate) class: String,
    type_variables: Vec<String>,
}

impl Scope {
    pub(crate) fn new(class: &str, type_variables: &[String]) -> Self {
        Self {
            class: class.to_owned(),
            type_variables: type_variables.to_vec(),
        }
    }

    pub(crate) fn with_type_variables(mut self, variables: Vec<String>) -> Self {
        self.type_variables.extend(variables);
        self
    }

    pub(crate) fn is_type_variable(&self, name: &str) -> bool {
        self.type_variables.iter().any(|variable| variable == name)
    }
}
