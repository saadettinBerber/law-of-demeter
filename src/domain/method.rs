#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    This,
    Field(String),
    Parameter(String),
    ReturnOf(Box<Call>),
}

impl Expression {
    pub fn field(name: &str) -> Self {
        Self::Field(name.to_owned())
    }

    pub fn parameter(name: &str) -> Self {
        Self::Parameter(name.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    receiver: Expression,
    name: String,
}

impl Call {
    pub fn new(receiver: Expression, name: &str) -> Self {
        Self {
            receiver,
            name: name.to_owned(),
        }
    }

    pub fn then(self, name: &str) -> Self {
        Self::new(Expression::ReturnOf(Box::new(self)), name)
    }

    fn violations(&self) -> Vec<Violation> {
        match &self.receiver {
            Expression::ReturnOf(previous) => {
                let mut violations = previous.violations();
                violations.push(Violation::calling(&self.name));
                violations
            }
            Expression::This | Expression::Field(_) | Expression::Parameter(_) => Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    call: String,
}

impl Violation {
    pub fn calling(name: &str) -> Self {
        Self {
            call: name.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Method {
    calls: Vec<Call>,
}

impl Method {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn call(mut self, call: Call) -> Self {
        self.calls.push(call);
        self
    }

    pub fn violations(&self) -> Vec<Violation> {
        self.calls.iter().flat_map(Call::violations).collect()
    }
}

#[cfg(test)]
mod tests;
