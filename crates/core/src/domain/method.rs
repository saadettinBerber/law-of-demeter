use std::collections::HashMap;

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

    fn verdicts(&self, scope: &Scope) -> Vec<Verdict> {
        match self {
            Self::ReturnOf(call) => call.verdicts(scope),
            Self::This | Self::Field(_) | Self::Parameter(_) | Self::Local(_) => Vec::new(),
        }
    }

    fn standing(&self, scope: &Scope) -> Standing {
        match self {
            Self::This => Standing::Friend(Reason::This),
            Self::Field(field) => Standing::Friend(Reason::field(field)),
            Self::Parameter(parameter) => Standing::Friend(Reason::parameter(parameter)),
            Self::ReturnOf(call) => Standing::Stranger(Reason::return_of(&call.name)),
            Self::Local(name) => scope
                .get(name)
                .cloned()
                .unwrap_or_else(|| Standing::Stranger(Reason::unassigned_local(name))),
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

    fn verdicts(&self, scope: &Scope) -> Vec<Verdict> {
        let mut verdicts = self.receiver.verdicts(scope);
        verdicts.extend(
            self.arguments
                .iter()
                .flat_map(|argument| argument.verdicts(scope)),
        );
        verdicts.push(self.verdict(scope));
        verdicts
    }

    fn verdict(&self, scope: &Scope) -> Verdict {
        self.receiver.standing(scope).verdict(&self.name)
    }
}

type Scope = HashMap<String, Standing>;

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

    fn through(self, wrap: impl FnOnce(Reason) -> Reason) -> Self {
        match self {
            Self::Friend(reason) => Self::Friend(wrap(reason)),
            Self::Stranger(reason) => Self::Stranger(wrap(reason)),
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
    Local { name: String, origin: Box<Reason> },
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

    pub fn local(name: &str, origin: Reason) -> Self {
        Self::Local {
            name: name.to_owned(),
            origin: Box::new(origin),
        }
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum Step {
    Call(Call),
    Assign { local: String, value: Expression },
}

impl Step {
    fn verdicts(&self, scope: &mut Scope) -> Vec<Verdict> {
        match self {
            Self::Call(call) => call.verdicts(scope),
            Self::Assign { local, value } => {
                let verdicts = value.verdicts(scope);
                let standing = value
                    .standing(scope)
                    .through(|origin| Reason::local(local, origin));
                scope.insert(local.clone(), standing);
                verdicts
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Method {
    steps: Vec<Step>,
}

impl Method {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn call(mut self, call: Call) -> Self {
        self.steps.push(Step::Call(call));
        self
    }

    pub fn assign(mut self, local: &str, value: Expression) -> Self {
        self.steps.push(Step::Assign {
            local: local.to_owned(),
            value,
        });
        self
    }

    pub fn verdicts(&self) -> Vec<Verdict> {
        let mut scope = Scope::new();
        self.steps
            .iter()
            .flat_map(|step| step.verdicts(&mut scope))
            .collect()
    }
}

#[cfg(test)]
mod tests;
