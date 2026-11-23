extern crate visibility_library;

use visibility_library::{Balance, Wallet};

#[test]
fn a_public_reexport_allows_a_second_crate_to_call_an_inherent_method() {
    let wallet = Wallet::new(50);

    let balance = wallet.balance();

    assert_eq!(balance, 50);
}

#[test]
fn a_trait_method_without_a_pub_modifier_is_callable_from_a_second_crate() {
    let wallet = Wallet::new(50);

    let balance = <Wallet as Balance>::amount(&wallet);

    assert_eq!(balance, 50);
}

#[test]
fn a_public_facade_can_use_a_crate_visible_method_in_the_provider() {
    let wallet = Wallet::new(50);

    let balance = visibility_library::read_crate_method(&wallet);

    assert_eq!(balance, 50);
}

#[test]
fn a_public_facade_can_use_a_private_trait_in_the_provider() {
    let wallet = Wallet::new(50);

    let balance = visibility_library::private_traits::read(&wallet);

    assert_eq!(balance, 50);
}

#[test]
fn a_returned_unnameable_value_allows_public_method_calls_and_field_writes() {
    let mut wallet = visibility_library::returned_wallet(50);

    let balance_before_write = wallet.balance();
    wallet.balance = 70;
    let balance_after_write = wallet.balance();

    assert_eq!(balance_before_write, 50);
    assert_eq!(wallet.balance, 70);
    assert_eq!(balance_after_write, 70);
}
