use law_of_demeter_core::domain::method::Reason;
use law_of_demeter_core::domain::type_shape::TypeRef;

pub fn describe(reason: &Reason) -> String {
    match reason {
        Reason::ReturnOf(call) => {
            format!("{call}() çağrısının döndürdüğü nesne f'nin arkadaşı değil")
        }
        Reason::Local { name, origin } => {
            format!("{name} yerel değişkeni: {}", describe(origin))
        }
        Reason::LambdaParameter { name, feed } => {
            format!("{name} lambda parametresi: {}", describe(feed))
        }
        Reason::LoopVariable { name, feed } => {
            format!("{name} döngü değişkeni: {}", describe(feed))
        }
        Reason::ContainerMutation { type_ref, source } => format!(
            "koleksiyon dışarıdan değiştiriliyor ({}): {}",
            type_name(type_ref),
            describe(source)
        ),
        Reason::UnresolvedType { type_ref, source } => match type_ref {
            Some(type_ref) => format!(
                "tip çözülemedi ({}): {}",
                type_name(type_ref),
                describe(source)
            ),
            None => format!("tip çözülemedi: {}", describe(source)),
        },
        _ => String::new(),
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
