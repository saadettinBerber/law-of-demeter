use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeRef, TypeShape};

use crate::platform::CONSTRUCTOR_NAME;
use crate::{self as model, ImplicitTarget, TypeName};

use super::MethodMapper;
use super::inference::{Bindings, apply_bindings, bind_results, complete};
use super::lambdas::functional_method;
use super::lineage::{
    Ancestor, arguments_of, class_name, erased, has_variable, lineage, substitute,
};
use super::links::Declared;
use super::memo::address;
use super::overloads::{ArgumentType, Selection, Signature, select};
use super::types::named_type;

impl MethodMapper<'_> {
    fn call_type(&mut self, call: &model::Call) -> Option<TypeRef> {
        self.returned_by(call, None)
    }

    pub(super) fn returned_by(
        &mut self,
        call: &model::Call,
        expected: Option<&TypeRef>,
    ) -> Option<TypeRef> {
        self.open_returned_by(call, expected)
            .filter(|returned| !has_variable(returned))
    }

    pub(super) fn open_returned_by(
        &mut self,
        call: &model::Call,
        expected: Option<&TypeRef>,
    ) -> Option<TypeRef> {
        let key = (address(call), expected.cloned());
        if let Some(known) = self.memo.calls.get(&key) {
            return known.clone();
        }
        let types = self.types;
        let returned = self
            .resolve(call, expected)
            .and_then(|resolved| resolved.invocation.returned_open(types, expected));
        self.memo.calls.insert(key, returned.clone());
        returned
    }

    pub(super) fn invocation_of(&mut self, call: &model::Call) -> Option<Invocation> {
        let key = address(call);
        if let Some(known) = self.memo.invocations.get(&key) {
            return known.clone();
        }
        let found = self.find_invocation(call);
        self.memo.invocations.insert(key, found.clone());
        found
    }

    fn find_invocation(&mut self, call: &model::Call) -> Option<Invocation> {
        let receiver = self.receiver_type(call)?;
        let arguments = self.argument_types(&call.arguments);
        invoked(
            self.types,
            lineage(self.types, &receiver),
            (&call.name, Dispatch::Any),
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
        let shape = own.first()?.shape;
        let invocation = invoked(
            self.types,
            own,
            (CONSTRUCTOR_NAME, Dispatch::Any),
            &arguments,
        )?;
        Some(if arguments_of(&receiver).is_empty() {
            invocation.creating(shape)
        } else {
            invocation
        })
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
            model::Expression::MethodReference { .. } => Some(ArgumentType::Reference),
            model::Expression::Call(call) => self.call_argument(call),
            other => self
                .type_of(other)
                .map(|type_ref| ArgumentType::of_type(self.types, type_ref)),
        }
    }

    fn call_argument(&mut self, call: &model::Call) -> Option<ArgumentType> {
        match self.call_type(call) {
            Some(known) => Some(ArgumentType::of_type(self.types, known)),
            None => self.open_return_of(call).map(ArgumentType::Open),
        }
    }

    pub(super) fn is_open(&mut self, call: &model::Call) -> bool {
        self.call_type(call).is_none() && self.open_return_of(call).is_some()
    }

    fn open_return_of(&mut self, call: &model::Call) -> Option<TypeRef> {
        self.invocation_of(call)?.open_return()
    }

    pub(super) fn receiver_type(&mut self, call: &model::Call) -> Option<TypeRef> {
        match (&call.receiver, &call.implicit_target) {
            (Some(receiver), _) => self.type_of(receiver),
            (None, Some(ImplicitTarget::OuterMethod { owner })) => Some(TypeRef::class(owner)),
            (None, Some(ImplicitTarget::OwnMethod) | None) => Some(self.own_type()),
            (None, Some(_)) => self.static_owner(call).or_else(|| Some(self.own_type())),
        }
    }

    pub(super) fn static_owner(&self, call: &model::Call) -> Option<TypeRef> {
        match &call.implicit_target {
            Some(ImplicitTarget::StaticImport { owner }) => Some(named_type(owner)),
            Some(ImplicitTarget::OnDemandStaticImport { candidates })
                if !self.declares(&self.own_type(), call, Dispatch::Any) =>
            {
                self.on_demand_owner(candidates, call)
            }
            _ => None,
        }
    }

    fn own_type(&self) -> TypeRef {
        TypeRef::class(&self.class.name)
    }

    fn on_demand_owner(&self, candidates: &[TypeName], call: &model::Call) -> Option<TypeRef> {
        let mut owners = candidates
            .iter()
            .map(named_type)
            .filter(|owner| self.declares(owner, call, Dispatch::Static));
        let owner = owners.next()?;
        owners.next().is_none().then_some(owner)
    }

    fn declares(&self, owner: &TypeRef, call: &model::Call, dispatch: Dispatch) -> bool {
        let ancestors = lineage(self.types, owner);
        let declared = candidates(&ancestors, (&call.name, dispatch), call.arguments.len());
        !declared.is_empty()
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
            model::Expression::Call(call) => self.call_type(call),
            model::Expression::New(created) => self.type_ref(&created.created),
            model::Expression::Cast { target, .. } => self.type_ref(target),
            model::Expression::ClassLiteral(named) => Some(self.class_literal_type(named)),
            model::Expression::FieldAccess { target, name } => self.field_access_type(target, name),
            model::Expression::StaticImport { owner, name } => self.static_field_type(owner, name),
            model::Expression::OnDemandStaticImport { name, candidates } => {
                self.on_demand_field_type(candidates, name)
            }
            _ => None,
        }
    }
}

#[derive(Clone)]
pub(super) struct Invocation {
    parameters: Vec<TypeRef>,
    bindings: Bindings,
    returned: Option<TypeRef>,
    free: Vec<String>,
    bounds: Bindings,
    results: Vec<(TypeRef, TypeRef)>,
    declared: Option<Declared>,
}

impl Invocation {
    pub(super) fn declared(&self) -> Option<&Declared> {
        self.declared.as_ref()
    }

    pub(super) fn parameter(&self, index: usize) -> Option<&TypeRef> {
        self.parameters.get(index)
    }

    pub(super) fn expecting(self, types: &Types, expected: Option<&TypeRef>) -> Self {
        let completed = self
            .returned
            .as_ref()
            .zip(expected)
            .and_then(|(returned, expected)| {
                complete(types, &self.free, returned, &self.bindings, expected)
            });
        match completed {
            Some(completed) => Self {
                parameters: self
                    .parameters
                    .iter()
                    .map(|parameter| apply_bindings(parameter, &completed))
                    .collect(),
                ..self
            },
            None => self,
        }
    }

    fn creating(self, class: &TypeShape) -> Self {
        let variables: Vec<_> = class
            .type_parameters()
            .iter()
            .map(|name| TypeRef::variable(name))
            .collect();
        if variables.is_empty() {
            return self;
        }
        Self {
            free: [self.free, class.type_parameters().to_vec()].concat(),
            returned: Some(TypeRef::generic(class.name(), variables)),
            ..self
        }
    }

    pub(super) fn open_return(&self) -> Option<TypeRef> {
        let open = apply_bindings(self.returned.as_ref()?, &self.bindings);
        has_variable(&open).then_some(open)
    }

    pub(super) fn returns_to(&self, types: &Types) -> Option<TypeRef> {
        self.returned(types, None)
    }

    pub(super) fn awaits_result(&self, types: &Types, index: usize) -> bool {
        self.functional_returns(types, index)
            .is_some_and(|returned| has_variable(&returned))
    }

    pub(super) fn takes_known_inputs(&self, types: &Types, index: usize) -> bool {
        self.parameter(index)
            .and_then(|parameter| functional_method(types, parameter))
            .is_some_and(|functional| !functional.parameters.iter().any(has_variable))
    }

    pub(super) fn binding(mut self, types: &Types, pattern: TypeRef, result: TypeRef) -> Self {
        self.results.push((pattern, result));
        let bound = self.bound_by(types);
        self.parameters = self
            .parameters
            .iter()
            .map(|parameter| apply_bindings(parameter, &bound))
            .collect();
        self
    }

    pub(super) fn returned(&self, types: &Types, expected: Option<&TypeRef>) -> Option<TypeRef> {
        self.returned_open(types, expected)
            .filter(|returned| !has_variable(returned))
    }

    pub(super) fn returned_open(
        &self,
        types: &Types,
        expected: Option<&TypeRef>,
    ) -> Option<TypeRef> {
        let returned = self.returned.as_ref()?;
        let bound = self.bound_by(types);
        let completed = match expected {
            Some(expected) => complete(types, &self.free, returned, &bound, expected)?,
            None => bound,
        };
        Some(apply_bindings(
            returned,
            &self.with_declared_bounds(completed),
        ))
    }

    fn bound_by(&self, types: &Types) -> Bindings {
        bind_results(types, &self.free, &self.bindings, &self.results)
    }

    pub(super) fn functional_returns(&self, types: &Types, index: usize) -> Option<TypeRef> {
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

#[derive(Clone, Copy)]
pub(super) enum Dispatch {
    Any,
    Static,
    Instance,
}

impl Dispatch {
    fn admits(self, method: &MethodShape) -> bool {
        match self {
            Self::Any => true,
            Self::Static => method.is_static(),
            Self::Instance => !method.is_static(),
        }
    }
}

pub(super) fn invoked(
    types: &Types,
    ancestors: Vec<Ancestor>,
    (method, dispatch): (&str, Dispatch),
    arguments: &[Option<ArgumentType>],
) -> Option<Invocation> {
    let candidates = candidates(&ancestors, (method, dispatch), arguments.len());
    if candidates.is_empty() {
        return None;
    }
    let signatures: Vec<_> = candidates.iter().map(Candidate::signature).collect();
    match select(types, &signatures, arguments) {
        Selection::Chosen {
            index,
            bindings,
            parameters,
        } => Some(candidates[index].invocation(types, (bindings, parameters), arguments.len())),
        Selection::Ambiguous(indices) => {
            let contenders: Vec<_> = indices.iter().map(|index| &candidates[*index]).collect();
            Some(Invocation {
                parameters: Vec::new(),
                bindings: Bindings::new(),
                returned: agreed_return(&contenders),
                free: Vec::new(),
                bounds: Bindings::new(),
                results: Vec::new(),
                declared: None,
            })
        }
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

    fn invocation(
        &self,
        types: &Types,
        (bindings, parameters): (Bindings, Vec<TypeRef>),
        argument_count: usize,
    ) -> Invocation {
        Invocation {
            parameters,
            bindings,
            returned: self.returned(),
            free: self.method.type_parameters().to_vec(),
            bounds: self.declared_bounds(),
            results: Vec::new(),
            declared: Some(Declared::of(
                types,
                self.method,
                self.ancestor.shape,
                argument_count,
            )),
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
        Some(self.specialize(self.method.returns()?))
    }

    fn specialize(&self, type_ref: &TypeRef) -> TypeRef {
        if self.is_raw_member() {
            return erased(type_ref, &|name| self.declared_bound(name));
        }
        let own = self.method.type_parameters();
        let shadowed = |name: &str| own.iter().any(|variable| variable == name);
        substitute(type_ref, &|name| {
            self.ancestor.bound(name).filter(|_| !shadowed(name))
        })
    }

    fn is_raw_member(&self) -> bool {
        self.ancestor.erased && !self.method.is_static() && self.method.name() != CONSTRUCTOR_NAME
    }

    fn declared_bound(&self, name: &str) -> Option<TypeRef> {
        let own = self
            .method
            .type_parameters()
            .iter()
            .any(|variable| variable == name);
        let bound = if own {
            self.method.type_parameter_bound(name)
        } else {
            self.ancestor.shape.type_parameter_bound(name)
        };
        bound.cloned()
    }
}

fn candidates<'a>(
    ancestors: &'a [Ancestor<'a>],
    (method, dispatch): (&str, Dispatch),
    argument_count: usize,
) -> Vec<Candidate<'a>> {
    let mut candidates: Vec<Candidate> = Vec::new();
    for ancestor in ancestors {
        for declared in named(ancestor.shape, method)
            .into_iter()
            .filter(|declared| dispatch.admits(declared))
        {
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

fn agreed_return(candidates: &[&Candidate]) -> Option<TypeRef> {
    let (first, rest) = candidates.split_first()?;
    let returned = first.returned()?;
    let agreed = rest
        .iter()
        .all(|candidate| candidate.returned().as_ref() == Some(&returned));
    agreed.then_some(returned)
}
