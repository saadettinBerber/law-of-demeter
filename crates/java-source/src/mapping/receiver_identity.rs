use law_of_demeter_core::domain::type_shape::TypeRef;

use crate::{self as model};

use super::MethodMapper;
use super::lineage::class_name;
use super::links::Declared;

impl MethodMapper<'_> {
    pub(super) fn fixed_dispatch(&mut self, call: &model::Call, declared: &Declared) -> bool {
        declared.has_fixed_dispatch() || self.exact_receiver(call)
    }

    fn exact_receiver(&mut self, call: &model::Call) -> bool {
        if self
            .receiver_type(call)
            .as_ref()
            .is_some_and(|type_ref| self.final_type(type_ref))
        {
            return true;
        }
        match call.receiver.as_deref() {
            Some(model::Expression::New(created)) => created.anonymous_class.is_none(),
            Some(model::Expression::Call(previous)) => self.exact_result(previous),
            _ => false,
        }
    }

    fn exact_result(&mut self, call: &model::Call) -> bool {
        let Some(declared) = self
            .invocation_of(call)
            .and_then(|invocation| invocation.declared().cloned())
        else {
            return false;
        };
        let same_return_type = self
            .receiver_type(call)
            .zip(self.returned_by(call, None))
            .is_some_and(|(receiver, returned)| {
                class_name(&receiver).is_some() && class_name(&receiver) == class_name(&returned)
            });
        self.fixed_dispatch(call, &declared)
            && (declared.returns_exact_type()
                || (declared.returns_receiver() && self.exact_receiver(call) && same_return_type))
    }

    pub(super) fn final_type(&self, type_ref: &TypeRef) -> bool {
        class_name(type_ref)
            .and_then(|name| self.types.shape(name))
            .is_some_and(|shape| shape.is_final())
    }
}
