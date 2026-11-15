use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use law_of_demeter_class_file::{Reading, read_types, read_types_lenient};
use law_of_demeter_core::domain::type_shape::{
    FieldShape, MethodShape, TypeKind, TypeRef, TypeShape,
};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn jar() -> PathBuf {
    fixtures().join("fixtures.jar")
}

fn scratch_directory(name: &str) -> PathBuf {
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).unwrap();
    directory
}

fn names(shapes: &[TypeShape]) -> Vec<&str> {
    shapes.iter().map(TypeShape::name).collect()
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
            "shop.Catalog",
            "shop.Catalog$Node",
            "shop.Container",
            "shop.Customer",
            "shop.Entity",
            "shop.Invoice",
            "shop.Money",
            "shop.Order",
            "shop.Order$Receipt",
            "shop.Order$Tag",
            "shop.OrderLine",
            "shop.Product",
            "shop.Registry",
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
            FieldShape::new("customer", TypeRef::class("shop.Customer")).as_final(),
            FieldShape::new(
                "lines",
                TypeRef::generic("java.util.List", vec![TypeRef::class("shop.OrderLine")])
            )
            .as_final(),
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
            .as_abstract()
            .with_parameters(vec![TypeRef::variable("ID")])
            .returning(TypeRef::variable("T"))
    );
    assert_eq!(
        find_all,
        MethodShape::new("findAll")
            .as_abstract()
            .returning(TypeRef::generic(
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

#[test]
fn reads_type_variables_wildcards_arrays_and_nested_generic_types_in_fields() {
    let catalog = shape("shop.Catalog");

    let fields = catalog.fields();

    assert_eq!(
        fields,
        [
            FieldShape::new(
                "index",
                TypeRef::generic(
                    "java.util.Map",
                    vec![
                        TypeRef::class("java.lang.String"),
                        TypeRef::generic("java.util.List", vec![TypeRef::variable("T")])
                    ]
                )
            ),
            FieldShape::new(
                "anything",
                TypeRef::generic("java.util.List", vec![TypeRef::class("java.lang.Object")])
            ),
            FieldShape::new("items", TypeRef::array(TypeRef::variable("T"))),
            FieldShape::new(
                "node",
                TypeRef::generic(
                    "shop.Catalog$Node",
                    vec![TypeRef::class("java.lang.Integer")]
                )
            ),
        ]
    );
}

#[test]
fn reads_generic_supertypes_of_a_class_with_bounded_type_parameters() {
    let catalog = shape("shop.Catalog");

    let supertypes = catalog.supertypes();

    assert_eq!(
        supertypes,
        [
            TypeRef::class("shop.Entity"),
            TypeRef::generic(
                "shop.Repository",
                vec![TypeRef::variable("T"), TypeRef::class("java.lang.Long")]
            )
        ]
    );
}

#[test]
fn reads_methods_with_their_own_type_parameters_and_leaves_out_bridges() {
    let catalog = shape("shop.Catalog");

    let convert = method(&catalog, "convert");

    assert_eq!(
        convert,
        MethodShape::new("convert")
            .with_type_parameter("R")
            .with_parameters(vec![TypeRef::generic(
                "java.util.function.Function",
                vec![TypeRef::variable("T"), TypeRef::variable("R")]
            )])
            .returning(TypeRef::variable("R"))
    );
    assert_eq!(
        method_names(&catalog),
        ["find", "findAll", "convert", "rethrow"]
    );
}

#[test]
fn a_method_type_parameter_keeps_its_name_without_the_bound() {
    let catalog = shape("shop.Catalog");

    let rethrow = method(&catalog, "rethrow");

    assert_eq!(rethrow.type_parameters(), ["X"]);
    assert_eq!(rethrow.parameters(), [TypeRef::variable("X")]);
}

#[test]
fn a_method_without_type_parameters_lists_none() {
    let catalog = shape("shop.Catalog");

    let find = method(&catalog, "find");

    assert!(find.type_parameters().is_empty());
}

#[test]
fn default_and_static_interface_methods_are_not_abstract() {
    let repository = shape("shop.Repository");

    let exists = method(&repository, "exists");
    let none = method(&repository, "none");

    assert!(!exists.is_abstract());
    assert!(!none.is_abstract());
    assert_eq!(none.type_parameters(), ["T"]);
}

#[test]
fn an_abstract_class_method_is_abstract_and_its_implementation_is_not() {
    let container = shape("shop.Container");
    let registry = shape("shop.Registry");

    let declared = method(&container, "keyOf");
    let implemented = method(&registry, "keyOf");

    assert!(declared.is_abstract());
    assert!(!implemented.is_abstract());
}

#[test]
fn a_method_ending_in_a_variable_arity_parameter_is_varargs() {
    let invoice = shape("shop.Invoice");

    let describe = method(&invoice, "describe");

    assert!(describe.is_varargs());
    assert_eq!(
        describe.parameters(),
        [
            TypeRef::class("java.lang.String"),
            TypeRef::array(TypeRef::class("java.lang.Object"))
        ]
    );
}

#[test]
fn a_method_with_a_plain_array_parameter_is_not_varargs() {
    let invoice = shape("shop.Invoice");

    let label = method(&invoice, "label");

    assert!(!label.is_varargs());
}

#[test]
fn reads_every_type_of_a_jar_file() {
    let shapes = read_types(&[jar()]).unwrap();

    let wallet = shapes.iter().find(|shape| shape.name() == "lib.Wallet");

    assert_eq!(names(&shapes), ["lib.Currency", "lib.Wallet"]);
    assert_eq!(
        wallet.map(|shape| method_names(shape)),
        Some(vec!["withdraw"])
    );
}

#[test]
fn reads_directories_and_jars_given_together() {
    let shapes = read_types(&[jar(), fixtures().join("classes")]).unwrap();

    let names = names(&shapes);

    assert!(names.contains(&"lib.Wallet"));
    assert!(names.contains(&"shop.Order"));
}

#[test]
fn ignores_multi_release_copies_under_meta_inf_in_a_jar() {
    let directory = scratch_directory("multi-release-jar");
    let multi_release = directory.join("multi-release.jar");
    write_jar(
        &multi_release,
        &[("META-INF/versions/9/app/Shifted.class", b"x")],
    );

    let shapes = read_types(&[&multi_release]).unwrap();

    assert!(shapes.is_empty());
}

#[test]
fn a_missing_path_is_an_error_carrying_that_path() {
    let missing = fixtures().join("does-not-exist");

    let error = read_types(&[&missing]).unwrap_err();

    assert_eq!(error.path(), missing);
}

#[test]
fn a_file_that_is_not_a_class_file_is_an_error_carrying_its_path() {
    let directory = scratch_directory("not-a-class");
    let broken = directory.join("Broken.class");
    fs::write(&broken, b"this is not a class file").unwrap();

    let error = read_types(&[&directory]).unwrap_err();

    assert_eq!(error.path(), broken);
}

#[test]
fn a_truncated_class_file_is_an_error_carrying_its_path() {
    let directory = scratch_directory("truncated-class");
    let truncated = directory.join("Order.class");
    let bytes = fs::read(fixtures().join("classes/shop/Order.class")).unwrap();
    fs::write(&truncated, &bytes[..bytes.len() / 2]).unwrap();

    let error = read_types(&[&truncated]).unwrap_err();

    assert_eq!(error.path(), truncated);
}

#[test]
fn a_jar_that_is_not_a_zip_is_an_error_carrying_its_path() {
    let directory = scratch_directory("not-a-zip");
    let broken = directory.join("broken.jar");
    fs::write(&broken, b"this is not a zip archive").unwrap();

    let error = read_types(&[&broken]).unwrap_err();

    assert_eq!(error.path(), broken);
}

#[test]
fn a_malformed_class_inside_a_jar_is_an_error_naming_the_jar_and_the_entry() {
    let directory = scratch_directory("bad-entry-jar");
    let broken = directory.join("broken.jar");
    write_jar(&broken, &[("app/Broken.class", b"not a class")]);

    let error = read_types(&[&broken]).unwrap_err();

    assert_eq!(error.path(), broken);
    assert!(error.to_string().contains("app/Broken.class"));
}

#[test]
fn an_unreadable_generic_signature_is_an_error_instead_of_a_skipped_type() {
    let directory = scratch_directory("bad-signature");
    let damaged = directory.join("Order.class");
    let signature = b"()Ljava/util/List<Lshop/OrderLine;>;";
    let mut bytes = fs::read(fixtures().join("classes/shop/Order.class")).unwrap();
    let start = bytes
        .windows(signature.len())
        .position(|window| window == signature)
        .unwrap();
    bytes[start + signature.len() - 3..start + signature.len()].copy_from_slice(b";;;");
    fs::write(&damaged, bytes).unwrap();

    let error = read_types(&[&damaged]).unwrap_err();

    assert_eq!(error.path(), damaged);
    assert!(error.to_string().contains("getLines"));
}

fn write_jar(path: &Path, entries: &[(&str, &[u8])]) {
    let mut writer = zip::ZipWriter::new(fs::File::create(path).unwrap());
    for (entry, content) in entries {
        writer
            .start_file(*entry, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(content).unwrap();
    }
    writer.finish().unwrap();
}

fn error_paths(reading: &Reading) -> Vec<&Path> {
    reading.errors.iter().map(|error| error.path()).collect()
}

fn customer_class() -> Vec<u8> {
    fs::read(fixtures().join("classes/shop/Customer.class")).unwrap()
}

#[test]
fn lenient_reading_keeps_readable_types_and_reports_each_unreadable_file_with_its_path() {
    let directory = scratch_directory("lenient-directory");
    fs::write(directory.join("Customer.class"), customer_class()).unwrap();
    fs::write(directory.join("Empty.class"), b"").unwrap();
    fs::write(directory.join("Broken.class"), b"not a class").unwrap();

    let reading = read_types_lenient(&[&directory]);

    assert_eq!(names(&reading.shapes), ["shop.Customer"]);
    assert_eq!(
        error_paths(&reading),
        [
            directory.join("Broken.class"),
            directory.join("Empty.class")
        ]
    );
}

#[test]
fn lenient_reading_continues_with_the_next_path_after_a_missing_one() {
    let missing = fixtures().join("does-not-exist");

    let reading = read_types_lenient(&[missing.clone(), jar()]);

    assert_eq!(names(&reading.shapes), ["lib.Currency", "lib.Wallet"]);
    assert_eq!(error_paths(&reading), [missing]);
}

#[test]
fn lenient_reading_keeps_the_good_entries_of_a_jar_with_a_bad_one() {
    let directory = scratch_directory("lenient-jar");
    let mixed = directory.join("mixed.jar");
    let customer = customer_class();
    write_jar(
        &mixed,
        &[
            ("app/Broken.class", b"not a class"),
            ("shop/Customer.class", &customer),
        ],
    );

    let reading = read_types_lenient(&[&mixed]);

    assert_eq!(names(&reading.shapes), ["shop.Customer"]);
    assert_eq!(error_paths(&reading), [mixed]);
    assert!(reading.errors[0].to_string().contains("app/Broken.class"));
}

#[test]
fn lenient_reading_reports_a_jar_that_is_not_a_zip_and_goes_on() {
    let directory = scratch_directory("lenient-not-a-zip");
    let broken = directory.join("broken.jar");
    fs::write(&broken, b"this is not a zip archive").unwrap();

    let reading = read_types_lenient(&[broken.clone(), jar()]);

    assert_eq!(names(&reading.shapes), ["lib.Currency", "lib.Wallet"]);
    assert_eq!(error_paths(&reading), [broken]);
}

#[test]
fn lenient_reading_reports_a_class_with_an_unreadable_signature_and_keeps_the_others() {
    let directory = scratch_directory("lenient-bad-signature");
    let signature = b"()Ljava/util/List<Lshop/OrderLine;>;";
    let mut damaged = fs::read(fixtures().join("classes/shop/Order.class")).unwrap();
    let start = damaged
        .windows(signature.len())
        .position(|window| window == signature)
        .unwrap();
    damaged[start + signature.len() - 3..start + signature.len()].copy_from_slice(b";;;");
    fs::write(directory.join("Order.class"), damaged).unwrap();
    fs::write(directory.join("Customer.class"), customer_class()).unwrap();

    let reading = read_types_lenient(&[&directory]);

    assert_eq!(names(&reading.shapes), ["shop.Customer"]);
    assert_eq!(error_paths(&reading), [directory.join("Order.class")]);
}

#[test]
fn lenient_reading_of_clean_input_matches_strict_reading() {
    let paths = [fixtures().join("classes"), jar()];

    let reading = read_types_lenient(&paths);

    assert!(reading.errors.is_empty());
    assert_eq!(reading.shapes, read_types(&paths).unwrap());
}

#[test]
fn module_descriptors_are_skipped_without_being_read() {
    let directory = scratch_directory("module-descriptors");
    fs::write(directory.join("module-info.class"), b"not a class").unwrap();
    let jar = directory.join("modular.jar");
    write_jar(&jar, &[("module-info.class", b"not a class")]);

    let reading = read_types_lenient(&[&directory, &jar]);

    assert!(reading.shapes.is_empty());
    assert!(reading.errors.is_empty(), "{:?}", reading.errors);
}

#[test]
fn reads_the_names_of_the_type_parameters_in_order() {
    let repository = shape("shop.Repository");
    let container = shape("shop.Container");
    let catalog = shape("shop.Catalog");
    let order = shape("shop.Order");

    assert_eq!(repository.type_parameters(), ["T", "ID"]);
    assert_eq!(container.type_parameters(), ["K", "V"]);
    assert_eq!(catalog.type_parameters(), ["T"]);
    assert!(order.type_parameters().is_empty());
}

#[test]
fn an_inner_class_lists_only_the_type_parameters_of_its_own_signature() {
    let node = shape("shop.Catalog$Node");

    let parameters = node.type_parameters();

    assert_eq!(parameters, ["U"]);
}

#[test]
fn reads_the_superclass_with_its_generic_arguments() {
    let registry = shape("shop.Registry");
    let product = shape("shop.Product");

    assert_eq!(
        registry.superclass(),
        Some(&TypeRef::generic(
            "shop.Container",
            vec![TypeRef::class("java.lang.String"), TypeRef::variable("V")]
        ))
    );
    assert_eq!(product.superclass(), Some(&TypeRef::class("shop.Entity")));
}

#[test]
fn the_superclass_of_enums_and_records_is_their_base_class() {
    let status = shape("shop.Status");
    let address = shape("shop.Address");

    assert_eq!(
        status.superclass(),
        Some(&TypeRef::generic(
            "java.lang.Enum",
            vec![TypeRef::class("shop.Status")]
        ))
    );
    assert_eq!(
        address.superclass(),
        Some(&TypeRef::class("java.lang.Record"))
    );
}

#[test]
fn a_type_extending_only_object_or_only_interfaces_has_no_superclass() {
    let customer = shape("shop.Customer");
    let order = shape("shop.Order");
    let repository = shape("shop.Repository");

    assert_eq!(customer.superclass(), None);
    assert_eq!(order.superclass(), None);
    assert_eq!(repository.superclass(), None);
}

#[test]
fn the_superclass_stays_first_among_the_supertypes() {
    let registry = shape("shop.Registry");

    let supertypes = registry.supertypes();

    assert_eq!(
        supertypes,
        [
            TypeRef::generic(
                "shop.Container",
                vec![TypeRef::class("java.lang.String"), TypeRef::variable("V")]
            ),
            TypeRef::generic(
                "shop.Repository",
                vec![TypeRef::variable("V"), TypeRef::class("java.lang.String")]
            ),
        ]
    );
}

fn legacy_and_current_box() -> (Vec<u8>, Vec<u8>) {
    let ct_sym = fs::File::open(fixtures().join("jdk/lib/ct.sym")).unwrap();
    let mut archive = zip::ZipArchive::new(ct_sym).unwrap();
    let mut read = |name: &str| {
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut archive.by_name(name).unwrap(), &mut bytes).unwrap();
        bytes
    };
    (
        read("8/sample.base/sample/util/Box.sig"),
        read("9AB/sample.base/sample/util/Box.sig"),
    )
}

#[test]
fn errors_of_a_directory_tree_come_in_sorted_path_order() {
    let root = scratch_directory("ordered-tree");
    let mut broken = Vec::new();
    for directory in ["c/y", "a/z", "b/x", "a/y", "b/w"] {
        fs::create_dir_all(root.join(directory)).unwrap();
        for index in 0..12 {
            let path = root.join(directory).join(format!("Broken{index:02}.class"));
            fs::write(&path, b"not a class").unwrap();
            broken.push(path);
            fs::write(
                root.join(directory).join(format!("Good{index:02}.class")),
                customer_class(),
            )
            .unwrap();
        }
    }
    broken.sort();

    let reading = read_types_lenient(&[&root]);

    assert_eq!(reading.shapes.len(), 60);
    assert_eq!(error_paths(&reading), broken);
}

#[test]
fn errors_follow_the_order_of_the_inputs_and_of_the_entries_inside_a_jar() {
    let directory = scratch_directory("ordered-inputs");
    let entries: Vec<String> = (0..40)
        .map(|index| format!("app/Bad{index:02}.class"))
        .collect();
    let jar_entries: Vec<(&str, &[u8])> = entries
        .iter()
        .map(|entry| (entry.as_str(), b"not a class".as_slice()))
        .collect();
    let first_jar = directory.join("first.jar");
    let second_jar = directory.join("second.jar");
    write_jar(&first_jar, &jar_entries);
    write_jar(&second_jar, &jar_entries);
    let missing = directory.join("missing");
    let classes = directory.join("classes");
    fs::create_dir_all(&classes).unwrap();
    fs::write(classes.join("Broken.class"), b"not a class").unwrap();

    let reading = read_types_lenient(&[&second_jar, &missing, &classes, &first_jar]);

    let mut expected: Vec<(PathBuf, Option<&str>)> = Vec::new();
    expected.extend(
        entries
            .iter()
            .map(|entry| (second_jar.clone(), Some(entry.as_str()))),
    );
    expected.push((missing, None));
    expected.push((classes.join("Broken.class"), None));
    expected.extend(
        entries
            .iter()
            .map(|entry| (first_jar.clone(), Some(entry.as_str()))),
    );
    assert_eq!(reading.errors.len(), expected.len());
    for (error, (path, entry)) in reading.errors.iter().zip(&expected) {
        assert_eq!(error.path(), path);
        if let Some(entry) = entry {
            assert!(error.to_string().contains(entry), "{error} lacks {entry}");
        }
    }
}

#[test]
fn equal_type_names_keep_the_order_of_the_inputs() {
    let directory = scratch_directory("same-name-inputs");
    let (legacy, current) = legacy_and_current_box();
    let jars: Vec<PathBuf> = (0..40)
        .map(|index| {
            let jar = directory.join(format!("box{index:02}.jar"));
            let class = if index % 2 == 0 { &current } else { &legacy };
            write_jar(&jar, &[("sample/util/Box.class", class)]);
            jar
        })
        .collect();

    let reading = read_types_lenient(&jars);

    let method_counts: Vec<usize> = reading
        .shapes
        .iter()
        .map(|shape| shape.methods().len())
        .collect();
    let expected: Vec<usize> = (0..40).map(|index| 2 - index % 2).collect();
    assert_eq!(method_counts, expected);
}

#[test]
fn an_immutable_value_has_only_final_fields() {
    let money = shape("shop.Money");

    let fields = money.fields();

    assert_eq!(
        fields,
        [
            FieldShape::new("ZERO", TypeRef::class("shop.Money"))
                .as_static()
                .as_final(),
            FieldShape::new("amount", TypeRef::primitive("long")).as_final(),
            FieldShape::new("currency", TypeRef::class("java.lang.String")).as_final(),
        ]
    );
}

#[test]
fn an_immutable_value_exposes_getters_and_value_methods() {
    let money = shape("shop.Money");

    let get_amount = method(&money, "getAmount");
    let get_currency = method(&money, "getCurrency");

    assert_eq!(
        method_names(&money),
        [
            "getAmount",
            "getCurrency",
            "equals",
            "hashCode",
            "compareTo"
        ]
    );
    assert_eq!(get_amount.reads_field(), Some("amount"));
    assert_eq!(get_currency.reads_field(), Some("currency"));
    assert_eq!(
        method(&money, "compareTo").parameters(),
        [TypeRef::class("shop.Money")]
    );
}

#[test]
fn fields_that_can_be_reassigned_are_not_final() {
    let customer = shape("shop.Customer");
    let order = shape("shop.Order");

    let customer_final: Vec<bool> = customer.fields().iter().map(FieldShape::is_final).collect();
    let history = order
        .fields()
        .iter()
        .find(|field| field.name() == "history");

    assert_eq!(customer_final, [false, false]);
    assert_eq!(history.map(FieldShape::is_final), Some(false));
}

#[test]
fn enum_constants_are_final_static_fields() {
    let status = shape("shop.Status");

    let fields = status.fields();

    assert!(
        fields
            .iter()
            .all(|field| field.is_static() && field.is_final())
    );
    assert_eq!(fields.len(), 2);
}

#[test]
fn record_components_are_final_fields() {
    let address = shape("shop.Address");

    let fields = address.fields();

    assert_eq!(fields.len(), 2);
    assert!(fields.iter().all(FieldShape::is_final));
}
