use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeRef, TypeShape};

use crate::{self as model, ImplicitTarget};

use super::MethodMapper;
use super::types::named_type;

const OBJECT: &str = "java.lang.Object";

impl MethodMapper<'_> {
    pub(super) fn returned_by(&self, call: &model::Call) -> Option<TypeRef> {
        let receiver = self.receiver_type(call)?;
        declared_return(
            self.types,
            class_name(&receiver)?,
            &call.name,
            call.arguments.len(),
        )
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
    receiver: &str,
    method: &str,
    argument_count: usize,
) -> Option<TypeRef> {
    let declared: Vec<_> = lineage(types, receiver)
        .into_iter()
        .map(|shape| named(shape, method))
        .find(|methods| methods.iter().any(takes(argument_count)))?;
    if declared.iter().any(|candidate| candidate.is_varargs()) {
        return None;
    }
    let matching: Vec<_> = declared.into_iter().filter(takes(argument_count)).collect();
    agreed_return(&matching)
}

fn takes(argument_count: usize) -> impl Fn(&&MethodShape) -> bool {
    move |candidate| candidate.parameters().len() == argument_count
}

fn lineage<'t>(types: &'t Types, receiver: &str) -> Vec<&'t TypeShape> {
    let mut names = vec![receiver.to_owned()];
    let mut index = 0;
    while index < names.len() {
        let supertypes = types
            .shape(&names[index])
            .map_or(&[][..], |shape| shape.supertypes());
        let unseen: Vec<_> = supertypes
            .iter()
            .filter_map(class_name)
            .filter(|name| !names.iter().any(|known| known == name))
            .map(str::to_owned)
            .collect();
        names.extend(unseen);
        index += 1;
    }
    if !names.iter().any(|name| name == OBJECT) {
        names.push(OBJECT.to_owned());
    }
    names.iter().filter_map(|name| types.shape(name)).collect()
}

fn named<'t>(shape: &'t TypeShape, method: &str) -> Vec<&'t MethodShape> {
    shape
        .methods()
        .iter()
        .filter(|candidate| candidate.name() == method)
        .collect()
}

fn agreed_return(candidates: &[&MethodShape]) -> Option<TypeRef> {
    let (first, rest) = candidates.split_first()?;
    let returned = first.returns()?;
    let agreed = rest
        .iter()
        .all(|candidate| candidate.returns() == Some(returned));
    (agreed && !has_variable(returned)).then(|| returned.clone())
}

fn has_variable(type_ref: &TypeRef) -> bool {
    match type_ref {
        TypeRef::Variable(_) => true,
        TypeRef::Array(element) => has_variable(element),
        TypeRef::Class { arguments, .. } => arguments.iter().any(has_variable),
        TypeRef::Primitive(_) => false,
    }
}

fn class_name(type_ref: &TypeRef) -> Option<&str> {
    match type_ref {
        TypeRef::Class { name, .. } => Some(name),
        _ => None,
    }
}
