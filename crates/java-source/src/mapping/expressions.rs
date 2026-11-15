use std::mem;

use law_of_demeter_core::domain::method::{
    Argument, Call, Creation, Expression, Lambda, MethodReference,
};

use law_of_demeter_core::domain::type_shape::TypeRef;

use crate::{self as model, ImplicitTarget};

use super::MethodMapper;
use super::steps::is_quiet;
use super::types::named_type;

impl MethodMapper<'_> {
    pub(super) fn expression(&mut self, expression: &model::Expression) -> Expression {
        match expression {
            model::Expression::This | model::Expression::Super => Expression::This,
            model::Expression::Parameter(name) => {
                Expression::parameter(name, self.parameter_type(name))
            }
            model::Expression::Local(name) | model::Expression::LambdaParameter(name) => {
                Expression::local(name)
            }
            model::Expression::Field(name) => {
                Expression::field(name, self.field_type(&self.class.name, name))
            }
            model::Expression::OuterField { owner, name } => {
                Expression::field(name, self.field_type(owner, name))
            }
            model::Expression::Captured(name) => Expression::field(name, self.captured_type(name)),
            model::Expression::Type(name) => Expression::static_type(named_type(name)),
            model::Expression::StaticImport { owner, name } => {
                Expression::field_of(Expression::static_type(named_type(owner)), name)
            }
            model::Expression::Unresolved(name) => Expression::unresolved(name),
            model::Expression::OnDemandStaticImport { name, candidates } => {
                Expression::unresolved_among(name, candidates.iter().map(named_type).collect())
            }
            model::Expression::FieldAccess { target, name } => {
                Expression::field_of(self.expression(target), name)
            }
            model::Expression::Call(call) => self.call(call),
            model::Expression::New(created) => self.creation(created),
            model::Expression::Assignment { target, value } => self.assignment(target, value),
            model::Expression::Operation(operands) => {
                let operands = operands.iter().map(|operand| self.expression(operand));
                Expression::operation(operands.collect())
            }
            model::Expression::Cast { target, operand } => {
                Expression::cast(self.type_ref_or_object(target), self.expression(operand))
            }
            model::Expression::Lambda(lambda) => Expression::lambda(self.lambda(lambda)),
            model::Expression::MethodReference { target, .. } => {
                let target = self.expression(target);
                if is_quiet(&target) {
                    Expression::literal()
                } else {
                    Expression::operation(vec![target])
                }
            }
            model::Expression::InstanceOf {
                operand, bindings, ..
            } => self.instance_of(operand, bindings),
            model::Expression::Switch(switch) => self.switch(switch),
            model::Expression::ClassLiteral(named) => Expression::cast(
                TypeRef::generic("java.lang.Class", vec![self.type_ref_or_object(named)]),
                Expression::literal(),
            ),
            model::Expression::Literal => Expression::literal(),
        }
    }

    fn call(&mut self, call: &model::Call) -> Expression {
        let receiver = match (&call.receiver, &call.implicit_target) {
            (Some(receiver), _) => self.expression(receiver),
            (None, Some(ImplicitTarget::StaticImport { owner })) => {
                Expression::static_type(named_type(owner))
            }
            (None, _) => Expression::This,
        };
        let mut mapped = Call::new(receiver, &call.name, call.line);
        for argument in &call.arguments {
            mapped = mapped.with(self.argument(argument));
        }
        Expression::return_of(mapped)
    }

    fn creation(&mut self, created: &model::New) -> Expression {
        let mut creation = Creation::new(self.type_ref_or_object(&created.created));
        for argument in &created.arguments {
            creation = creation.with(self.argument(argument));
        }
        Expression::created(creation)
    }

    fn argument(&mut self, argument: &model::Expression) -> Argument {
        match argument {
            model::Expression::Lambda(lambda) => Argument::Lambda(self.lambda(lambda)),
            model::Expression::MethodReference { target, name, .. } => {
                Argument::MethodReference(MethodReference::new(self.expression(target), name))
            }
            other => Argument::Value(self.expression(other)),
        }
    }

    fn assignment(&mut self, target: &model::Expression, value: &model::Expression) -> Expression {
        let model::Expression::Local(name) = target else {
            let expected = self.field_type_of(target);
            let target = self.expression(target);
            self.steps.evaluate(target);
            let value = self.expression(value);
            return match expected {
                Some(type_ref) => expecting(value, type_ref),
                None => value,
            };
        };
        let value = self.expression(value);
        let declared = self.locals.get(name).cloned().flatten();
        self.steps.assign(name, declared, value);
        Expression::local(name)
    }

    fn field_type_of(&self, target: &model::Expression) -> Option<TypeRef> {
        match target {
            model::Expression::Field(name) => Some(self.field_type(&self.class.name, name)),
            model::Expression::OuterField { owner, name } => Some(self.field_type(owner, name)),
            _ => None,
        }
    }

    fn lambda(&mut self, lambda: &model::Lambda) -> Lambda {
        let names: Vec<&str> = lambda
            .parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect();
        let (result, body) = match lambda.body.split_last() {
            Some((model::Statement::Return(Some(result)), earlier)) => (Some(result), earlier),
            _ => (None, lambda.body.as_slice()),
        };
        let outer_steps = mem::take(&mut self.steps);
        let outer_returns = self.returns.take();
        self.statements(body);
        let result = result.map(|result| self.expression(result));
        self.returns = outer_returns;
        let mapped = mem::replace(&mut self.steps, outer_steps).into_lambda(Lambda::new(&names));
        let mapped = lambda
            .parameters
            .iter()
            .filter_map(|parameter| {
                Some((parameter, self.type_ref(parameter.declared_type.as_ref()?)?))
            })
            .fold(mapped, |mapped, (parameter, declared)| {
                mapped.declaring(&parameter.name, declared)
            });
        match result {
            Some(value) => mapped.yielding(value),
            None => mapped,
        }
    }
}

pub(super) fn expecting(value: Expression, type_ref: TypeRef) -> Expression {
    match value {
        Expression::ReturnOf(call) => Expression::return_of(call.expecting(type_ref)),
        Expression::Lambda(lambda) => Expression::lambda(lambda.expecting(type_ref)),
        other => other,
    }
}
