use std::fs;
use std::path::PathBuf;

use cafebabe::attributes::AttributeData;
use cafebabe::bytecode::Opcode;
use cafebabe::parse_class;
use law_of_demeter_class_file::read_types;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fluent/classes")
}

#[test]
fn cafebabe_normalizes_the_receiver_load_before_a_reference_return() {
    let bytes = fs::read(fixtures().join("shop/Builder.class")).expect("compiled builder");
    let class = parse_class(&bytes).expect("valid class");
    let same = class
        .methods
        .iter()
        .find(|method| method.name == "same")
        .expect("same method");

    let bytecode = same
        .attributes
        .iter()
        .find_map(|attribute| match &attribute.data {
            AttributeData::Code(code) => code.bytecode.as_ref(),
            _ => None,
        })
        .expect("parsed bytecode");

    assert!(matches!(
        bytecode.opcodes.as_slice(),
        [(0, Opcode::Aload(0)), (1, Opcode::Areturn)]
    ));
}

#[test]
fn only_returns_of_the_receiver_are_marked_as_receiver_returns() {
    let shapes = read_types(&[fixtures()]).expect("compiled shapes");
    let builder = shapes
        .iter()
        .find(|shape| shape.name() == "shop.Builder")
        .expect("builder");

    let evidence: Vec<_> = builder
        .methods()
        .iter()
        .filter(|method| method.name() != "<init>")
        .map(|method| (method.name(), method.returns_receiver()))
        .collect();

    assert_eq!(
        evidence,
        [
            ("same", true),
            ("add", true),
            ("other", false),
            ("choose", false),
            ("stable", true),
            ("fresh", false),
            ("alias", true),
            ("chooseOther", false),
            ("sameFromEither", true)
        ]
    );
}

#[test]
fn final_dispatch_and_exact_allocations_are_recorded_separately_from_receiver_returns() {
    let shapes = read_types(&[fixtures()]).expect("compiled shapes");
    let builder = shapes
        .iter()
        .find(|shape| shape.name() == "shop.Builder")
        .expect("builder");
    let closed = shapes
        .iter()
        .find(|shape| shape.name() == "shop.Builder$Closed")
        .expect("closed builder");

    let evidence: Vec<_> = builder
        .methods()
        .iter()
        .filter(|method| method.name() != "<init>")
        .map(|method| {
            (
                method.name(),
                method.is_final(),
                method.returns_exact_type(),
            )
        })
        .collect();

    assert!(!builder.is_final());
    assert!(closed.is_final());
    assert_eq!(
        evidence,
        [
            ("same", false, false),
            ("add", false, false),
            ("other", false, false),
            ("choose", false, false),
            ("stable", true, false),
            ("fresh", false, true),
            ("alias", false, false),
            ("chooseOther", false, false),
            ("sameFromEither", false, false)
        ]
    );
}

#[test]
fn overwriting_the_receiver_slot_prevents_receiver_identity_evidence() {
    let mut bytes = fs::read(fixtures().join("shop/Builder.class")).expect("compiled builder");
    let store_offset = {
        let class = parse_class(&bytes).expect("valid class");
        let alias = class
            .methods
            .iter()
            .find(|method| method.name == "alias")
            .expect("alias method");
        let code = alias
            .attributes
            .iter()
            .find_map(|attribute| match &attribute.data {
                AttributeData::Code(code) => Some(code),
                _ => None,
            })
            .expect("method code");
        let bytecode = code.bytecode.as_ref().expect("parsed bytecode");
        let (offset, _) = bytecode
            .opcodes
            .iter()
            .find(|(_, opcode)| matches!(opcode, Opcode::Astore(2)))
            .expect("alias store");
        code.code.as_ptr() as usize - bytes.as_ptr() as usize + offset
    };
    bytes[store_offset] = 0x4b;
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("receiver-slot.class");
    fs::write(&path, bytes).expect("modified class");

    let shapes = read_types(&[path]).expect("modified shape");
    let alias = shapes[0]
        .methods()
        .iter()
        .find(|method| method.name() == "alias")
        .expect("alias method");

    assert!(!alias.returns_receiver());
}
