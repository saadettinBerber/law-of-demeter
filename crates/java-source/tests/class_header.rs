use law_of_demeter_java_source::{DeclaredType, TypeName, TypeParameter, parse};

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

fn type_parameter(name: &str, bound: Option<DeclaredType>) -> TypeParameter {
    TypeParameter {
        name: name.to_owned(),
        bound,
    }
}

#[test]
fn class_type_parameters_keep_their_names_and_first_bounds() {
    let source = "class Box<T extends Order & Customer, U, V extends Comparable<V>> {}\nrecord Pair<A extends Order>(A first) {}\ninterface Shape<S extends Order> {}";

    let classes = parse("A.java", source).unwrap().classes;

    assert_eq!(
        classes[0].type_parameters,
        [
            type_parameter("T", Some(DeclaredType::unresolved("Order"))),
            type_parameter("U", None),
            type_parameter(
                "V",
                Some(DeclaredType::Class {
                    name: TypeName::Unresolved("Comparable".to_owned()),
                    arguments: vec![DeclaredType::variable("V")]
                })
            ),
        ]
    );
    assert_eq!(
        classes[1].type_parameters,
        [type_parameter("A", Some(DeclaredType::unresolved("Order")))]
    );
    assert_eq!(
        classes[2].type_parameters,
        [type_parameter("S", Some(DeclaredType::unresolved("Order")))]
    );
}

#[test]
fn a_class_without_type_parameters_has_none() {
    let class = parse("A.java", "class Plain {}").unwrap().classes.remove(0);

    assert_eq!(class.type_parameters, []);
}
