use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeRef, TypeShape};

use crate::{self as model, ImplicitTarget};

use super::MethodMapper;
use super::lineage::{Ancestor, has_variable, lineage, substitute};
use super::overloads::{ArgumentType, Selection, is_known, select};
use super::types::named_type;

impl MethodMapper<'_> {
    pub(super) fn returned_by(&self, call: &model::Call) -> Option<TypeRef> {
        let receiver = self.receiver_type(call)?;
        let arguments: Vec<_> = call
            .arguments
            .iter()
            .map(|argument| self.argument_type(argument))
            .collect();
        declared_return(self.types, &receiver, &call.name, &arguments)
    }

    fn argument_type(&self, argument: &model::Expression) -> Option<ArgumentType> {
        match argument {
            model::Expression::Literal(literal) => Some(ArgumentType::of_literal(*literal)),
            other => self
                .type_of(other)
                .filter(|type_ref| is_known(self.types, type_ref))
                .map(ArgumentType::Typed),
        }
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
            model::Expression::New(created) => self.type_ref(&created.created),
            model::Expression::Cast { target, .. } => self.type_ref(target),
            _ => None,
        }
    }
}

fn declared_return(
    types: &Types,
    receiver: &TypeRef,
    method: &str,
    arguments: &[Option<ArgumentType>],
) -> Option<TypeRef> {
    let ancestors = lineage(types, receiver);
    let candidates = candidates(&ancestors, method, arguments.len());
    if candidates.is_empty()
        || candidates
            .iter()
            .any(|candidate| candidate.method.is_varargs())
    {
        return None;
    }
    let parameters: Vec<_> = candidates.iter().map(Candidate::parameters).collect();
    let returned = match select(types, &parameters, arguments) {
        Selection::Chosen(index) => candidates[index].returned(),
        Selection::Undecided => agreed_return(&candidates),
        Selection::Unresolvable => None,
    };
    returned.filter(|returned| !has_variable(returned))
}

struct Candidate<'a> {
    method: &'a MethodShape,
    ancestor: &'a Ancestor<'a>,
}

impl Candidate<'_> {
    fn parameters(&self) -> Vec<TypeRef> {
        self.method
            .parameters()
            .iter()
            .map(|parameter| self.specialize(parameter))
            .collect()
    }

    fn returned(&self) -> Option<TypeRef> {
        let returned = self.method.returns()?;
        if self.ancestor.erased && mentions_generics(returned) {
            return None;
        }
        Some(self.specialize(returned))
    }

    fn specialize(&self, type_ref: &TypeRef) -> TypeRef {
        let own = self.method.type_parameters();
        let shadowed = |name: &str| own.iter().any(|variable| variable == name);
        substitute(type_ref, &|name| {
            self.ancestor.bound(name).filter(|_| !shadowed(name))
        })
    }
}

fn candidates<'a>(
    ancestors: &'a [Ancestor<'a>],
    method: &str,
    argument_count: usize,
) -> Vec<Candidate<'a>> {
    let mut candidates: Vec<Candidate> = Vec::new();
    for ancestor in ancestors {
        for declared in named(ancestor.shape, method) {
            let candidate = Candidate {
                method: declared,
                ancestor,
            };
            let overridden = candidates
                .iter()
                .any(|known| known.parameters() == candidate.parameters());
            if declared.parameters().len() == argument_count && !overridden {
                candidates.push(candidate);
            }
        }
    }
    candidates
}

fn named<'t>(shape: &'t TypeShape, method: &str) -> Vec<&'t MethodShape> {
    shape
        .methods()
        .iter()
        .filter(|candidate| candidate.name() == method)
        .collect()
}

fn agreed_return(candidates: &[Candidate]) -> Option<TypeRef> {
    let (first, rest) = candidates.split_first()?;
    let returned = first.returned()?;
    let agreed = rest
        .iter()
        .all(|candidate| candidate.returned().as_ref() == Some(&returned));
    agreed.then_some(returned)
}

fn mentions_generics(type_ref: &TypeRef) -> bool {
    match type_ref {
        TypeRef::Variable(_) => true,
        TypeRef::Array(element) => mentions_generics(element),
        TypeRef::Class { arguments, .. } => !arguments.is_empty(),
        TypeRef::Primitive(_) => false,
    }
}
