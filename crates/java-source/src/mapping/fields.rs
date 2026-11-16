use law_of_demeter_core::domain::type_shape::TypeRef;

use crate::{self as model, TypeName};

use super::MethodMapper;
use super::lineage::{lineage, substitute};
use super::types::named_type;

impl MethodMapper<'_> {
    pub(super) fn field_access_type(
        &mut self,
        target: &model::Expression,
        name: &str,
    ) -> Option<TypeRef> {
        let owner = self.type_of(target)?;
        self.declared_field(&owner, name, false)
    }

    pub(super) fn static_field_type(&self, owner: &TypeName, name: &str) -> Option<TypeRef> {
        self.declared_field(&named_type(owner), name, false)
    }

    pub(super) fn on_demand_field_type(
        &self,
        candidates: &[TypeName],
        name: &str,
    ) -> Option<TypeRef> {
        candidates
            .iter()
            .find_map(|owner| self.declared_field(&named_type(owner), name, true))
    }

    fn declared_field(&self, owner: &TypeRef, name: &str, only_static: bool) -> Option<TypeRef> {
        lineage(self.types, owner).into_iter().find_map(|ancestor| {
            let field = ancestor
                .shape
                .fields()
                .iter()
                .find(|field| field.name() == name && (!only_static || field.is_static()))?;
            Some(substitute(field.type_ref(), &|variable| {
                ancestor.bound(variable)
            }))
        })
    }
}
