use std::collections::HashMap;

use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::type_shape::{TypeRef, TypeShape};

pub(super) const OBJECT: &str = "java.lang.Object";
pub(super) const ARRAY_SUPERTYPES: [&str; 3] =
    [OBJECT, "java.lang.Cloneable", "java.io.Serializable"];

pub(super) fn lineage<'t>(types: &'t Types, receiver: &TypeRef) -> Vec<Ancestor<'t>> {
    let supertypes = closure(types, receiver);
    let mut ancestors: Vec<_> = supertypes
        .iter()
        .filter_map(|supertype| Ancestor::of(types, supertype))
        .collect();
    if !supertypes
        .iter()
        .any(|known| class_name(known) == Some(OBJECT))
    {
        ancestors.extend(Ancestor::of(types, &TypeRef::class(OBJECT)));
    }
    ancestors
}

pub(super) fn closure(types: &Types, receiver: &TypeRef) -> Vec<TypeRef> {
    let mut queue = vec![receiver.clone()];
    let mut index = 0;
    while index < queue.len() {
        if let Some(ancestor) = Ancestor::of(types, &queue[index]) {
            let unseen: Vec<_> = ancestor
                .supertypes()
                .into_iter()
                .filter(|supertype| !queue.iter().any(|known| same_class(known, supertype)))
                .collect();
            queue.extend(unseen);
        }
        index += 1;
    }
    queue
}

pub(super) struct Ancestor<'t> {
    pub(super) shape: &'t TypeShape,
    bindings: HashMap<String, TypeRef>,
    pub(super) erased: bool,
}

impl<'t> Ancestor<'t> {
    fn of(types: &'t Types, type_ref: &TypeRef) -> Option<Self> {
        let shape = types.shape(class_name(type_ref)?)?;
        let arguments = arguments_of(type_ref);
        let bindings = shape
            .type_parameters()
            .iter()
            .cloned()
            .zip(arguments.iter().cloned())
            .collect();
        let erased = arguments.is_empty() && !shape.type_parameters().is_empty();
        Some(Self {
            shape,
            bindings,
            erased,
        })
    }

    fn supertypes(&self) -> Vec<TypeRef> {
        self.shape
            .supertypes()
            .iter()
            .map(|supertype| {
                if self.erased {
                    erase(supertype)
                } else {
                    substitute(supertype, &|name| self.bound(name))
                }
            })
            .collect()
    }

    pub(super) fn bound(&self, name: &str) -> Option<TypeRef> {
        self.bindings.get(name).cloned()
    }
}

pub(super) fn substitute(type_ref: &TypeRef, binding: &dyn Fn(&str) -> Option<TypeRef>) -> TypeRef {
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

pub(super) fn has_variable(type_ref: &TypeRef) -> bool {
    match type_ref {
        TypeRef::Variable(_) => true,
        TypeRef::Array(element) => has_variable(element),
        TypeRef::Class { arguments, .. } => arguments.iter().any(has_variable),
        TypeRef::Primitive(_) => false,
    }
}

pub(super) fn class_name(type_ref: &TypeRef) -> Option<&str> {
    match type_ref {
        TypeRef::Class { name, .. } => Some(name),
        _ => None,
    }
}
