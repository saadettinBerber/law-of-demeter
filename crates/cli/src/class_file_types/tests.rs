use std::path::PathBuf;

use law_of_demeter_class_file::Jdk;
use law_of_demeter_core::application::TypeSource;

use super::ClassFileTypes;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../class-file/tests/fixtures")
}

fn names(types: &ClassFileTypes) -> Vec<String> {
    let reading = types.read();
    reading
        .shapes
        .iter()
        .map(|shape| shape.name().to_owned())
        .collect()
}

#[test]
fn classpath_directories_are_read_into_shapes() {
    let types = ClassFileTypes::new(vec![fixtures().join("classes")], None);

    let reading = types.read();

    assert_eq!(
        (
            reading
                .shapes
                .iter()
                .any(|shape| shape.name() == "shop.Order"),
            reading.errors.len()
        ),
        (true, 0)
    );
}

#[test]
fn jdk_types_of_the_asked_release_are_added() {
    let jdk = Jdk::at(fixtures().join("jdk"));
    let types = ClassFileTypes::new(Vec::new(), Some((jdk, 8)));

    let names = names(&types);

    assert_eq!(
        names,
        [
            "sample.awt.Canvas",
            "sample.util.Box",
            "sample.util.Pair",
            "sample.util.Tagged"
        ]
    );
}
