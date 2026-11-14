use std::mem;

use law_of_demeter_core::domain::method::{Argument, Call, Creation, Expression, Lambda};

use crate::{self as model, ImplicitTarget};

use super::MethodMapper;
use super::steps::is_quiet;
use super::types::{named_type, object, type_ref_or_object};

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
                Expression::cast(type_ref_or_object(target), self.expression(operand))
            }
            model::Expression::Lambda(lambda) => {
                self.inline_lambda(lambda);
                Expression::literal()
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
        let mut creation = Creation::new(type_ref_or_object(&created.created));
        for argument in &created.arguments {
            creation = creation.with(self.argument(argument));
        }
        Expression::created(creation)
    }

    fn argument(&mut self, argument: &model::Expression) -> Argument {
        match argument {
            model::Expression::Lambda(lambda) => Argument::Lambda(self.lambda(lambda)),
            other => Argument::Value(self.expression(other)),
        }
    }

    fn assignment(&mut self, target: &model::Expression, value: &model::Expression) -> Expression {
        let model::Expression::Local(name) = target else {
            let target = self.expression(target);
            self.steps.evaluate(target);
            return self.expression(value);
        };
        let value = self.expression(value);
        let declared = self.locals.get(name).cloned().flatten();
        self.steps.assign(name, declared, value);
        Expression::local(name)
    }

    fn lambda(&mut self, lambda: &model::Lambda) -> Lambda {
        let names: Vec<&str> = lambda
            .parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect();
        let outer_steps = mem::take(&mut self.steps);
        self.statements(&lambda.body);
        mem::replace(&mut self.steps, outer_steps).into_lambda(Lambda::new(&names))
    }

    fn inline_lambda(&mut self, lambda: &model::Lambda) {
        for parameter in &lambda.parameters {
            let declared = parameter
                .declared_type
                .as_ref()
                .map_or_else(object, type_ref_or_object);
            self.locals
                .insert(parameter.name.clone(), Some(declared.clone()));
            let value = Expression::parameter(&parameter.name, declared.clone());
            self.steps.assign(&parameter.name, Some(declared), value);
        }
        self.statements(&lambda.body);
    }
}
