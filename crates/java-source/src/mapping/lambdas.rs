use std::mem;

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::Lambda;
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef};

use crate::{self as model};

use super::MethodMapper;
use super::lineage::{class_name, has_variable, lineage, substitute};

const OBJECT_METHODS: [(&str, usize); 3] = [("equals", 1), ("hashCode", 0), ("toString", 0)];

pub(super) struct Functional {
    pub(super) parameters: Vec<TypeRef>,
}

impl MethodMapper<'_> {
    pub(super) fn lambda(&mut self, lambda: &model::Lambda, target: Option<&TypeRef>) -> Lambda {
        let names: Vec<&str> = lambda
            .parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect();
        let (result, body) = match lambda.body.split_last() {
            Some((model::Statement::Return(Some(result)), earlier)) => (Some(result), earlier),
            _ => (None, lambda.body.as_slice()),
        };
        let outer_steps = mem::take(&mut self.steps);
        let outer_returns = self.returns.take();
        self.statements(body);
        let result = result.map(|result| self.expression(result));
        self.returns = outer_returns;
        let mapped = mem::replace(&mut self.steps, outer_steps).into_lambda(Lambda::new(&names));
        let resolved = self.resolved_parameters(lambda, target);
        let mapped = lambda
            .parameters
            .iter()
            .zip(resolved)
            .fold(mapped, |mapped, (parameter, resolved)| {
                self.typed(mapped, parameter, resolved)
            });
        match result {
            Some(value) => mapped.yielding(value),
            None => mapped,
        }
    }

    fn typed(
        &self,
        mapped: Lambda,
        parameter: &model::LambdaParameter,
        resolved: Option<TypeRef>,
    ) -> Lambda {
        let declared = parameter
            .declared_type
            .as_ref()
            .and_then(|declared| self.type_ref(declared));
        match (declared, resolved) {
            (Some(declared), _) => mapped.declaring(&parameter.name, declared),
            (None, Some(resolved)) => mapped.resolving(&parameter.name, resolved),
            (None, None) => mapped,
        }
    }

    fn resolved_parameters(
        &self,
        lambda: &model::Lambda,
        target: Option<&TypeRef>,
    ) -> Vec<Option<TypeRef>> {
        let count = lambda.parameters.len();
        match target.and_then(|target| functional_method(self.types, target)) {
            Some(functional) if functional.parameters.len() == count => functional
                .parameters
                .into_iter()
                .map(|parameter| Some(parameter).filter(|known| !has_variable(known)))
                .collect(),
            _ => vec![None; count],
        }
    }
}

pub(super) fn functional_method(types: &Types, target: &TypeRef) -> Option<Functional> {
    let shape = types.shape(class_name(target)?)?;
    if shape.kind() != TypeKind::Interface {
        return None;
    }
    let mut declared: Vec<((&str, usize), Option<Functional>)> = Vec::new();
    for ancestor in &lineage(types, target) {
        for method in ancestor
            .shape
            .methods()
            .iter()
            .filter(|m| !is_object_method(m))
        {
            let key = (method.name(), method.parameters().len());
            if declared.iter().any(|(known, _)| *known == key) {
                continue;
            }
            let functional = (method.is_abstract() && !method.is_static()).then(|| Functional {
                parameters: method
                    .parameters()
                    .iter()
                    .map(|parameter| substitute(parameter, &|name| ancestor.bound(name)))
                    .collect(),
            });
            declared.push((key, functional));
        }
    }
    let mut abstracts = declared
        .into_iter()
        .filter_map(|(_, functional)| functional);
    let only = abstracts.next()?;
    abstracts.next().is_none().then_some(only)
}

fn is_object_method(method: &MethodShape) -> bool {
    OBJECT_METHODS.contains(&(method.name(), method.parameters().len()))
}
