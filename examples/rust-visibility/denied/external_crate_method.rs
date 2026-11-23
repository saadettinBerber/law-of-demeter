extern crate visibility_library;

fn main() {
    let wallet = visibility_library::Wallet::new(50);

    assert_eq!(wallet.crate_balance(), 50);
}
