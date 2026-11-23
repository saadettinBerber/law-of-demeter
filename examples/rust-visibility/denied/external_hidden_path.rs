extern crate visibility_library;

fn main() {
    let wallet = visibility_library::hidden::Wallet::new(50);

    assert_eq!(wallet.balance(), 50);
}
