use law_of_demeter_core::domain::method::{Call, Expression, Lambda, Method};
use law_of_demeter_core::domain::type_shape::TypeRef;

#[derive(Default)]
pub(super) struct Steps {
    operations: Vec<Operation>,
}

enum Operation {
    Call(Call),
    Assign {
        local: String,
        declared: Option<TypeRef>,
        value: Expression,
    },
    ForEach {
        variable: String,
        iterable: Expression,
    },
    Evaluate(Expression),
    GiveBack(Expression),
}

impl Steps {
    pub(super) fn assign(&mut self, local: &str, declared: Option<TypeRef>, value: Expression) {
        self.operations.push(Operation::Assign {
            local: local.to_owned(),
            declared,
            value,
        });
    }

    pub(super) fn for_each(&mut self, variable: &str, iterable: Expression) {
        self.operations.push(Operation::ForEach {
            variable: variable.to_owned(),
            iterable,
        });
    }

    pub(super) fn evaluate(&mut self, value: Expression) {
        match value {
            Expression::ReturnOf(call) => self.operations.push(Operation::Call(*call)),
            other if is_quiet(&other) => {}
            other => self.operations.push(Operation::Evaluate(other)),
        }
    }

    pub(super) fn give_back(&mut self, value: Expression) {
        self.operations.push(Operation::GiveBack(value));
    }

    pub(super) fn into_method(self, method: Method) -> Method {
        self.operations
            .into_iter()
            .fold(method, |method, operation| match operation {
                Operation::Call(call) => method.call(call),
                Operation::Assign {
                    local,
                    declared: Some(type_ref),
                    value,
                } => method.assign(&local, type_ref, value),
                Operation::Assign {
                    local,
                    declared: None,
                    value,
                } => method.assign_var(&local, value),
                Operation::ForEach { variable, iterable } => method.for_each(&variable, iterable),
                Operation::Evaluate(value) => method.evaluate(value),
                Operation::GiveBack(value) => method.give_back(value),
            })
    }

    pub(super) fn into_lambda(self, lambda: Lambda) -> Lambda {
        self.operations
            .into_iter()
            .fold(lambda, |lambda, operation| match operation {
                Operation::Call(call) => lambda.call(call),
                Operation::Assign {
                    local,
                    declared: Some(type_ref),
                    value,
                } => lambda.assign(&local, type_ref, value),
                Operation::Assign {
                    local,
                    declared: None,
                    value,
                } => lambda.assign_var(&local, value),
                Operation::ForEach { variable, iterable } => lambda.for_each(&variable, iterable),
                Operation::Evaluate(value) | Operation::GiveBack(value) => lambda.evaluate(value),
            })
    }
}

pub(super) fn is_quiet(value: &Expression) -> bool {
    match value {
        Expression::Literal
        | Expression::This
        | Expression::Local(_)
        | Expression::Parameter { .. }
        | Expression::Field { .. }
        | Expression::StaticType(_)
        | Expression::Unresolved { .. } => true,
        Expression::Operation(operands) => operands.iter().all(is_quiet),
        Expression::FieldOf { target, .. } => is_quiet(target),
        _ => false,
    }
}
