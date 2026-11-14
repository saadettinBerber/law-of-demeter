use law_of_demeter_java_source::{DeclaredType, TypeName, parse};

#[test]
fn supertypes_are_extended_classes_and_implemented_interfaces() {
    let source = "package shop;\n\
        import java.io.Serializable;\n\
        class Order<T> extends Base<T> implements Serializable, Comparable<Order<T>> {}\n\
        interface Shape extends Drawable {}\n\
        enum Color implements Named { RED }";

    let classes = parse("A.java", source).unwrap().classes;

    assert_eq!(
        classes[0].supertypes,
        [
            DeclaredType::Class {
                name: TypeName::Unresolved("Base".to_owned()),
                arguments: vec![DeclaredType::variable("T")]
            },
            DeclaredType::class("java.io.Serializable"),
            DeclaredType::Class {
                name: TypeName::Unresolved("Comparable".to_owned()),
                arguments: vec![DeclaredType::Class {
                    name: TypeName::Resolved("shop.Order".to_owned()),
                    arguments: vec![DeclaredType::variable("T")]
                }]
            },
        ]
    );
    assert_eq!(
        classes[1].supertypes,
        [DeclaredType::unresolved("Drawable")]
    );
    assert_eq!(classes[2].supertypes, [DeclaredType::unresolved("Named")]);
}

#[test]
fn class_annotations_are_resolved_type_names() {
    let source = "import lombok.Getter;\n\
        @Getter\n\
        @Setter(value = 1)\n\
        class Account {}";

    let class = parse("A.java", source).unwrap().classes.remove(0);

    assert_eq!(
        class.annotations,
        [
            TypeName::Resolved("lombok.Getter".to_owned()),
            TypeName::Unresolved("Setter".to_owned()),
        ]
    );
}
