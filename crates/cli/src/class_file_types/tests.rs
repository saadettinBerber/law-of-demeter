use std::path::PathBuf;

use law_of_demeter_class_file::Jdk;
use law_of_demeter_core::application::TypeSource;
use law_of_demeter_core::domain::classification::{Classification, Types};
use law_of_demeter_core::domain::type_shape::TypeRef;

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

#[test]
fn an_unreadable_classpath_entry_is_reported_without_stopping_the_others() {
    let missing = fixtures().join("missing.jar");
    let types = ClassFileTypes::new(vec![missing.clone(), fixtures().join("classes")], None);

    let reading = types.read();

    assert_eq!(
        (
            reading
                .shapes
                .iter()
                .any(|shape| shape.name() == "shop.Order"),
            reading
                .errors
                .iter()
                .map(|error| (
                    error.path().to_path_buf(),
                    error.message().contains("missing.jar")
                ))
                .collect::<Vec<_>>()
        ),
        (true, vec![(missing, false)])
    );
}

#[test]
fn a_jdk_that_cannot_give_the_release_is_reported_and_the_classpath_is_still_read() {
    let jdk = Jdk::at(fixtures().join("jdk"));
    let types = ClassFileTypes::new(vec![fixtures().join("classes")], Some((jdk, 99)));

    let reading = types.read();

    assert_eq!(
        (
            reading
                .shapes
                .iter()
                .any(|shape| shape.name() == "shop.Order"),
            reading.errors.len()
        ),
        (true, 1)
    );
}

#[test]
fn the_reading_carries_what_the_java_platform_knows_about_its_types() {
    let types = ClassFileTypes::new(Vec::new(), None);

    let platform = types.read().platform;

    assert_eq!(
        Types::new()
            .with_platform(platform)
            .classify(&TypeRef::class("java.util.List")),
        Classification::Container
    );
}
