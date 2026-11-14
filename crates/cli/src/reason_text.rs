use law_of_demeter_core::domain::classification::Classification;
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
        Reason::This => "f'nin kendi nesnesi".to_owned(),
        Reason::Field(name) => format!("{name} sınıfın alanı"),
        Reason::FieldOf(name) => {
            format!("başka bir nesnenin {name} alanı f'nin arkadaşı değil")
        }
        Reason::Parameter(name) => format!("{name} f'ye argüman olarak verildi"),
        Reason::Created(type_ref) => {
            format!("f'nin içinde oluşturulan nesne ({})", type_name(type_ref))
        }
        Reason::StaticType(type_ref) => {
            format!("{} tipi üzerinde statik çağrı", type_name(type_ref))
        }
        Reason::Literal => "sabit değer".to_owned(),
        Reason::UnassignedLocal(name) => format!("{name} değişkenine değer atanmamış"),
        Reason::Classified {
            classification,
            type_ref,
        } => format!(
            "{} {}",
            type_name(type_ref),
            classification_text(*classification)
        ),
        Reason::Fluent(type_ref) => {
            format!("{} üzerinde akıcı (fluent) çağrı", type_name(type_ref))
        }
        Reason::StaticResult(call) => {
            format!("{call}() statik çağrısının döndürdüğü nesne f'nin arkadaşı")
        }
        Reason::FactoryResult(call) => {
            format!("{call}() factory çağrısının döndürdüğü nesne f'nin arkadaşı")
        }
        Reason::QueryResult(call) => {
            format!("{call}() depo sorgusunun döndürdüğü nesne f'nin arkadaşı")
        }
    }
}

pub fn category(reason: &Reason) -> &'static str {
    match reason {
        Reason::Local { origin, .. } => category(origin),
        Reason::LambdaParameter { feed, .. } | Reason::LoopVariable { feed, .. } => category(feed),
        Reason::ReturnOf(_) => "Döndürülen nesne f'nin arkadaşı değil",
        Reason::ContainerMutation { .. } => "Döndürülen koleksiyon değiştiriliyor",
        Reason::UnresolvedType { .. } => "Tip çözülemedi",
        Reason::UnassignedLocal(_) => "Değişkene değer atanmamış",
        Reason::This => "Kendi nesnesi",
        Reason::Field(_) => "Sınıfın alanı",
        Reason::FieldOf(_) => "Başka nesnenin alanı",
        Reason::Parameter(_) => "Argüman",
        Reason::Created(_) => "Metotta oluşturulan nesne",
        Reason::StaticType(_) => "Statik çağrı",
        Reason::Literal => "Sabit değer",
        Reason::Classified { classification, .. } => match classification {
            Classification::DataStructure => "Veri yapısı",
            Classification::Value => "Değer",
            Classification::Container => "Konteyner",
            Classification::Object => "Nesne",
            Classification::Hybrid => "Hibrit sınıf",
            Classification::Unknown => "Tip çözülemedi",
        },
        Reason::Fluent(_) => "Akıcı çağrı",
        Reason::StaticResult(_) => "Statik çağrının sonucu",
        Reason::FactoryResult(_) => "Factory'nin sonucu",
        Reason::QueryResult(_) => "Depo sorgusunun sonucu",
    }
}

fn classification_text(classification: Classification) -> &'static str {
    match classification {
        Classification::DataStructure => "bir veri yapısı",
        Classification::Object => "bir nesne",
        Classification::Hybrid => "bir hibrit sınıf",
        Classification::Value => "bir değer",
        Classification::Container => "bir konteyner",
        Classification::Unknown => "çözülemeyen bir tip",
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
