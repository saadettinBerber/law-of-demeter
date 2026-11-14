use super::{Classification, Types};
use crate::domain::type_shape::{FieldShape, MethodShape, TypeKind, TypeRef, TypeShape};

#[test]
fn a_record_is_a_data_structure() {
    let types = Types::new().with_shape(TypeShape::new("Server", TypeKind::Record));

    let classification = types.classify(&TypeRef::class("Server"));

    assert_eq!(classification, Classification::DataStructure);
}

fn string() -> TypeRef {
    TypeRef::class("java.lang.String")
}

fn classify(shape: TypeShape) -> Classification {
    let name = shape.name().to_owned();
    Types::new()
        .with_shape(shape)
        .classify(&TypeRef::class(&name))
}

#[test]
fn a_class_with_only_getters_and_setters_is_a_data_structure() {
    let address = TypeShape::new("Address", TypeKind::Class)
        .with_method(
            MethodShape::new("getCity")
                .returning(string())
                .reading("city"),
        )
        .with_method(
            MethodShape::new("setCity")
                .with_parameters(vec![string()])
                .writing("city"),
        );

    let classification = classify(address);

    assert_eq!(classification, Classification::DataStructure);
}

fn int() -> TypeRef {
    TypeRef::primitive("int")
}

#[test]
fn a_class_with_behavior_and_no_setters_is_an_object() {
    let order = TypeShape::new("Order", TypeKind::Class)
        .with_method(MethodShape::new("getId").returning(string()).reading("id"))
        .with_method(MethodShape::new("charge").with_parameters(vec![int()]));

    let classification = classify(order);

    assert_eq!(classification, Classification::Object);
}

#[test]
fn a_getter_that_computes_instead_of_reading_a_field_is_behavior() {
    let invoice = TypeShape::new("Invoice", TypeKind::Class)
        .with_field(FieldShape::new("total", int()))
        .with_method(MethodShape::new("getTotal").returning(int()));

    let classification = classify(invoice);

    assert_eq!(classification, Classification::Object);
}

#[test]
fn a_class_with_behavior_and_a_setter_is_a_hybrid() {
    let account = TypeShape::new("Account", TypeKind::Class)
        .with_method(
            MethodShape::new("getBalance")
                .returning(int())
                .reading("balance"),
        )
        .with_method(
            MethodShape::new("setBalance")
                .with_parameters(vec![int()])
                .writing("balance"),
        )
        .with_method(MethodShape::new("withdraw").with_parameters(vec![int()]));

    let classification = classify(account);

    assert_eq!(classification, Classification::Hybrid);
}

#[test]
fn generated_and_hand_written_accessors_are_classified_the_same() {
    let generated = TypeShape::new("Account", TypeKind::Class)
        .with_method(
            MethodShape::new("getBalance")
                .returning(int())
                .reading("balance")
                .generated(),
        )
        .with_method(
            MethodShape::new("setBalance")
                .with_parameters(vec![int()])
                .writing("balance")
                .generated(),
        )
        .with_method(MethodShape::new("withdraw").with_parameters(vec![int()]));
    let hand_written = TypeShape::new("Account", TypeKind::Class)
        .with_method(
            MethodShape::new("getBalance")
                .returning(int())
                .reading("balance"),
        )
        .with_method(
            MethodShape::new("setBalance")
                .with_parameters(vec![int()])
                .writing("balance"),
        )
        .with_method(MethodShape::new("withdraw").with_parameters(vec![int()]));

    let classifications = (classify(generated), classify(hand_written));

    assert_eq!(
        classifications,
        (Classification::Hybrid, Classification::Hybrid)
    );
}

#[test]
fn constructors_object_methods_statics_and_non_public_methods_are_not_behavior() {
    let dto = TypeShape::new("Customer", TypeKind::Class)
        .with_method(
            MethodShape::new("getName")
                .returning(string())
                .reading("name"),
        )
        .with_method(MethodShape::new("<init>"))
        .with_method(MethodShape::new("toString").returning(string()).generated())
        .with_method(MethodShape::new("hashCode").returning(int()).generated())
        .with_method(
            MethodShape::new("equals")
                .with_parameters(vec![TypeRef::class("java.lang.Object")])
                .generated(),
        )
        .with_method(
            MethodShape::new("builder")
                .returning(TypeRef::class("Customer$Builder"))
                .as_static(),
        )
        .with_method(MethodShape::new("validate").not_public());

    let classification = classify(dto);

    assert_eq!(classification, Classification::DataStructure);
}

#[test]
fn an_overload_of_an_object_method_with_other_parameters_is_behavior() {
    let customer = TypeShape::new("Customer", TypeKind::Class)
        .with_method(MethodShape::new("toString").with_parameters(vec![int()]));

    let classification = classify(customer);

    assert_eq!(classification, Classification::Object);
}

#[test]
fn an_enum_is_a_value() {
    let status = TypeShape::new("Status", TypeKind::Enum)
        .with_method(MethodShape::new("isFinal").returning(TypeRef::primitive("boolean")));

    let classification = classify(status);

    assert_eq!(classification, Classification::Value);
}

#[test]
fn an_interface_is_an_object() {
    let wallet = TypeShape::new("Wallet", TypeKind::Interface)
        .with_method(MethodShape::new("getBalance").returning(int()));

    let classification = classify(wallet);

    assert_eq!(classification, Classification::Object);
}

#[test]
fn a_data_structure_and_the_object_it_carries_are_told_apart() {
    let types = Types::new()
        .with_shape(
            TypeShape::new("OrderShippedEvent", TypeKind::Class)
                .with_field(FieldShape::new("order", TypeRef::class("Order")))
                .with_method(
                    MethodShape::new("getOrder")
                        .returning(TypeRef::class("Order"))
                        .reading("order"),
                ),
        )
        .with_shape(
            TypeShape::new("Order", TypeKind::Class)
                .with_method(MethodShape::new("getTotal").returning(int())),
        );

    let event = types.classify(&TypeRef::class("OrderShippedEvent"));
    let carried = types.classify(&TypeRef::class("Order"));

    assert_eq!(
        (event, carried),
        (Classification::DataStructure, Classification::Object)
    );
}
