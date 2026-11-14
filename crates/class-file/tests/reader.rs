use std::path::PathBuf;

use law_of_demeter_class_file::read_types;
use law_of_demeter_core::domain::type_shape::{
    FieldShape, MethodShape, TypeKind, TypeRef, TypeShape,
};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn read_shop() -> Vec<TypeShape> {
    read_types(&[fixtures().join("classes")]).unwrap()
}

fn shape(name: &str) -> TypeShape {
    read_shop()
        .into_iter()
        .find(|shape| shape.name() == name)
        .unwrap_or_else(|| panic!("no type named {name}"))
}

fn method(shape: &TypeShape, name: &str) -> MethodShape {
    shape
        .methods()
        .iter()
        .find(|method| method.name() == name)
        .cloned()
        .unwrap_or_else(|| panic!("no method named {name}"))
}

fn method_names(shape: &TypeShape) -> Vec<&str> {
    shape.methods().iter().map(MethodShape::name).collect()
}

#[test]
fn reads_every_type_of_a_class_directory_by_binary_name() {
    let shapes = read_shop();

    let names: Vec<&str> = shapes.iter().map(TypeShape::name).collect();

    assert_eq!(
        names,
        [
            "shop.Account",
            "shop.Address",
            "shop.Customer",
            "shop.Invoice",
            "shop.Order",
            "shop.Order$Receipt",
            "shop.Order$Tag",
            "shop.OrderLine",
            "shop.Repository",
            "shop.Status",
        ]
    );
}

#[test]
fn tells_classes_records_enums_and_interfaces_apart() {
    let kinds: Vec<TypeKind> = [
        "shop.Order",
        "shop.Address",
        "shop.Status",
        "shop.Repository",
    ]
    .map(|name| shape(name).kind())
    .to_vec();

    assert_eq!(
        kinds,
        [
            TypeKind::Class,
            TypeKind::Record,
            TypeKind::Enum,
            TypeKind::Interface
        ]
    );
}

#[test]
fn reads_implemented_interfaces_with_their_generic_arguments() {
    let order = shape("shop.Order");

    let supertypes = order.supertypes();

    assert_eq!(
        supertypes,
        [TypeRef::generic(
            "java.lang.Comparable",
            vec![TypeRef::class("shop.Order")]
        )]
    );
}

#[test]
fn reads_the_superclass_of_enums_and_records() {
    let status = shape("shop.Status");
    let address = shape("shop.Address");

    assert_eq!(
        status.supertypes(),
        [TypeRef::generic(
            "java.lang.Enum",
            vec![TypeRef::class("shop.Status")]
        )]
    );
    assert_eq!(address.supertypes(), [TypeRef::class("java.lang.Record")]);
}

#[test]
fn leaves_out_object_as_a_supertype() {
    let customer = shape("shop.Customer");
    let repository = shape("shop.Repository");

    assert!(customer.supertypes().is_empty());
    assert!(repository.supertypes().is_empty());
}

#[test]
fn reads_field_types_with_generics_arrays_and_the_static_flag() {
    let order = shape("shop.Order");

    let fields = order.fields();

    assert_eq!(
        fields,
        [
            FieldShape::new("created", TypeRef::primitive("int")).as_static(),
            FieldShape::new("customer", TypeRef::class("shop.Customer")),
            FieldShape::new(
                "lines",
                TypeRef::generic("java.util.List", vec![TypeRef::class("shop.OrderLine")])
            ),
            FieldShape::new("history", TypeRef::array(TypeRef::primitive("int"))),
        ]
    );
}

#[test]
fn leaves_out_synthetic_fields() {
    let status = shape("shop.Status");
    let receipt = shape("shop.Order$Receipt");

    let status_fields: Vec<&str> = status.fields().iter().map(FieldShape::name).collect();

    assert_eq!(status_fields, ["OPEN", "PAID"]);
    assert!(receipt.fields().is_empty());
}

#[test]
fn reads_static_methods_with_parameter_and_return_types() {
    let order = shape("shop.Order");

    let of = method(&order, "of");

    assert_eq!(
        of,
        MethodShape::new("of")
            .as_static()
            .with_parameters(vec![TypeRef::class("shop.Customer")])
            .returning(TypeRef::class("shop.Order"))
    );
}

#[test]
fn reads_void_methods_and_non_public_methods() {
    let order = shape("shop.Order");

    let add_line = method(&order, "addLine");
    let audit = method(&order, "audit");

    assert_eq!(
        add_line,
        MethodShape::new("addLine").with_parameters(vec![TypeRef::class("shop.OrderLine")])
    );
    assert_eq!(audit, MethodShape::new("audit").not_public());
}

#[test]
fn reads_generic_signatures_of_interface_methods() {
    let repository = shape("shop.Repository");

    let find = method(&repository, "find");
    let find_all = method(&repository, "findAll");

    assert_eq!(
        find,
        MethodShape::new("find")
            .with_parameters(vec![TypeRef::variable("ID")])
            .returning(TypeRef::variable("T"))
    );
    assert_eq!(
        find_all,
        MethodShape::new("findAll").returning(TypeRef::generic(
            "java.util.List",
            vec![TypeRef::variable("T")]
        ))
    );
}

#[test]
fn leaves_out_constructors_initializers_bridges_and_synthetic_methods() {
    let order = shape("shop.Order");
    let status = shape("shop.Status");

    assert_eq!(
        method_names(&order),
        [
            "of",
            "getCustomer",
            "getLines",
            "customerOf",
            "total",
            "addLine",
            "audit",
            "compareTo"
        ]
    );
    assert_eq!(method_names(&status), ["values", "valueOf"]);
}
