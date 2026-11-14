use law_of_demeter_core::domain::method::{Call, Expression, Method};

#[derive(Default)]
pub(super) struct Steps {
    operations: Vec<Operation>,
}

enum Operation {
    Call(Call),
    Evaluate(Expression),
}

impl Steps {
    pub(super) fn evaluate(&mut self, value: Expression) {
        match value {
            Expression::ReturnOf(call) => self.operations.push(Operation::Call(*call)),
            Expression::Literal
            | Expression::This
            | Expression::Local(_)
            | Expression::Parameter { .. }
            | Expression::Field { .. }
            | Expression::StaticType(_)
            | Expression::Unresolved(_) => {}
            other => self.operations.push(Operation::Evaluate(other)),
        }
    }

    pub(super) fn into_method(self, method: Method) -> Method {
        self.operations
            .into_iter()
            .fold(method, |method, operation| match operation {
                Operation::Call(call) => method.call(call),
                Operation::Evaluate(value) => method.evaluate(value),
            })
    }
}
