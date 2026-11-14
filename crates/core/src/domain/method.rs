#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    This,
    Field(String),
    Parameter(String),
    ReturnOf(Box<Call>),
}

impl Expression {
    pub fn return_of(call: Call) -> Self {
        Self::ReturnOf(Box::new(call))
    }

    fn verdicts(&self) -> Vec<Verdict> {
        match self {
            Self::ReturnOf(call) => call.verdicts(),
            Self::This | Self::Field(_) | Self::Parameter(_) => Vec::new(),
        }
    }

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
    arguments: Vec<Expression>,
}

impl Call {
    pub fn new(receiver: Expression, name: &str) -> Self {
        Self {
            receiver,
            name: name.to_owned(),
            arguments: Vec::new(),
        }
    }

    pub fn with(mut self, argument: Expression) -> Self {
        self.arguments.push(argument);
        self
    }

    pub fn then(self, name: &str) -> Self {
        Self::new(Expression::return_of(self), name)
    }

    fn verdicts(&self) -> Vec<Verdict> {
        let mut verdicts = self.receiver.verdicts();
        verdicts.extend(self.arguments.iter().flat_map(Expression::verdicts));
        verdicts.push(self.verdict());
        verdicts
    }

    fn verdict(&self) -> Verdict {
        match &self.receiver {
            Expression::This => Verdict::allowed(&self.name, Reason::This),
            Expression::Field(field) => Verdict::allowed(&self.name, Reason::field(field)),
            Expression::Parameter(parameter) => {
                Verdict::allowed(&self.name, Reason::parameter(parameter))
            }
            Expression::ReturnOf(previous) => {
                Verdict::violation(&self.name, Reason::return_of(&previous.name))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    This,
    Field(String),
    Parameter(String),
    ReturnOf(String),
}

impl Reason {
    pub fn field(name: &str) -> Self {
        Self::Field(name.to_owned())
    }

    pub fn parameter(name: &str) -> Self {
        Self::Parameter(name.to_owned())
    }

    pub fn return_of(call: &str) -> Self {
        Self::ReturnOf(call.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Allowed { call: String, reason: Reason },
    Violation { call: String, reason: Reason },
}

impl Verdict {
    pub fn allowed(call: &str, reason: Reason) -> Self {
        Self::Allowed {
            call: call.to_owned(),
            reason,
        }
    }

    pub fn violation(call: &str, reason: Reason) -> Self {
        Self::Violation {
            call: call.to_owned(),
            reason,
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

    pub fn verdicts(&self) -> Vec<Verdict> {
        self.calls.iter().flat_map(Call::verdicts).collect()
    }
}

#[cfg(test)]
mod tests;
