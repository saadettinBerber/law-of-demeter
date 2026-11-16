use law_of_demeter_core::domain::method::{Argument, Call, Creation, Expression, MethodReference};

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
            model::Expression::Call(call) => self.call(call, None),
            model::Expression::New(created) => self.creation(created),
            model::Expression::Assignment { target, value } => self.assignment(target, value),
            model::Expression::Operation(operands) => {
                let operands = operands.iter().map(|operand| self.expression(operand));
                Expression::operation(operands.collect())
            }
            model::Expression::Cast { target, operand } => {
                let target = self.type_ref_or_object(target);
                let operand = self.expression_expecting(operand, Some(&target));
                Expression::cast(target, operand)
            }
            model::Expression::Lambda(lambda) => {
                Expression::lambda(self.lambda(lambda, None).lambda)
            }
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
            model::Expression::Literal(_) => Expression::literal(),
        }
    }

    pub(super) fn expression_expecting(
        &mut self,
        expression: &model::Expression,
        expected: Option<&TypeRef>,
    ) -> Expression {
        match expression {
            model::Expression::Call(call) => self.call(call, expected),
            model::Expression::Lambda(lambda) => {
                Expression::lambda(self.lambda(lambda, expected).lambda)
            }
            other => self.expression(other),
        }
    }

    fn call(&mut self, call: &model::Call, expected: Option<&TypeRef>) -> Expression {
        let receiver = match (&call.receiver, &call.implicit_target) {
            (Some(receiver), _) => self.expression(receiver),
            (None, Some(ImplicitTarget::StaticImport { owner })) => {
                Expression::static_type(named_type(owner))
            }
            (None, _) => Expression::This,
        };
        let invocation = self
            .invocation_of(call)
            .map(|found| found.expecting(self.types, expected));
        let mut mapped = Call::new(receiver, &call.name, call.line);
        let mut results = Vec::new();
        for (index, argument) in call.arguments.iter().enumerate() {
            let target = invocation.as_ref().and_then(|found| found.parameter(index));
            let (argument, result) = self.argument(argument, target);
            mapped = mapped.with(argument);
            results.push(result);
        }
        let returned = invocation.and_then(|found| found.returned(self.types, &results, expected));
        match returned {
            Some(returned) => Expression::return_of(mapped.returning(returned)),
            None => Expression::return_of(mapped),
        }
    }

    fn creation(&mut self, created: &model::New) -> Expression {
        let invocation = self.constructor_of(created);
        let mut creation = Creation::new(self.type_ref_or_object(&created.created));
        for (index, argument) in created.arguments.iter().enumerate() {
            let target = invocation.as_ref().and_then(|found| found.parameter(index));
            creation = creation.with(self.argument(argument, target).0);
        }
        Expression::created(creation)
    }

    fn argument(
        &mut self,
        argument: &model::Expression,
        target: Option<&TypeRef>,
    ) -> (Argument, Option<TypeRef>) {
        match argument {
            model::Expression::Lambda(lambda) => {
                let mapped = self.lambda(lambda, target);
                (Argument::Lambda(mapped.lambda), mapped.result)
            }
            model::Expression::MethodReference { target, name, line } => {
                let reference = MethodReference::new(self.expression(target), name, *line);
                (Argument::MethodReference(reference), None)
            }
            other => (Argument::Value(self.expression(other)), None),
        }
    }

    fn assignment(&mut self, target: &model::Expression, value: &model::Expression) -> Expression {
        let model::Expression::Local(name) = target else {
            let expected = self.field_type_of(target);
            let target = self.expression(target);
            self.steps.evaluate(target);
            let value = self.expression_expecting(value, expected.as_ref());
            return match expected {
                Some(type_ref) => expecting(value, type_ref),
                None => value,
            };
        };
        let declared = self.locals.get(name).cloned().flatten();
        let value = self.expression_expecting(value, declared.as_ref());
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
}

pub(super) fn expecting(value: Expression, type_ref: TypeRef) -> Expression {
    match value {
        Expression::ReturnOf(call) => Expression::return_of(call.expecting(type_ref)),
        Expression::Lambda(lambda) => Expression::lambda(lambda.expecting(type_ref)),
        other => other,
    }
}
