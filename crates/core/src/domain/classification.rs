mod functional;
mod invocation;
mod jdk;
mod naming;

pub use naming::{is_factory_method, is_repository_query};

use std::collections::{HashMap, HashSet};

pub(crate) use functional::FunctionalMethod;
pub(crate) use invocation::{Invocation, has_variable};

use super::type_shape::{FieldShape, MethodShape, TypeKind, TypeRef, TypeShape};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classification {
    DataStructure,
    Object,
    Hybrid,
    Value,
    Container,
    Unknown,
}

#[derive(Debug, Clone, Default)]
pub struct Types {
    shapes: HashMap<String, TypeShape>,
    project_types: HashSet<String>,
}

impl Types {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_shape(mut self, shape: TypeShape) -> Self {
        self.shapes.insert(shape.name().to_owned(), shape);
        self
    }

    pub fn shape(&self, name: &str) -> Option<&TypeShape> {
        self.shapes.get(name)
    }

    pub fn with_project_types(mut self, names: impl IntoIterator<Item = String>) -> Self {
        self.project_types.extend(names);
        self
    }

    pub fn is_project_type(&self, name: &str) -> bool {
        self.project_types.contains(name)
    }

    pub fn return_type(
        &self,
        type_name: &str,
        method: &str,
        argument_count: usize,
    ) -> Option<&TypeRef> {
        let overloads = self.overloads(type_name, method, argument_count);
        unambiguous_return(&overloads)
    }

    pub fn return_type_on(
        &self,
        receiver: &TypeRef,
        method: &str,
        argument_count: usize,
    ) -> Option<TypeRef> {
        self.invocation(receiver, method, &vec![None; argument_count])
            .returns()
    }

    pub(crate) fn invocation(
        &self,
        receiver: &TypeRef,
        method: &str,
        values: &[Option<&TypeRef>],
    ) -> Invocation {
        let Some((declaring, overloads)) = self.overloads_on(receiver, method, values.len()) else {
            return Invocation::unknown();
        };
        let class_bindings = self
            .bound(&declaring)
            .map(|(_, bindings)| bindings)
            .unwrap_or_default();
        let overloads = invocation::applicable(self, overloads, &class_bindings, values);
        match overloads.as_slice() {
            [only] => Invocation::of(self, only, &class_bindings, values),
            _ => Invocation::ambiguous(
                unambiguous_return(&overloads)
                    .map(|returned| substitute(returned, &class_bindings)),
            ),
        }
    }

    fn overloads(&self, type_name: &str, method: &str, argument_count: usize) -> Vec<&MethodShape> {
        self.overloads_on(&TypeRef::class(type_name), method, argument_count)
            .map(|(_, overloads)| overloads)
            .unwrap_or_default()
    }

    fn overloads_on(
        &self,
        receiver: &TypeRef,
        method: &str,
        argument_count: usize,
    ) -> Option<(TypeRef, Vec<&MethodShape>)> {
        self.declaring_overloads(receiver, method, |candidate| {
            candidate.parameters().len() == argument_count
        })
        .or_else(|| {
            self.declaring_overloads(receiver, method, |candidate| {
                candidate.is_varargs() && candidate.parameters().len() <= argument_count + 1
            })
        })
    }

    fn declaring_overloads(
        &self,
        receiver: &TypeRef,
        method: &str,
        takes: impl Fn(&MethodShape) -> bool,
    ) -> Option<(TypeRef, Vec<&MethodShape>)> {
        let roots = self.root_types(receiver);
        self.ancestors(receiver)
            .into_iter()
            .chain(roots)
            .find_map(|ancestor| {
                let shape = self.shape(class_name(&ancestor)?)?;
                let declared: Vec<_> = shape
                    .methods()
                    .iter()
                    .filter(|candidate| candidate.name() == method && takes(candidate))
                    .collect();
                (!declared.is_empty()).then_some((ancestor, declared))
            })
    }

    fn root_types(&self, receiver: &TypeRef) -> Vec<TypeRef> {
        let Some(name) = class_name(receiver) else {
            return Vec::new();
        };
        let specific = match self.shape(name).map(TypeShape::kind) {
            Some(TypeKind::Enum) => {
                Some(TypeRef::generic("java.lang.Enum", vec![receiver.clone()]))
            }
            Some(TypeKind::Record) => Some(TypeRef::class("java.lang.Record")),
            _ => None,
        };
        specific
            .into_iter()
            .chain([TypeRef::class("java.lang.Object")])
            .collect()
    }

    fn ancestors(&self, receiver: &TypeRef) -> Vec<TypeRef> {
        let mut found = vec![receiver.clone()];
        let mut index = 0;
        while let Some(current) = found.get(index) {
            index += 1;
            let Some((shape, bindings)) = self.bound(current) else {
                continue;
            };
            let unseen: Vec<_> = shape
                .supertypes()
                .iter()
                .map(|supertype| substitute(supertype, &bindings))
                .filter(|supertype| {
                    !found
                        .iter()
                        .any(|known| class_name(known) == class_name(supertype))
                })
                .collect();
            found.extend(unseen);
        }
        found
    }

    fn bound(&self, type_ref: &TypeRef) -> Option<(&TypeShape, HashMap<String, TypeRef>)> {
        let TypeRef::Class { name, arguments } = type_ref else {
            return None;
        };
        let shape = self.shape(name)?;
        let bindings = if shape.type_parameters().len() == arguments.len() {
            shape
                .type_parameters()
                .iter()
                .cloned()
                .zip(arguments.iter().cloned())
                .collect()
        } else {
            HashMap::new()
        };
        Some((shape, bindings))
    }

    pub fn element_type_of(&self, type_ref: &TypeRef) -> Option<TypeRef> {
        let container = self.ancestors(type_ref).into_iter().find(|ancestor| {
            matches!(ancestor, TypeRef::Array(_))
                || class_name(ancestor).and_then(jdk::classify) == Some(Classification::Container)
        })?;
        element_type(&container).cloned()
    }

    pub fn is_fluent(&self, type_name: &str, method: &str, argument_count: usize) -> bool {
        let overloads = self.overloads(type_name, method, argument_count);
        !overloads.is_empty()
            && overloads.iter().all(|overload| {
                !overload.is_static() && overload.returns().and_then(class_name) == Some(type_name)
            })
    }

    pub fn hybrid_class(&self, name: &str) -> Option<HybridClass> {
        let shape = self.shape(name)?;
        (self.classify_named(name) == Classification::Hybrid).then(|| {
            let methods = self.class_members(shape).project_methods;
            HybridClass::new(
                name,
                distinct_names(methods.iter().filter(|method| is_behavior(method))),
                distinct_names(methods.iter().filter(|method| is_setter(method))),
            )
        })
    }

    pub fn classify(&self, type_ref: &TypeRef) -> Classification {
        match type_ref {
            TypeRef::Class { name, .. } => self.classify_named(name),
            TypeRef::Primitive(_) => Classification::Value,
            TypeRef::Array(_) => Classification::Container,
            TypeRef::Variable(_) => Classification::Unknown,
        }
    }

    fn classify_named(&self, name: &str) -> Classification {
        jdk::classify(name).unwrap_or_else(|| {
            self.shapes
                .get(name)
                .map_or(Classification::Unknown, |shape| self.classify_shape(shape))
        })
    }

    fn classify_shape(&self, shape: &TypeShape) -> Classification {
        self.inherited_value_or_container(shape)
            .unwrap_or_else(|| match shape.kind() {
                TypeKind::Record => Classification::DataStructure,
                TypeKind::Class => self.classify_class(shape),
                TypeKind::Enum => Classification::Value,
                TypeKind::Interface => Classification::Object,
            })
    }

    fn class_members<'a>(&'a self, shape: &'a TypeShape) -> Members<'a> {
        let mut chain = vec![shape];
        let mut complete = true;
        let mut index = 0;
        while let Some(&current) = chain.get(index) {
            index += 1;
            let Some(name) = current.superclass().and_then(class_name) else {
                continue;
            };
            if jdk::is_behaviorless(name) {
                continue;
            }
            match self.shape(name) {
                Some(superclass) => {
                    if !chain.iter().any(|known| known.name() == superclass.name()) {
                        chain.push(superclass);
                    }
                }
                None => complete &= jdk::classify(name).is_some(),
            }
        }
        let mut methods: Vec<&MethodShape> = Vec::new();
        let mut project_methods: Vec<&MethodShape> = Vec::new();
        let mut hidden: HashSet<(&str, usize)> = HashSet::new();
        for (index, class) in chain.iter().enumerate() {
            let declared: Vec<_> = class
                .methods()
                .iter()
                .filter(|method| is_exposed(method) && !is_to_builder(class, method))
                .collect();
            let visible: Vec<_> = declared
                .iter()
                .filter(|method| !hidden.contains(&signature(method)))
                .copied()
                .collect();
            hidden.extend(declared.iter().map(|method| signature(method)));
            if index == 0 || self.is_project_type(class.name()) {
                project_methods.extend(&visible);
            }
            methods.extend(visible);
        }
        Members {
            chain,
            methods,
            project_methods,
            complete,
        }
    }

    fn classify_class(&self, shape: &TypeShape) -> Classification {
        let members = self.class_members(shape);
        if members.is_value_object() {
            return Classification::Value;
        }
        let has_behavior = members.methods.iter().any(|method| is_behavior(method));
        match (members.is_hybrid(), has_behavior, members.complete) {
            (true, _, _) => Classification::Hybrid,
            (false, true, _) => Classification::Object,
            (false, false, true) => Classification::DataStructure,
            (false, false, false) => Classification::Unknown,
        }
    }

    fn inherited_value_or_container(&self, shape: &TypeShape) -> Option<Classification> {
        self.ancestors(&TypeRef::class(shape.name()))
            .iter()
            .skip(1)
            .filter_map(class_name)
            .filter_map(jdk::classify)
            .find(|classification| {
                matches!(
                    classification,
                    Classification::Value | Classification::Container
                )
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HybridClass {
    class: String,
    behavior_methods: Vec<String>,
    setters: Vec<String>,
}

impl HybridClass {
    fn new(class: &str, behavior_methods: Vec<String>, setters: Vec<String>) -> Self {
        Self {
            class: class.to_owned(),
            behavior_methods,
            setters,
        }
    }

    pub fn class(&self) -> &str {
        &self.class
    }

    pub fn behavior_methods(&self) -> &[String] {
        &self.behavior_methods
    }

    pub fn setters(&self) -> &[String] {
        &self.setters
    }
}

struct Members<'a> {
    chain: Vec<&'a TypeShape>,
    methods: Vec<&'a MethodShape>,
    project_methods: Vec<&'a MethodShape>,
    complete: bool,
}

impl Members<'_> {
    fn is_hybrid(&self) -> bool {
        self.project_methods
            .iter()
            .any(|method| is_behavior(method))
            && self.project_methods.iter().any(|method| is_setter(method))
    }

    fn is_value_object(&self) -> bool {
        let mut fields = self
            .chain
            .iter()
            .flat_map(|class| class.fields())
            .filter(|field| !field.is_static())
            .peekable();
        let has_fields = fields.peek().is_some();
        self.complete
            && has_fields
            && fields.all(FieldShape::is_final)
            && !self.methods.iter().any(|method| is_setter(method))
            && self.declares("equals", 1)
            && self.declares("hashCode", 0)
    }

    fn declares(&self, name: &str, parameter_count: usize) -> bool {
        self.chain
            .iter()
            .flat_map(|class| class.methods())
            .any(|method| {
                method.name() == name
                    && method.parameters().len() == parameter_count
                    && method.is_public()
                    && !method.is_static()
            })
    }
}

pub fn element_type(type_ref: &TypeRef) -> Option<&TypeRef> {
    match type_ref {
        TypeRef::Array(element) => Some(element),
        TypeRef::Class { name, arguments }
            if jdk::classify(name) == Some(Classification::Container) =>
        {
            arguments.last()
        }
        TypeRef::Class { .. } | TypeRef::Primitive(_) | TypeRef::Variable(_) => None,
    }
}

fn unambiguous_return<'a>(overloads: &[&'a MethodShape]) -> Option<&'a TypeRef> {
    let (first, rest) = overloads.split_first()?;
    rest.iter()
        .all(|overload| overload.returns() == first.returns())
        .then(|| first.returns())
        .flatten()
}

fn substitute(type_ref: &TypeRef, bindings: &HashMap<String, TypeRef>) -> TypeRef {
    match type_ref {
        TypeRef::Variable(name) => bindings
            .get(name)
            .cloned()
            .unwrap_or_else(|| type_ref.clone()),
        TypeRef::Class { name, arguments } => TypeRef::Class {
            name: name.clone(),
            arguments: arguments
                .iter()
                .map(|argument| substitute(argument, bindings))
                .collect(),
        },
        TypeRef::Array(element) => TypeRef::Array(Box::new(substitute(element, bindings))),
        TypeRef::Primitive(_) => type_ref.clone(),
    }
}

fn class_name(type_ref: &TypeRef) -> Option<&str> {
    match type_ref {
        TypeRef::Class { name, .. } => Some(name),
        TypeRef::Primitive(_) | TypeRef::Array(_) | TypeRef::Variable(_) => None,
    }
}

fn is_object_method(method: &MethodShape) -> bool {
    matches!(
        (method.name(), method.parameters().len()),
        ("equals" | "canEqual", 1) | ("hashCode" | "toString", 0)
    )
}

fn is_exposed(method: &MethodShape) -> bool {
    !method.is_private()
        && !method.is_static()
        && !is_object_method(method)
        && method.name() != "<init>"
}

fn is_to_builder(class: &TypeShape, method: &MethodShape) -> bool {
    method.name() == "toBuilder"
        && method.parameters().is_empty()
        && method
            .returns()
            .and_then(class_name)
            .is_some_and(|name| name.starts_with(&format!("{}$", class.name())))
}

fn signature(method: &MethodShape) -> (&str, usize) {
    (method.name(), method.parameters().len())
}

fn is_behavior(method: &MethodShape) -> bool {
    !is_getter(method) && !is_setter(method)
}

fn distinct_names<'a>(methods: impl Iterator<Item = &'a &'a MethodShape>) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for method in methods {
        if !names.iter().any(|name| name == method.name()) {
            names.push(method.name().to_owned());
        }
    }
    names
}

fn is_getter(method: &MethodShape) -> bool {
    method.reads_field().is_some() && method.parameters().is_empty()
}

fn is_setter(method: &MethodShape) -> bool {
    method.writes_field().is_some() && method.parameters().len() == 1
}

#[cfg(test)]
mod tests;
