use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeRef, TypeShape};

use crate::platform::CONSTRUCTOR_NAME;
use crate::{self as model, ImplicitTarget};

use super::MethodMapper;
use super::inference::{Bindings, apply_bindings, bind_results, complete};
use super::lambdas::functional_method;
use super::lineage::{Ancestor, class_name, has_variable, lineage, substitute};
use super::overloads::{ArgumentType, Selection, Signature, is_known, select};
use super::types::named_type;

impl MethodMapper<'_> {
    pub(super) fn returned_by(
        &mut self,
        call: &model::Call,
        expected: Option<&TypeRef>,
    ) -> Option<TypeRef> {
        let invocation = self.invocation_of(call)?;
        let results = self.lambda_results(&call.arguments, &invocation);
        invocation.returned(self.types, &results, expected)
    }

    fn lambda_results(
        &mut self,
        arguments: &[model::Expression],
        invocation: &Invocation,
    ) -> Vec<Option<TypeRef>> {
        arguments
            .iter()
            .enumerate()
            .map(|(index, argument)| match argument {
                model::Expression::Lambda(lambda)
                    if invocation.awaits_result(self.types, index) =>
                {
                    self.lambda(lambda, invocation.parameter(index)).result
                }
                _ => None,
            })
            .collect()
    }

    pub(super) fn invocation_of(&mut self, call: &model::Call) -> Option<Invocation> {
        let receiver = self.receiver_type(call)?;
        let arguments = self.argument_types(&call.arguments);
        invoked(
            self.types,
            lineage(self.types, &receiver),
            &call.name,
            &arguments,
        )
    }

    pub(super) fn constructor_of(&mut self, created: &model::New) -> Option<Invocation> {
        let receiver = self.type_ref(&created.created)?;
        let arguments = self.argument_types(&created.arguments);
        let mut own = lineage(self.types, &receiver);
        own.truncate(1);
        let is_the_class = own
            .first()
            .is_some_and(|ancestor| Some(ancestor.shape.name()) == class_name(&receiver));
        if !is_the_class {
            return None;
        }
        invoked(self.types, own, CONSTRUCTOR_NAME, &arguments)
    }

    fn argument_types(&mut self, arguments: &[model::Expression]) -> Vec<Option<ArgumentType>> {
        arguments
            .iter()
            .map(|argument| self.argument_type(argument))
            .collect()
    }

    fn argument_type(&mut self, argument: &model::Expression) -> Option<ArgumentType> {
        match argument {
            model::Expression::Literal(literal) => Some(ArgumentType::of_literal(*literal)),
            model::Expression::Lambda(lambda) => {
                Some(ArgumentType::Lambda(lambda.parameters.len()))
            }
            other => self
                .type_of(other)
                .filter(|type_ref| is_known(self.types, type_ref))
                .map(ArgumentType::Typed),
        }
    }

    fn receiver_type(&mut self, call: &model::Call) -> Option<TypeRef> {
        match (&call.receiver, &call.implicit_target) {
            (Some(receiver), _) => self.type_of(receiver),
            (None, Some(ImplicitTarget::StaticImport { owner })) => Some(named_type(owner)),
            (None, Some(ImplicitTarget::OwnMethod) | None) => {
                Some(TypeRef::class(&self.class.name))
            }
            (None, Some(_)) => None,
        }
    }

    pub(super) fn type_of(&mut self, expression: &model::Expression) -> Option<TypeRef> {
        match expression {
            model::Expression::This => Some(TypeRef::class(&self.class.name)),
            model::Expression::Parameter(name) => Some(self.parameter_type(name)),
            model::Expression::LambdaParameter(name) => self.lambda_parameters.get(name).cloned(),
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

pub(super) struct Invocation {
    parameters: Vec<TypeRef>,
    bindings: Bindings,
    returned: Option<TypeRef>,
    free: Vec<String>,
    bounds: Bindings,
}

impl Invocation {
    pub(super) fn parameter(&self, index: usize) -> Option<&TypeRef> {
        self.parameters.get(index)
    }

    pub(super) fn awaits_result(&self, types: &Types, index: usize) -> bool {
        self.functional_returns(types, index)
            .is_some_and(|returned| has_variable(&returned))
    }

    pub(super) fn returned(
        &self,
        types: &Types,
        results: &[Option<TypeRef>],
        expected: Option<&TypeRef>,
    ) -> Option<TypeRef> {
        let returned = self.returned.as_ref()?;
        let bound = self.bound_by(types, results);
        let completed = match expected {
            Some(expected) => complete(types, &self.free, returned, &bound, expected)?,
            None => bound,
        };
        let applied = apply_bindings(returned, &self.with_declared_bounds(completed));
        (!has_variable(&applied)).then_some(applied)
    }

    fn bound_by(&self, types: &Types, results: &[Option<TypeRef>]) -> Bindings {
        let returns: Vec<_> = results
            .iter()
            .enumerate()
            .filter_map(|(index, result)| {
                Some((self.functional_returns(types, index)?, result.as_ref()?))
            })
            .collect();
        let pairs: Vec<_> = returns
            .iter()
            .map(|(returned, result)| (returned, *result))
            .collect();
        bind_results(types, &self.free, &self.bindings, &pairs)
    }

    fn functional_returns(&self, types: &Types, index: usize) -> Option<TypeRef> {
        functional_method(types, self.parameter(index)?)?.returns
    }

    fn with_declared_bounds(&self, mut bindings: Bindings) -> Bindings {
        for (name, bound) in &self.bounds {
            bindings
                .entry(name.clone())
                .or_insert_with(|| bound.clone());
        }
        bindings
    }
}

fn invoked(
    types: &Types,
    ancestors: Vec<Ancestor>,
    method: &str,
    arguments: &[Option<ArgumentType>],
) -> Option<Invocation> {
    let candidates = candidates(&ancestors, method, arguments.len());
    if candidates.is_empty() {
        return None;
    }
    let signatures: Vec<_> = candidates.iter().map(Candidate::signature).collect();
    match select(types, &signatures, arguments) {
        Selection::Chosen {
            index,
            bindings,
            parameters,
        } => Some(candidates[index].invocation(bindings, parameters)),
        Selection::Undecided => Some(Invocation {
            parameters: Vec::new(),
            bindings: Bindings::new(),
            returned: agreed_return(&candidates),
            free: Vec::new(),
            bounds: Bindings::new(),
        }),
        Selection::Unresolvable => None,
    }
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

    fn invocation(&self, bindings: Bindings, parameters: Vec<TypeRef>) -> Invocation {
        Invocation {
            parameters,
            bindings,
            returned: self.returned(),
            free: self.method.type_parameters().to_vec(),
            bounds: self.declared_bounds(),
        }
    }

    fn declared_bounds(&self) -> Bindings {
        self.method
            .type_parameters()
            .iter()
            .filter_map(|name| {
                Some((
                    name.clone(),
                    self.method.type_parameter_bound(name)?.clone(),
                ))
            })
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
