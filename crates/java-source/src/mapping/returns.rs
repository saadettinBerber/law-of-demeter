use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeRef, TypeShape};

use crate::{self as model, ImplicitTarget};

use super::MethodMapper;
use super::inference::{Bindings, apply_bindings, complete};
use super::lineage::{Ancestor, has_variable, lineage, substitute};
use super::overloads::{ArgumentType, Selection, Signature, is_known, select};
use super::types::named_type;

impl MethodMapper<'_> {
    pub(super) fn returned_by(
        &self,
        call: &model::Call,
        expected: Option<&TypeRef>,
    ) -> Option<TypeRef> {
        let receiver = self.receiver_type(call)?;
        let arguments: Vec<_> = call
            .arguments
            .iter()
            .map(|argument| self.argument_type(argument))
            .collect();
        declared_return(self.types, &receiver, &call.name, &arguments, expected)
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
            model::Expression::Call(call) => self.returned_by(call, None),
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
    expected: Option<&TypeRef>,
) -> Option<TypeRef> {
    let ancestors = lineage(types, receiver);
    let candidates = candidates(&ancestors, method, arguments.len());
    if candidates.is_empty() {
        return None;
    }
    let signatures: Vec<_> = candidates.iter().map(Candidate::signature).collect();
    let returned = match select(types, &signatures, arguments) {
        Selection::Chosen { index, bindings } => {
            candidates[index].returned_under(types, &bindings, expected)
        }
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

    fn signature(&self) -> Signature {
        Signature {
            parameters: self.parameters(),
            variable_arity: self.method.is_varargs(),
            free: self.method.type_parameters().to_vec(),
        }
    }

    fn returned_under(
        &self,
        types: &Types,
        bindings: &Bindings,
        expected: Option<&TypeRef>,
    ) -> Option<TypeRef> {
        let returned = self.returned()?;
        let completed = match expected {
            Some(expected) => complete(
                types,
                self.method.type_parameters(),
                &returned,
                bindings,
                expected,
            )?,
            None => bindings.clone(),
        };
        Some(apply_bindings(&returned, &self.declared_bounds(completed)))
    }

    fn declared_bounds(&self, mut bindings: Bindings) -> Bindings {
        for name in self.method.type_parameters() {
            if let Some(bound) = self.method.type_parameter_bound(name) {
                bindings
                    .entry(name.clone())
                    .or_insert_with(|| bound.clone());
            }
        }
        bindings
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
            if candidate.signature().takes(argument_count) && !overridden {
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
