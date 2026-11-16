use law_of_demeter_core::domain::method::{Argument, Call, Creation, Expression, MethodReference};

use law_of_demeter_core::domain::type_shape::TypeRef;

use crate::{self as model, ImplicitTarget};

use super::MethodMapper;
use super::links::{ArgumentLinks, Declared, Flow};
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
            model::Expression::MethodReference { target, name, line } => {
                let reference = MethodReference::new(self.expression(target), name, *line);
                Expression::method_reference(reference)
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
        self.returning_call(call, expected, &[])
    }

    fn returning_call(
        &mut self,
        call: &model::Call,
        expected: Option<&TypeRef>,
        inherited: &[String],
    ) -> Expression {
        let mapped = self.mapped_call(call, expected, inherited);
        match self.returned_by(call, expected) {
            Some(returned) => Expression::return_of(mapped.returning(returned)),
            None => Expression::return_of(mapped),
        }
    }

    fn mapped_call(
        &mut self,
        call: &model::Call,
        expected: Option<&TypeRef>,
        inherited: &[String],
    ) -> Call {
        let receiver = match (&call.receiver, &call.implicit_target) {
            (Some(receiver), _) => self.expression(receiver),
            (None, Some(ImplicitTarget::StaticImport { owner })) => {
                Expression::static_type(named_type(owner))
            }
            (None, _) => Expression::This,
        };
        let resolved = self.resolve(call, expected);
        let declared = resolved
            .as_ref()
            .and_then(|resolved| resolved.invocation.declared().cloned());
        let targets = resolved
            .map(|resolved| resolved.targets)
            .unwrap_or_default();
        let mut mapped = Call::new(receiver, &call.name, call.line);
        for (index, argument) in call.arguments.iter().enumerate() {
            let target = targets.get(index).and_then(Option::as_ref);
            let links = self.argument_links(declared.as_ref(), index, argument, inherited);
            mapped = mapped.with(self.argument(argument, target, &links));
        }
        match declared {
            Some(declared) => self.linked(mapped, call, &declared),
            None => mapped,
        }
    }

    fn produces_keys(&mut self, call: &model::Call) -> bool {
        self.invocation_of(call)
            .and_then(|found| {
                found
                    .declared()
                    .map(|declared| declared.produces_keys(self.types))
            })
            .unwrap_or(false)
    }

    fn argument_links(
        &mut self,
        declared: Option<&Declared>,
        index: usize,
        argument: &model::Expression,
        inherited: &[String],
    ) -> ArgumentLinks {
        let Some(declared) = declared else {
            return ArgumentLinks::default();
        };
        match argument {
            model::Expression::Lambda(_) | model::Expression::MethodReference { .. } => {
                declared.links(self.types, index, inherited)
            }
            model::Expression::Call(nested) => {
                let nested = self
                    .invocation_of(nested)
                    .and_then(|found| found.declared().cloned());
                ArgumentLinks {
                    inherited: nested.map_or_else(Vec::new, |nested| {
                        declared.inherited_by(index, &nested, inherited)
                    }),
                    ..ArgumentLinks::default()
                }
            }
            _ => ArgumentLinks::default(),
        }
    }

    fn linked(&mut self, mapped: Call, call: &model::Call, declared: &Declared) -> Call {
        let flows = self.flows_of(call, declared);
        let mapped = with_feeding(mapped, call, &flows);
        let mapped = match flows.iter().position(|flow| *flow == Flow::Keys) {
            Some(index) => mapped.keyed_by(index),
            None => mapped,
        };
        if declared.yields_keys() {
            mapped.yielding_keys()
        } else {
            mapped
        }
    }

    fn flows_of(&mut self, call: &model::Call, declared: &Declared) -> Vec<Flow> {
        call.arguments
            .iter()
            .enumerate()
            .map(|(index, argument)| match argument {
                model::Expression::Call(nested) if self.produces_keys(nested) => {
                    declared.value_flow(self.types, index)
                }
                _ if is_functional(argument) => declared.result_flow(self.types, index),
                _ => Flow::None,
            })
            .collect()
    }

    fn creation(&mut self, created: &model::New) -> Expression {
        let invocation = self.constructor_of(created);
        let mut creation = Creation::new(self.type_ref_or_object(&created.created));
        for (index, argument) in created.arguments.iter().enumerate() {
            let target = invocation.as_ref().and_then(|found| found.parameter(index));
            creation = creation.with(self.argument(argument, target, &ArgumentLinks::default()));
        }
        Expression::created(creation)
    }

    fn argument(
        &mut self,
        argument: &model::Expression,
        target: Option<&TypeRef>,
        links: &ArgumentLinks,
    ) -> Argument {
        match argument {
            model::Expression::Lambda(lambda) => {
                let names: Vec<&str> = lambda
                    .parameters
                    .iter()
                    .map(|parameter| parameter.name.as_str())
                    .collect();
                let mapped = self.lambda(lambda, target).lambda;
                Argument::Lambda(links.on_lambda(mapped, &names))
            }
            model::Expression::MethodReference {
                target: referenced,
                name,
                line,
            } => {
                let result = self.reference_result(referenced, name, target);
                let reference = MethodReference::new(self.expression(referenced), name, *line);
                let reference = links.on_reference(reference);
                let reference = match self.reference_input(referenced, name, target) {
                    Some(input) => reference.resolving(input),
                    None => reference,
                };
                match result {
                    Some(returned) => Argument::MethodReference(reference.returning(returned)),
                    None => Argument::MethodReference(reference),
                }
            }
            model::Expression::Call(call) if self.is_open(call) => {
                Argument::Value(self.returning_call(call, target, &links.inherited))
            }
            other => Argument::Value(self.expression(other)),
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

fn with_feeding(mapped: Call, call: &model::Call, flows: &[Flow]) -> Call {
    if !call.arguments.iter().any(is_functional) {
        return mapped;
    }
    let feeding: Vec<usize> = call
        .arguments
        .iter()
        .enumerate()
        .filter(|(index, argument)| is_functional(argument) && flows[*index] == Flow::Return)
        .map(|(index, _)| index)
        .collect();
    mapped.returning_from(&feeding)
}

fn is_functional(argument: &model::Expression) -> bool {
    matches!(
        argument,
        model::Expression::Lambda(_) | model::Expression::MethodReference { .. }
    )
}

pub(super) fn expecting(value: Expression, type_ref: TypeRef) -> Expression {
    match value {
        Expression::ReturnOf(call) => Expression::return_of(call.expecting(type_ref)),
        Expression::Lambda(lambda) => Expression::lambda(lambda.expecting(type_ref)),
        Expression::MethodReference(reference) => {
            Expression::method_reference(reference.expecting(type_ref))
        }
        other => other,
    }
}
