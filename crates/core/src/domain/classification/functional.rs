use std::collections::HashMap;

use super::{Types, class_name, is_object_method, substitute};
use crate::domain::type_shape::{TypeKind, TypeRef};

pub(crate) struct FunctionalMethod {
    pub(crate) parameters: Vec<TypeRef>,
}

impl Types {
    pub(crate) fn functional_method(&self, type_ref: &TypeRef) -> Option<FunctionalMethod> {
        let is_interface = class_name(type_ref)
            .and_then(|name| self.shape(name))
            .is_some_and(|shape| shape.kind() == TypeKind::Interface);
        if !is_interface {
            return None;
        }
        let mut declared: HashMap<(&str, usize), Option<FunctionalMethod>> = HashMap::new();
        let ancestors = self.ancestors(type_ref);
        for ancestor in &ancestors {
            let Some((shape, bindings)) = self.bound(ancestor) else {
                continue;
            };
            for method in shape
                .methods()
                .iter()
                .filter(|method| !method.is_static() && !is_object_method(method))
            {
                declared
                    .entry((method.name(), method.parameters().len()))
                    .or_insert_with(|| {
                        method.is_abstract().then(|| FunctionalMethod {
                            parameters: method
                                .parameters()
                                .iter()
                                .map(|parameter| substitute(parameter, &bindings))
                                .collect(),
                        })
                    });
            }
        }
        let mut abstracts = declared.into_values().flatten();
        let only = abstracts.next()?;
        abstracts.next().is_none().then_some(only)
    }
}
