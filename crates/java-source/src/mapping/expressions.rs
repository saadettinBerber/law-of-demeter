use law_of_demeter_core::domain::method::{Argument, Call, Creation, Expression};

use crate::{self as model, ImplicitTarget};

use super::MethodMapper;
use super::types::{named_type, type_ref_or_object};

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
            model::Expression::Literal => Expression::literal(),
            _ => Expression::literal(),
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
        Argument::Value(self.expression(argument))
    }
}
