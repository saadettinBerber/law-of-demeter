#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    This,
    Field(String),
    Parameter(String),
    ReturnOf(Box<Call>),
    Local(String),
}

impl Expression {
    pub fn local(name: &str) -> Self {
        Self::Local(name.to_owned())
    }

    pub fn return_of(call: Call) -> Self {
        Self::ReturnOf(Box::new(call))
    }

    fn verdicts(&self) -> Vec<Verdict> {
        match self {
            Self::ReturnOf(call) => call.verdicts(),
            Self::This | Self::Field(_) | Self::Parameter(_) | Self::Local(_) => Vec::new(),
        }
    }

    fn standing(&self) -> Standing {
        match self {
            Self::This => Standing::Friend(Reason::This),
            Self::Field(field) => Standing::Friend(Reason::field(field)),
            Self::Parameter(parameter) => Standing::Friend(Reason::parameter(parameter)),
            Self::ReturnOf(call) => Standing::Stranger(Reason::return_of(&call.name)),
            Self::Local(name) => Standing::Stranger(Reason::unassigned_local(name)),
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
        self.receiver.standing().verdict(&self.name)
    }
}

#[derive(Debug, Clone)]
enum Standing {
    Friend(Reason),
    Stranger(Reason),
}

impl Standing {
    fn verdict(self, call: &str) -> Verdict {
        match self {
            Self::Friend(reason) => Verdict::allowed(call, reason),
            Self::Stranger(reason) => Verdict::violation(call, reason),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    This,
    Field(String),
    Parameter(String),
    ReturnOf(String),
    UnassignedLocal(String),
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

    pub fn unassigned_local(name: &str) -> Self {
        Self::UnassignedLocal(name.to_owned())
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
