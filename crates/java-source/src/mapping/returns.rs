use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeRef, TypeShape};

use crate::{self as model, ImplicitTarget};

use super::MethodMapper;
use super::lineage::{Ancestor, has_variable, lineage, substitute};
use super::types::named_type;

impl MethodMapper<'_> {
    pub(super) fn returned_by(&self, call: &model::Call) -> Option<TypeRef> {
        let receiver = self.receiver_type(call)?;
        declared_return(self.types, &receiver, &call.name, call.arguments.len())
    }

    fn receiver_type(&self, call: &model::Call) -> Option<TypeRef> {
        match (&call.receiver, &call.implicit_target) {
            (Some(receiver), _) => self.type_of(receiver),
            (None, Some(ImplicitTarget::StaticImport { owner })) => Some(named_type(owner)),
            (None, Some(ImplicitTarget::OwnMethod) | None) => {
                Some(TypeRef::class(&self.class.name))
            }
            (None, Some(_)) => None,
        }
    }

    fn type_of(&self, expression: &model::Expression) -> Option<TypeRef> {
        match expression {
            model::Expression::This => Some(TypeRef::class(&self.class.name)),
            model::Expression::Parameter(name) => Some(self.parameter_type(name)),
            model::Expression::Local(name) => self.locals.get(name).cloned().flatten(),
            model::Expression::Field(name) => Some(self.field_type(&self.class.name, name)),
            model::Expression::OuterField { owner, name } => Some(self.field_type(owner, name)),
            model::Expression::Captured(name) => Some(self.captured_type(name)),
            model::Expression::Type(name) => Some(named_type(name)),
            model::Expression::Call(call) => self.returned_by(call),
            _ => None,
        }
    }
}

fn declared_return(
    types: &Types,
    receiver: &TypeRef,
    method: &str,
    argument_count: usize,
) -> Option<TypeRef> {
    let (ancestor, declared) = lineage(types, receiver)
        .into_iter()
        .map(|ancestor| {
            let methods = named(ancestor.shape, method);
            (ancestor, methods)
        })
        .find(|(_, methods)| methods.iter().any(takes(argument_count)))?;
    if declared.iter().any(|candidate| candidate.is_varargs()) {
        return None;
    }
    let matching: Vec<_> = declared.into_iter().filter(takes(argument_count)).collect();
    agreed_return(&matching, &ancestor)
}

fn takes(argument_count: usize) -> impl Fn(&&MethodShape) -> bool {
    move |candidate| candidate.parameters().len() == argument_count
}

fn named<'t>(shape: &'t TypeShape, method: &str) -> Vec<&'t MethodShape> {
    shape
        .methods()
        .iter()
        .filter(|candidate| candidate.name() == method)
        .collect()
}

fn agreed_return(candidates: &[&MethodShape], ancestor: &Ancestor) -> Option<TypeRef> {
    let (first, rest) = candidates.split_first()?;
    let returned = returned_by_candidate(first, ancestor)?;
    let agreed = rest
        .iter()
        .all(|candidate| returned_by_candidate(candidate, ancestor).as_ref() == Some(&returned));
    (agreed && !has_variable(&returned)).then_some(returned)
}

fn returned_by_candidate(candidate: &MethodShape, ancestor: &Ancestor) -> Option<TypeRef> {
    let returned = candidate.returns()?;
    if ancestor.erased && mentions_generics(returned) {
        return None;
    }
    let own = candidate.type_parameters();
    let shadowed = |name: &str| own.iter().any(|variable| variable == name);
    Some(substitute(returned, &|name| {
        ancestor.bound(name).filter(|_| !shadowed(name))
    }))
}

fn mentions_generics(type_ref: &TypeRef) -> bool {
    match type_ref {
        TypeRef::Variable(_) => true,
        TypeRef::Array(element) => mentions_generics(element),
        TypeRef::Class { arguments, .. } => !arguments.is_empty(),
        TypeRef::Primitive(_) => false,
    }
}
