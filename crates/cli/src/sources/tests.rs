use std::fs;

use tempfile::TempDir;

use super::java_files;

fn touch(root: &TempDir, relative: &str) {
    let path = root.path().join(relative);
    fs::create_dir_all(path.parent().expect("file has a parent")).expect("create directories");
    fs::write(path, "").expect("write file");
}

#[test]
fn java_files_are_found_recursively_in_a_stable_order() {
    let root = TempDir::new().expect("temp dir");
    touch(&root, "pay/Wallet.java");
    touch(&root, "Order.java");
    touch(&root, "pay/notes.txt");
    touch(&root, "pay/deep/Ledger.java");

    let found = java_files(&[root.path().to_path_buf()]);

    assert_eq!(
        found.files,
        [
            root.path().join("Order.java"),
            root.path().join("pay/Wallet.java"),
            root.path().join("pay/deep/Ledger.java"),
        ]
    );
}

#[test]
fn a_directory_that_cannot_be_read_is_reported_and_the_others_are_still_walked() {
    let root = TempDir::new().expect("temp dir");
    touch(&root, "Order.java");
    let missing = root.path().join("missing");

    let found = java_files(&[missing.clone(), root.path().to_path_buf()]);

    assert_eq!(
        (
            found.files,
            found
                .errors
                .iter()
                .map(|error| error.path().to_path_buf())
                .collect::<Vec<_>>()
        ),
        (vec![root.path().join("Order.java")], vec![missing])
    );
}
