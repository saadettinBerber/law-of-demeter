#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Receiver {
    This,
    Field(String),
    Parameter(String),
    ReturnOf(Box<Call>),
}

impl Receiver {
    pub fn field(name: &str) -> Self {
        Self::Field(name.to_owned())
    }

    pub fn parameter(name: &str) -> Self {
        Self::Parameter(name.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    receiver: Receiver,
    name: String,
}

impl Call {
    pub fn new(receiver: Receiver, name: &str) -> Self {
        Self {
            receiver,
            name: name.to_owned(),
        }
    }

    pub fn then(self, name: &str) -> Self {
        Self::new(Receiver::ReturnOf(Box::new(self)), name)
    }

    fn violations(&self) -> Vec<Violation> {
        match &self.receiver {
            Receiver::ReturnOf(previous) => {
                let mut violations = previous.violations();
                violations.push(Violation::calling(&self.name));
                violations
            }
            Receiver::This | Receiver::Field(_) | Receiver::Parameter(_) => Vec::new(),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Method {
    calls: Vec<Call>,
}

impl Method {
    pub fn new(calls: Vec<Call>) -> Self {
        Self { calls }
    }

    pub fn violations(&self) -> Vec<Violation> {
        self.calls.iter().flat_map(Call::violations).collect()
    }
}

#[cfg(test)]
mod tests;
