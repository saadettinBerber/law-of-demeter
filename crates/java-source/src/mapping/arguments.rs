use law_of_demeter_core::domain::type_shape::TypeRef;

use crate::{self as model};

use super::MethodMapper;
use super::returns::Invocation;

pub(super) struct Resolved {
    pub(super) invocation: Invocation,
    pub(super) targets: Vec<Option<TypeRef>>,
}

#[derive(Clone, Copy, PartialEq)]
enum Round {
    Ready,
    Nested,
    Remaining,
}

impl MethodMapper<'_> {
    pub(super) fn resolve(
        &mut self,
        call: &model::Call,
        expected: Option<&TypeRef>,
    ) -> Option<Resolved> {
        let mut invocation = self.invocation_of(call)?.expecting(self.types, expected);
        let count = call.arguments.len();
        let mut targets: Vec<Option<TypeRef>> = vec![None; count];
        let mut attempted = vec![false; count];
        for round in [Round::Ready, Round::Nested, Round::Remaining] {
            for (index, argument) in call.arguments.iter().enumerate() {
                if !attempted[index] && self.takes_part(argument, round, &invocation, index) {
                    attempted[index] = true;
                    targets[index] = invocation.parameter(index).cloned();
                    invocation = self.bound_by(argument, invocation, index);
                }
            }
        }
        for (index, target) in targets.iter_mut().enumerate() {
            if !attempted[index] {
                *target = invocation.parameter(index).cloned();
            }
        }
        Some(Resolved {
            invocation,
            targets,
        })
    }

    fn takes_part(
        &mut self,
        argument: &model::Expression,
        round: Round,
        invocation: &Invocation,
        index: usize,
    ) -> bool {
        match argument {
            model::Expression::Lambda(_) | model::Expression::MethodReference { .. } => {
                invocation.awaits_result(self.types, index)
                    && match round {
                        Round::Ready => invocation.takes_known_inputs(self.types, index),
                        Round::Nested => false,
                        Round::Remaining => true,
                    }
            }
            model::Expression::Call(call) => round == Round::Nested && self.is_open(call),
            _ => false,
        }
    }

    fn bound_by(
        &mut self,
        argument: &model::Expression,
        invocation: Invocation,
        index: usize,
    ) -> Invocation {
        match self.result_of(argument, &invocation, index) {
            Some((pattern, result)) => invocation.binding(self.types, pattern, result),
            None => invocation,
        }
    }

    fn result_of(
        &mut self,
        argument: &model::Expression,
        invocation: &Invocation,
        index: usize,
    ) -> Option<(TypeRef, TypeRef)> {
        let target = invocation.parameter(index)?.clone();
        match argument {
            model::Expression::Lambda(lambda) => {
                let pattern = invocation.functional_returns(self.types, index)?;
                Some((pattern, self.lambda(lambda, Some(&target)).result?))
            }
            model::Expression::MethodReference {
                target: referenced,
                name,
                ..
            } => {
                let pattern = invocation.functional_returns(self.types, index)?;
                let result = self.reference_result(referenced, name, Some(&target))?;
                Some((pattern, result))
            }
            model::Expression::Call(call) => {
                let result = self.returned_by(call, Some(&target))?;
                Some((target, result))
            }
            _ => None,
        }
    }
}
