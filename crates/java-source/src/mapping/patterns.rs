use law_of_demeter_core::domain::method::Expression;

use crate::{self as model, PatternBinding, Switch};

use super::MethodMapper;
use super::steps::is_quiet;

const SUBJECT: &str = "<subject>";

impl MethodMapper<'_> {
    pub(super) fn instance_of(
        &mut self,
        operand: &model::Expression,
        bindings: &[PatternBinding],
    ) -> Expression {
        let operand = self.expression(operand);
        if bindings.is_empty() {
            return Expression::operation(vec![operand]);
        }
        let subject = self.stable(operand);
        self.bind(&subject, bindings);
        Expression::literal()
    }

    pub(super) fn switch(&mut self, switch: &Switch) -> Expression {
        let selector = self.expression(&switch.selector);
        let has_bindings = switch
            .branches
            .iter()
            .any(|branch| !branch.bindings.is_empty());
        let subject = if has_bindings {
            self.stable(selector)
        } else {
            self.steps.evaluate(selector);
            Expression::literal()
        };
        for branch in &switch.branches {
            self.exact_locals.clear();
            self.bind(&subject, &branch.bindings);
            if let Some(guard) = &branch.guard {
                self.evaluate(guard);
            }
            self.statements(&branch.body);
        }
        self.exact_locals.clear();
        Expression::literal()
    }

    fn stable(&mut self, value: Expression) -> Expression {
        if is_quiet(&value) {
            return value;
        }
        self.steps.assign(SUBJECT, None, value);
        Expression::local(SUBJECT)
    }

    fn bind(&mut self, subject: &Expression, bindings: &[PatternBinding]) {
        for binding in bindings {
            self.exact_locals.remove(&binding.name);
            let declared = self.type_ref_or_object(&binding.declared_type);
            self.locals
                .insert(binding.name.clone(), Some(declared.clone()));
            let value = Expression::cast(declared.clone(), subject.clone());
            self.steps.assign(&binding.name, Some(declared), value);
        }
    }
}
