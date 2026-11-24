use law_of_demeter_core::domain::type_shape::TypeRef;

use crate::{self as model};

use super::MethodMapper;
use super::lineage::class_name;
use super::links::Declared;
use super::memo::address;

impl MethodMapper<'_> {
    pub(super) fn record_call_identity(
        &mut self,
        call: &model::Call,
        declared: Option<&Declared>,
    ) -> bool {
        let exact_receiver = self.exact_receiver(call);
        let fixed =
            declared.is_some_and(|declared| declared.has_fixed_dispatch() || exact_receiver);
        let returns_receiver = fixed && declared.is_some_and(Declared::returns_receiver);
        let same_return_type = self
            .receiver_type(call)
            .zip(self.returned_by(call, None))
            .is_some_and(|(receiver, returned)| {
                class_name(&receiver).is_some() && class_name(&receiver) == class_name(&returned)
            });
        let exact_result = fixed
            && (declared.is_some_and(Declared::returns_exact_type)
                || (returns_receiver && exact_receiver && same_return_type));
        self.memo.exact_calls.insert(address(call), exact_result);
        returns_receiver
    }

    fn exact_receiver(&mut self, call: &model::Call) -> bool {
        if self
            .receiver_type(call)
            .as_ref()
            .is_some_and(|type_ref| self.final_type(type_ref))
        {
            return true;
        }
        call.receiver
            .as_deref()
            .is_some_and(|receiver| self.exact_value(receiver))
    }

    pub(super) fn exact_value(&self, value: &model::Expression) -> bool {
        match value {
            model::Expression::New(created) => created.anonymous_class.is_none(),
            model::Expression::Call(call) => self
                .memo
                .exact_calls
                .get(&address(call))
                .copied()
                .unwrap_or(false),
            model::Expression::Local(name) => self.exact_locals.contains(name),
            model::Expression::Assignment { value, .. } => self.exact_value(value),
            _ => false,
        }
    }

    pub(super) fn record_local_identity(
        &mut self,
        name: &str,
        declared: Option<&TypeRef>,
        value: &model::Expression,
    ) {
        let actual = self.type_of(value);
        let matching = declared.or(actual.as_ref()).and_then(class_name)
            == actual.as_ref().and_then(class_name);
        if matching && actual.is_some() && self.exact_value(value) {
            self.exact_locals.insert(name.to_owned());
            if declared.is_none() {
                self.locals.insert(name.to_owned(), actual);
            }
        } else {
            self.exact_locals.remove(name);
        }
    }

    pub(super) fn final_type(&self, type_ref: &TypeRef) -> bool {
        class_name(type_ref)
            .and_then(|name| self.types.shape(name))
            .is_some_and(|shape| shape.is_final())
    }
}
