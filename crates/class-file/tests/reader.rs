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
            "shop.Entity",
            "shop.Invoice",
            "shop.Order",
            "shop.Order$Receipt",
            "shop.Order$Tag",
            "shop.OrderLine",
            "shop.Product",
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

fn generated_method_names(shape: &TypeShape) -> Vec<&str> {
    shape
        .methods()
        .iter()
        .filter(|method| method.is_generated())
        .map(MethodShape::name)
        .collect()
}

#[test]
fn marks_methods_carrying_the_lombok_generated_annotation() {
    let account = shape("shop.Account");

    let generated = generated_method_names(&account);

    assert_eq!(
        generated,
        ["getBalance", "getOwner", "setBalance", "setOwner"]
    );
}

#[test]
fn does_not_mark_hand_written_methods_as_generated() {
    let customer = shape("shop.Customer");

    let generated = generated_method_names(&customer);

    assert!(generated.is_empty());
}

#[test]
fn a_method_that_only_returns_one_of_its_fields_reads_that_field() {
    let order = shape("shop.Order");
    let customer = shape("shop.Customer");

    let get_lines = method(&order, "getLines");
    let is_active = method(&customer, "isActive");

    assert_eq!(
        get_lines,
        MethodShape::new("getLines")
            .returning(TypeRef::generic(
                "java.util.List",
                vec![TypeRef::class("shop.OrderLine")]
            ))
            .reading("lines")
    );
    assert_eq!(
        is_active,
        MethodShape::new("isActive")
            .returning(TypeRef::primitive("boolean"))
            .reading("active")
    );
}

#[test]
fn a_method_that_only_assigns_its_parameter_to_a_field_writes_that_field() {
    let customer = shape("shop.Customer");

    let set_name = method(&customer, "setName");
    let set_active = method(&customer, "setActive");

    assert_eq!(
        set_name,
        MethodShape::new("setName")
            .with_parameters(vec![TypeRef::class("java.lang.String")])
            .writing("name")
    );
    assert_eq!(
        set_active,
        MethodShape::new("setActive")
            .with_parameters(vec![TypeRef::primitive("boolean")])
            .writing("active")
    );
}

#[test]
fn lombok_accessors_read_and_write_their_fields() {
    let account = shape("shop.Account");

    let get_balance = method(&account, "getBalance");
    let set_owner = method(&account, "setOwner");

    assert_eq!(get_balance.reads_field(), Some("balance"));
    assert_eq!(set_owner.writes_field(), Some("owner"));
}

#[test]
fn a_getter_that_computes_its_result_reads_no_field() {
    let invoice = shape("shop.Invoice");

    let get_total = method(&invoice, "getTotal");

    assert_eq!(get_total.reads_field(), None);
}

#[test]
fn a_method_that_updates_state_beyond_one_assignment_writes_no_field() {
    let account = shape("shop.Account");

    let withdraw = method(&account, "withdraw");

    assert_eq!(withdraw.writes_field(), None);
    assert_eq!(withdraw.reads_field(), None);
}

#[test]
fn returning_a_field_of_another_object_reads_no_field() {
    let order = shape("shop.Order");

    let customer_of = method(&order, "customerOf");

    assert_eq!(customer_of.reads_field(), None);
}

#[test]
fn returning_the_outer_instance_reads_no_field() {
    let receipt = shape("shop.Order$Receipt");

    let issuer = method(&receipt, "issuer");

    assert_eq!(issuer.reads_field(), None);
}

#[test]
fn returning_an_inherited_field_reads_no_field() {
    let product = shape("shop.Product");

    let identity = method(&product, "identity");
    let title = method(&product, "title");

    assert_eq!(identity.reads_field(), None);
    assert_eq!(title.reads_field(), Some("title"));
}

#[test]
fn reads_the_superclass_of_a_class() {
    let product = shape("shop.Product");

    let supertypes = product.supertypes();

    assert_eq!(supertypes, [TypeRef::class("shop.Entity")]);
}

#[test]
fn a_method_with_more_than_one_parameter_writes_no_field() {
    let product = shape("shop.Product");

    let retitle = method(&product, "retitle");

    assert_eq!(retitle.writes_field(), None);
}

#[test]
fn a_static_method_reading_a_field_of_its_argument_reads_no_field() {
    let product = shape("shop.Product");

    let title_of = method(&product, "titleOf");

    assert_eq!(title_of.reads_field(), None);
}

#[test]
fn a_method_with_a_parameter_that_returns_a_field_reads_no_field() {
    let product = shape("shop.Product");

    let title_at = method(&product, "titleAt");

    assert_eq!(title_at.reads_field(), None);
}

#[test]
fn a_method_assigning_something_other_than_its_parameter_writes_no_field() {
    let product = shape("shop.Product");

    let clear_title = method(&product, "clearTitle");

    assert_eq!(clear_title.writes_field(), None);
}

#[test]
fn reads_and_writes_fields_of_every_primitive_width() {
    let product = shape("shop.Product");

    let reads: Vec<Option<String>> = ["getStock", "getRating", "getPrice"]
        .map(|name| method(&product, name).reads_field().map(str::to_owned))
        .to_vec();
    let writes: Vec<Option<String>> = ["setStock", "setRating", "setPrice"]
        .map(|name| method(&product, name).writes_field().map(str::to_owned))
        .to_vec();

    assert_eq!(
        reads,
        [
            Some("stock".to_owned()),
            Some("rating".to_owned()),
            Some("price".to_owned())
        ]
    );
    assert_eq!(
        writes,
        [
            Some("stock".to_owned()),
            Some("rating".to_owned()),
            Some("price".to_owned())
        ]
    );
}
