extern crate visibility_library;

fn main() {
    let wallet = visibility_library::Wallet::new(50);

    let balance = <visibility_library::Wallet as visibility_library::private_traits::LocalBalance>::local_amount(&wallet);

    assert_eq!(balance, 50);
}
