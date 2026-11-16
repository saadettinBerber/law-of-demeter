use std::collections::HashMap;

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeRef, TypeShape};

use crate::{self as model, ImplicitTarget};

use super::MethodMapper;
use super::types::named_type;

const OBJECT: &str = "java.lang.Object";

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

struct Ancestor<'t> {
    shape: &'t TypeShape,
    bindings: HashMap<String, TypeRef>,
    erased: bool,
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

fn lineage<'t>(types: &'t Types, receiver: &TypeRef) -> Vec<Ancestor<'t>> {
    let mut queue = vec![receiver.clone()];
    let mut ancestors = Vec::new();
    let mut index = 0;
    while index < queue.len() {
        if let Some(ancestor) = class_name(&queue[index]).and_then(|name| types.shape(name)) {
            let arguments = arguments_of(&queue[index]);
            let erased = arguments.is_empty() && !ancestor.type_parameters().is_empty();
            let bindings: HashMap<_, _> = ancestor
                .type_parameters()
                .iter()
                .cloned()
                .zip(arguments.iter().cloned())
                .collect();
            let supertypes = ancestor.supertypes().iter().map(|supertype| {
                if erased {
                    erase(supertype)
                } else {
                    substitute(supertype, &|name| bindings.get(name).cloned())
                }
            });
            let unseen: Vec<_> = supertypes
                .filter(|supertype| !queue.iter().any(|known| same_class(known, supertype)))
                .collect();
            queue.extend(unseen);
            ancestors.push(Ancestor {
                shape: ancestor,
                bindings,
                erased,
            });
        }
        index += 1;
    }
    if !queue.iter().any(|known| class_name(known) == Some(OBJECT)) {
        ancestors.extend(types.shape(OBJECT).map(|shape| Ancestor {
            shape,
            bindings: HashMap::new(),
            erased: false,
        }));
    }
    ancestors
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
    Some(substitute(returned, &|name| {
        if own.iter().any(|variable| variable == name) {
            return None;
        }
        ancestor.bindings.get(name).cloned()
    }))
}

fn substitute(type_ref: &TypeRef, binding: &dyn Fn(&str) -> Option<TypeRef>) -> TypeRef {
    match type_ref {
        TypeRef::Variable(name) => binding(name).unwrap_or_else(|| type_ref.clone()),
        TypeRef::Array(element) => TypeRef::array(substitute(element, binding)),
        TypeRef::Class { name, arguments } => TypeRef::generic(
            name,
            arguments
                .iter()
                .map(|argument| substitute(argument, binding))
                .collect(),
        ),
        TypeRef::Primitive(_) => type_ref.clone(),
    }
}

fn erase(type_ref: &TypeRef) -> TypeRef {
    class_name(type_ref).map_or_else(|| type_ref.clone(), TypeRef::class)
}

fn arguments_of(type_ref: &TypeRef) -> &[TypeRef] {
    match type_ref {
        TypeRef::Class { arguments, .. } => arguments,
        _ => &[],
    }
}

fn same_class(left: &TypeRef, right: &TypeRef) -> bool {
    class_name(left) == class_name(right)
}

fn mentions_generics(type_ref: &TypeRef) -> bool {
    match type_ref {
        TypeRef::Variable(_) => true,
        TypeRef::Array(element) => mentions_generics(element),
        TypeRef::Class { arguments, .. } => !arguments.is_empty(),
        TypeRef::Primitive(_) => false,
    }
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
