use law_of_demeter_core::domain::classification::Classification;
use law_of_demeter_core::domain::method::Reason;
use law_of_demeter_core::domain::type_shape::TypeRef;

pub fn describe(reason: &Reason) -> String {
    match reason {
        Reason::ReturnOf(call) => {
            format!("the object returned by {call}() is not a friend of f")
        }
        Reason::Local { name, origin } => {
            format!("local variable {name}: {}", describe(origin))
        }
        Reason::LambdaParameter { name, feed } => {
            format!("lambda parameter {name}: {}", describe(feed))
        }
        Reason::LoopVariable { name, feed } => {
            format!("loop variable {name}: {}", describe(feed))
        }
        Reason::MethodReference { owner, name, feed } => {
            format!(
                "method reference {}::{name}: {}",
                simple_name(owner),
                describe(feed)
            )
        }
        Reason::ContainerMutation { type_ref, source } => format!(
            "collection modified from outside ({}): {}",
            type_name(type_ref),
            describe(source)
        ),
        Reason::UnresolvedType { type_ref, source } => match type_ref {
            Some(type_ref) => format!(
                "type not resolved ({}): {}",
                type_name(type_ref),
                describe(source)
            ),
            None => format!("type not resolved: {}", describe(source)),
        },
        Reason::This => "f's own object".to_owned(),
        Reason::Field(name) => format!("{name} is a field of the class"),
        Reason::FieldOf(name) => {
            format!("field {name} of another object is not a friend of f")
        }
        Reason::Parameter(name) => format!("{name} is an argument of f"),
        Reason::Created(type_ref) => {
            format!("object created in f ({})", type_name(type_ref))
        }
        Reason::StaticType(type_ref) => {
            format!("static call on type {}", type_name(type_ref))
        }
        Reason::Literal => "literal value".to_owned(),
        Reason::FunctionArgument => "object passed to a function as an argument".to_owned(),
        Reason::UnassignedLocal(name) => format!("variable {name} has no assigned value"),
        Reason::UnresolvedName(name) => format!("name {name} could not be resolved"),
        Reason::Classified {
            classification,
            type_ref,
        } => format!(
            "{} is {}",
            type_name(type_ref),
            classification_text(*classification)
        ),
        Reason::Library(type_ref) => format!("library type ({})", type_name(type_ref)),
        Reason::Fluent(type_ref) => {
            format!("fluent call on {}", type_name(type_ref))
        }
        Reason::StaticResult(call) => {
            format!("the object returned by static call {call}() is a friend of f")
        }
        Reason::StaticField(name) => format!("static constant {name} is a friend of f"),
        Reason::FactoryResult(call) => {
            format!("the object returned by factory call {call}() is a friend of f")
        }
        Reason::QueryResult(call) => {
            format!("the object returned by repository query {call}() is a friend of f")
        }
    }
}

pub fn category(reason: &Reason) -> &'static str {
    match reason {
        Reason::Local { origin, .. } => category(origin),
        Reason::LambdaParameter { feed, .. }
        | Reason::LoopVariable { feed, .. }
        | Reason::MethodReference { feed, .. } => category(feed),
        Reason::ReturnOf(_) => "Returned object is not a friend of f",
        Reason::ContainerMutation { .. } => "Returned collection is modified",
        Reason::UnresolvedType { .. } => "Type not resolved",
        Reason::UnassignedLocal(_) => "Variable has no value",
        Reason::UnresolvedName(_) => "Name not resolved",
        Reason::This => "Own object",
        Reason::Field(_) => "Field of the class",
        Reason::FieldOf(_) => "Field of another object",
        Reason::Parameter(_) => "Argument",
        Reason::Created(_) => "Object created in the method",
        Reason::StaticType(_) => "Static call",
        Reason::Literal => "Literal",
        Reason::FunctionArgument => "Function argument",
        Reason::Classified { classification, .. } => match classification {
            Classification::DataStructure => "Data structure",
            Classification::Value => "Value",
            Classification::Container => "Container",
            Classification::Object => "Object",
            Classification::Hybrid => "Hybrid class",
            Classification::Unknown => "Type not resolved",
        },
        Reason::Library(_) => "Library type",
        Reason::Fluent(_) => "Fluent call",
        Reason::StaticResult(_) => "Result of a static call",
        Reason::StaticField(_) => "Static constant",
        Reason::FactoryResult(_) => "Result of a factory",
        Reason::QueryResult(_) => "Result of a repository query",
    }
}

fn classification_text(classification: Classification) -> &'static str {
    match classification {
        Classification::DataStructure => "a data structure",
        Classification::Object => "an object",
        Classification::Hybrid => "a hybrid class",
        Classification::Value => "a value",
        Classification::Container => "a container",
        Classification::Unknown => "an unresolved type",
    }
}

fn simple_name(type_ref: &TypeRef) -> String {
    match type_ref {
        TypeRef::Class { name, .. } => name.rsplit('.').next().unwrap_or(name).to_owned(),
        other => type_name(other),
    }
}

fn type_name(type_ref: &TypeRef) -> String {
    match type_ref {
        TypeRef::Class { name, arguments } if arguments.is_empty() => name.clone(),
        TypeRef::Class { name, arguments } => {
            let arguments: Vec<String> = arguments.iter().map(type_name).collect();
            format!("{name}<{}>", arguments.join(", "))
        }
        TypeRef::Primitive(name) | TypeRef::Variable(name) => name.clone(),
        TypeRef::Array(element) => format!("{}[]", type_name(element)),
    }
}
