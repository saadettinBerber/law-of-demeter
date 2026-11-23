extern crate visibility_library;

fn main() {
    let wallet: visibility_library::unnameable::Wallet = visibility_library::returned_wallet(50);

    assert_eq!(wallet.balance(), 50);
}
