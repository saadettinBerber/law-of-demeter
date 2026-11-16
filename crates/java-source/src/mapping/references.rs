use law_of_demeter_core::domain::type_shape::TypeRef;

use crate::{self as model, TypeName};

use super::MethodMapper;
use super::lambdas::functional_method;
use super::lineage::{class_name, closure, has_variable, lineage};
use super::overloads::{ArgumentType, is_known};
use super::returns::{Dispatch, Invocation, invoked};
use super::types::named_type;

impl MethodMapper<'_> {
    pub(super) fn reference_result(
        &mut self,
        target: &model::Expression,
        name: &str,
        functional: Option<&TypeRef>,
    ) -> Option<TypeRef> {
        let parameters = functional_method(self.types, functional?)?.parameters;
        match (target, name) {
            (model::Expression::Type(owner), "new") => self.constructed(owner),
            (model::Expression::Type(owner), _) => self.through_type(owner, name, &parameters),
            (_, "new") => None,
            _ => self.through_value(target, name, &parameters),
        }
    }

    pub(super) fn reference_input(
        &self,
        target: &model::Expression,
        name: &str,
        functional: Option<&TypeRef>,
    ) -> Option<TypeRef> {
        let unbound = matches!(target, model::Expression::Type(_)) && name != "new";
        let inputs = functional_method(self.types, functional?)?.parameters;
        inputs
            .into_iter()
            .next()
            .filter(|input| unbound && !has_variable(input))
    }

    fn constructed(&self, owner: &TypeName) -> Option<TypeRef> {
        let created = named_type(owner);
        let is_generic = class_name(&created)
            .and_then(|name| self.types.shape(name))
            .is_some_and(|shape| !shape.type_parameters().is_empty());
        (!is_generic).then_some(created)
    }

    fn through_type(
        &self,
        owner: &TypeName,
        name: &str,
        parameters: &[TypeRef],
    ) -> Option<TypeRef> {
        let owner = named_type(owner);
        let as_static = self.invoke(&owner, (name, Dispatch::Static), parameters);
        let as_unbound = parameters.split_first().and_then(|(receiver, arguments)| {
            let receiver = self.supertype_named(receiver, &owner)?;
            self.invoke(&receiver, (name, Dispatch::Instance), arguments)
        });
        match (as_static, as_unbound) {
            (Some(only), None) | (None, Some(only)) => only.returns_to(self.types),
            _ => None,
        }
    }

    fn through_value(
        &mut self,
        target: &model::Expression,
        name: &str,
        parameters: &[TypeRef],
    ) -> Option<TypeRef> {
        let receiver = self.type_of(target)?;
        self.invoke(&receiver, (name, Dispatch::Instance), parameters)?
            .returns_to(self.types)
    }

    fn supertype_named(&self, receiver: &TypeRef, owner: &TypeRef) -> Option<TypeRef> {
        closure(self.types, receiver)
            .into_iter()
            .find(|supertype| class_name(supertype) == class_name(owner))
    }

    fn invoke(
        &self,
        receiver: &TypeRef,
        method: (&str, Dispatch),
        parameters: &[TypeRef],
    ) -> Option<Invocation> {
        let arguments: Vec<_> = parameters
            .iter()
            .map(|parameter| {
                is_known(self.types, parameter).then(|| ArgumentType::Typed(parameter.clone()))
            })
            .collect();
        invoked(
            self.types,
            lineage(self.types, receiver),
            method,
            &arguments,
        )
    }
}
