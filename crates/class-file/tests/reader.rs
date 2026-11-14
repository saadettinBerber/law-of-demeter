use std::path::PathBuf;

use law_of_demeter_class_file::read_types;
use law_of_demeter_core::domain::type_shape::{TypeKind, TypeShape};

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
