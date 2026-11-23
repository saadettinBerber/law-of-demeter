use std::collections::HashMap;
use std::hash::{BuildHasherDefault, DefaultHasher};

fn main() {
    let mut wallets: HashMap<String, u32, BuildHasherDefault<DefaultHasher>> = HashMap::default();
    wallets.insert(String::from("main"), 50);

    let value: Option<&BuildHasherDefault<DefaultHasher>> = wallets.get("main");

    assert!(value.is_some());
}
