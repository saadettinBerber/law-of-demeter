use std::collections::HashMap;

use law_of_demeter_core::domain::type_shape::TypeRef;

use super::lambdas::Mapped;

#[derive(Default)]
pub(super) struct Memo {
    pub(super) calls: HashMap<usize, Option<TypeRef>>,
    pub(super) lambdas: HashMap<(usize, Option<TypeRef>), Mapped>,
}

pub(super) fn address<T>(node: &T) -> usize {
    std::ptr::from_ref(node) as usize
}
