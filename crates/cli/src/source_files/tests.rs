use std::fs;
use std::path::Path;

use law_of_demeter_core::application::{ReadError, SourceFile, SourceReader};
use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::Method;
use law_of_demeter_core::domain::type_shape::{TypeKind, TypeShape};
use tempfile::TempDir;

use super::{Declarations, Mapper, SourceFiles};

fn write(root: &TempDir, name: &str, text: &str) -> std::path::PathBuf {
    let path = root.path().join(name);
    fs::write(&path, text).expect("write source");
    path
}

#[test]
fn every_file_is_read_and_handed_to_the_mapper_with_the_type_catalog() {
    let root = TempDir::new().expect("temp dir");
    let order = write(&root, "Order.java", "class Order {}");
    let wallet = write(&root, "Wallet.java", "class Wallet {}");
    let mapper: &Mapper = &|path, text, types| {
        let class = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        Ok(Declarations {
            classes: vec![format!("{class}:{text}:{}", types.shape("Known").is_some())],
            methods: vec![Method::new(&class, "pay", 1)],
        })
    };
    let sources = SourceFiles::new(vec![order.clone(), wallet.clone()], mapper);

    let files = sources.read(&Types::new().with_shape(TypeShape::new("Known", TypeKind::Class)));

    assert_eq!(
        files,
        [
            Ok(SourceFile::new(
                &order,
                vec!["Order:class Order {}:true".to_owned()],
                vec![Method::new("Order", "pay", 1)]
            )),
            Ok(SourceFile::new(
                &wallet,
                vec!["Wallet:class Wallet {}:true".to_owned()],
                vec![Method::new("Wallet", "pay", 1)]
            )),
        ]
    );
}

#[test]
fn a_file_that_cannot_be_read_and_a_file_the_mapper_rejects_are_reported_in_place() {
    let root = TempDir::new().expect("temp dir");
    let missing = root.path().join("Missing.java");
    let broken = write(&root, "Broken.java", "class {");
    let good = write(&root, "Good.java", "class Good {}");
    let mapper: &Mapper = &|path, text, _| {
        if text.starts_with("class {") {
            Err(ReadError::at_line(path, 1, "class name expected"))
        } else {
            Ok(Declarations {
                classes: Vec::new(),
                methods: Vec::new(),
            })
        }
    };
    let sources = SourceFiles::new(vec![missing.clone(), broken.clone(), good.clone()], mapper);

    let files = sources.read(&Types::new());

    assert_eq!(
        (
            files[0]
                .as_ref()
                .err()
                .map(|error| error.path().to_path_buf()),
            files[1].clone(),
            files[2].clone()
        ),
        (
            Some(missing),
            Err(ReadError::at_line(&broken, 1, "class name expected")),
            Ok(SourceFile::new(Path::new(&good), Vec::new(), Vec::new()))
        )
    );
}

#[test]
fn errors_found_while_walking_are_yielded_before_the_files() {
    let root = TempDir::new().expect("temp dir");
    let good = write(&root, "Good.java", "class Good {}");
    let mapper: &Mapper = &|_, _, _| {
        Ok(Declarations {
            classes: Vec::new(),
            methods: Vec::new(),
        })
    };
    let walk_error = ReadError::new(Path::new("src/locked"), "permission denied");
    let sources =
        SourceFiles::new(vec![good.clone()], mapper).with_errors(vec![walk_error.clone()]);

    let files = sources.read(&Types::new());

    assert_eq!(
        files,
        [
            Err(walk_error),
            Ok(SourceFile::new(&good, Vec::new(), Vec::new()))
        ]
    );
}
