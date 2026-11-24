use std::path::Path;

use law_of_demeter_class_file::read_types;
use law_of_demeter_core::domain::classification::{Classification, Platform, Types};
use law_of_demeter_core::domain::method::{Call, Expression, Method, Reason, Verdict};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeRef};

#[test]
fn a_direct_field_getter_allows_a_call_on_the_returned_data_structure() {
    let inspect = inspect_returned_order();

    let types = read_fixture("direct");
    let verdicts = inspect.verdicts(&types);

    assert_eq!(
        method(&types, "shop.Order", "getLines"),
        MethodShape::new("getLines")
            .returning(TypeRef::generic(
                "java.util.List",
                vec![TypeRef::class("java.lang.String")]
            ))
            .reading("lines")
    );
    assert_eq!(
        types.classify(&TypeRef::class("shop.Order")),
        Classification::DataStructure
    );
    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getOrder", 2, Reason::parameter("customer")),
            Verdict::allowed(
                "getLines",
                3,
                Reason::classified(Classification::DataStructure, TypeRef::class("shop.Order"))
            )
        ]
    );
}

#[test]
fn a_wrapping_getter_is_behavior_and_rejects_a_call_on_the_returned_object() {
    let inspect = inspect_returned_order();

    let types = read_fixture("wrapped");
    let verdicts = inspect.verdicts(&types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getOrder", 2, Reason::parameter("customer")),
            Verdict::violation("getLines", 3, Reason::return_of("getOrder"))
        ]
    );
    assert_eq!(
        method(&types, "shop.Order", "getLines"),
        MethodShape::new("getLines").returning(TypeRef::generic(
            "java.util.List",
            vec![TypeRef::class("java.lang.String")]
        ))
    );
    assert_eq!(
        types.classify(&TypeRef::class("shop.Order")),
        Classification::Object
    );
}

#[test]
fn a_computing_getter_is_behavior_and_rejects_a_call_on_the_returned_object() {
    let inspect = Method::new("shop.OrderService", "fullName", 1).call(Call::new(
        Expression::return_of(
            Call::new(
                Expression::parameter("order", TypeRef::class("shop.Order")),
                "getCustomer",
                2,
            )
            .returning(TypeRef::class("shop.Customer")),
        ),
        "getFullName",
        3,
    ));

    let types = read_fixture("computed");
    let verdicts = inspect.verdicts(&types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 2, Reason::parameter("order")),
            Verdict::violation("getFullName", 3, Reason::return_of("getCustomer"))
        ]
    );
    assert_eq!(
        method(&types, "shop.Customer", "getFullName"),
        MethodShape::new("getFullName").returning(TypeRef::class("java.lang.String"))
    );
    assert_eq!(
        types.classify(&TypeRef::class("shop.Customer")),
        Classification::Object
    );
}

fn inspect_returned_order() -> Method {
    Method::new("shop.OrderService", "lines", 1).call(Call::new(
        Expression::return_of(
            Call::new(
                Expression::parameter("customer", TypeRef::class("shop.Customer")),
                "getOrder",
                2,
            )
            .returning(TypeRef::class("shop.Order")),
        ),
        "getLines",
        3,
    ))
}

fn read_fixture(name: &str) -> Types {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/accessors")
        .join(name)
        .join("classes");
    let shapes = read_types(&[directory]).expect("accessor fixture can be read");
    let types = Types::new()
        .with_platform(Platform::new().with_constructor_name("<init>"))
        .with_project_types(shapes.iter().map(|shape| shape.name().to_owned()));
    shapes.into_iter().fold(types, Types::with_shape)
}

fn method(types: &Types, class: &str, name: &str) -> MethodShape {
    types
        .shape(class)
        .expect("fixture class is present")
        .methods()
        .iter()
        .find(|method| method.name() == name)
        .expect("fixture getter is present")
        .clone()
}
