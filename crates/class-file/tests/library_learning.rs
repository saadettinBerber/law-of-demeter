use std::fs::{self, File};
use std::io::Read;

use cafebabe::attributes::AttributeData;
use cafebabe::bytecode::Opcode;
use cafebabe::descriptors::FieldType;
use cafebabe::{ClassFile, MethodInfo, parse_class};
use zip::ZipArchive;

fn fixtures() -> String {
    format!("{}/tests/fixtures", env!("CARGO_MANIFEST_DIR"))
}

fn class_bytes(name: &str) -> Vec<u8> {
    fs::read(format!("{}/classes/{name}.class", fixtures())).unwrap()
}

fn method<'a, 'b>(class: &'b ClassFile<'a>, name: &str) -> &'b MethodInfo<'a> {
    class.methods.iter().find(|m| m.name == name).unwrap()
}

#[test]
fn cafebabe_decodes_a_getter_into_this_load_field_read_and_return() {
    let bytes = class_bytes("shop/Account");
    let class = parse_class(&bytes).unwrap();

    let code = method(&class, "getBalance")
        .attributes
        .iter()
        .find_map(|a| match &a.data {
            AttributeData::Code(code) => code.bytecode.as_ref(),
            _ => None,
        })
        .unwrap();

    let [
        (_, Opcode::Aload(0)),
        (_, Opcode::Getfield(field)),
        (_, Opcode::Ireturn),
    ] = code.opcodes.as_slice()
    else {
        panic!("unexpected opcodes: {:?}", code.opcodes);
    };
    assert_eq!(field.class_name, "shop/Account");
    assert_eq!(field.name_and_type.name, "balance");
}

#[test]
fn cafebabe_returns_the_generic_signature_as_raw_text() {
    let bytes = class_bytes("shop/Order");
    let class = parse_class(&bytes).unwrap();

    let signature = method(&class, "getLines")
        .attributes
        .iter()
        .find_map(|a| match &a.data {
            AttributeData::Signature(text) => Some(text.to_string()),
            _ => None,
        });

    assert_eq!(
        signature.as_deref(),
        Some("()Ljava/util/List<Lshop/OrderLine;>;")
    );
}

#[test]
fn cafebabe_exposes_the_invisible_lombok_generated_annotation() {
    let bytes = class_bytes("shop/Account");
    let class = parse_class(&bytes).unwrap();

    let annotated: Vec<String> = class
        .methods
        .iter()
        .filter(|m| {
            m.attributes.iter().any(|a| match &a.data {
                AttributeData::RuntimeInvisibleAnnotations(annotations) => {
                    annotations.iter().any(|annotation| {
                        matches!(
                            &annotation.type_descriptor.field_type,
                            FieldType::Object(name) if &**name == "lombok/Generated"
                        )
                    })
                }
                _ => false,
            })
        })
        .map(|m| m.name.to_string())
        .collect();

    assert_eq!(
        annotated,
        ["getBalance", "getOwner", "setBalance", "setOwner"]
    );
}

#[test]
fn cafebabe_flags_the_bridge_method_and_the_lambda_as_synthetic() {
    let bytes = class_bytes("shop/Order");
    let class = parse_class(&bytes).unwrap();

    let synthetic: Vec<String> = class
        .methods
        .iter()
        .filter(|m| {
            m.access_flags
                .contains(cafebabe::MethodAccessFlags::SYNTHETIC)
        })
        .map(|m| m.name.to_string())
        .collect();

    assert_eq!(synthetic, ["compareTo", "lambda$total$0"]);
}

#[test]
fn cafebabe_reports_a_record_through_its_superclass() {
    let bytes = class_bytes("shop/Address");
    let class = parse_class(&bytes).unwrap();

    let superclass = class.super_class.map(|name| name.to_string());

    assert_eq!(superclass.as_deref(), Some("java/lang/Record"));
}

#[test]
fn cafebabe_rejects_a_truncated_class_without_panicking() {
    let bytes = class_bytes("shop/Order");

    let results: Vec<bool> = (0..bytes.len())
        .map(|length| parse_class(&bytes[..length]).is_err())
        .collect();

    assert!(results.iter().all(|rejected| *rejected));
}

#[test]
fn zip_lists_jar_entries_and_reads_class_bytes() {
    let file = File::open(format!("{}/fixtures.jar", fixtures())).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();

    let names: Vec<String> = archive
        .file_names()
        .map(|name| name.unwrap().into_owned())
        .collect();
    let mut wallet = Vec::new();
    archive
        .by_name("lib/Wallet.class")
        .unwrap()
        .read_to_end(&mut wallet)
        .unwrap();

    assert!(names.contains(&"lib/Currency.class".to_owned()));
    assert_eq!(&wallet[..4], [0xCA, 0xFE, 0xBA, 0xBE]);
}
