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
