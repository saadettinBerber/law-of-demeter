use std::cell::Cell;

#[test]
fn a_shared_borrow_assigned_to_another_local_keeps_its_referent() {
    let wallet = Wallet { balance: 50 };
    let borrowed = &wallet;

    let local = borrowed;

    assert!(std::ptr::eq(local, &wallet));
    assert_eq!(local.balance, 50);
}

#[test]
fn reassigning_a_reference_keeps_an_earlier_copy_on_its_original_target() {
    let first = Cell::new(50);
    let second = Cell::new(50);
    let mut current = &first;
    let previous = current;

    current = &second;
    current.set(30);

    assert!(std::ptr::eq(previous, &first));
    assert!(std::ptr::eq(current, &second));
    assert_eq!((previous.get(), current.get()), (50, 30));
}

#[test]
fn a_mutable_reborrow_changes_the_wallet_and_then_releases_its_parent() {
    let mut wallet = Wallet { balance: 50 };
    let parent = &mut wallet;

    let after_child = {
        let child = &mut *parent;
        child.balance = 30;
        child.balance
    };
    let parent_observed = parent.balance;
    parent.balance = 20;

    assert_eq!((after_child, parent_observed, wallet.balance), (30, 30, 20));
}

#[test]
fn leaving_a_shadowing_scope_restores_the_outer_binding() {
    let first = Wallet { balance: 50 };
    let second = Wallet { balance: 50 };
    let current = &first;

    let (inner, inner_balance) = {
        let current = &second;
        (current, current.balance)
    };

    assert!(std::ptr::eq(inner, &second));
    assert!(std::ptr::eq(current, &first));
    assert!(!std::ptr::eq(inner, current));
    assert_eq!((inner_balance, current.balance), (50, 50));
}

#[test]
fn destructuring_moves_one_field_and_leaves_the_other_field_usable() {
    let order = Order {
        wallet: Box::new(Wallet { balance: 50 }),
        label: String::from("pending"),
    };
    let address = &*order.wallet as *const Wallet;

    let Order { mut wallet, .. } = order;
    let before_write = wallet.balance;
    wallet.balance = 30;

    assert!(std::ptr::eq(&*wallet, address));
    assert_eq!((before_write, wallet.balance), (50, 30));
    assert_eq!(order.label, "pending");
}

#[test]
fn struct_update_keeps_the_unspecified_reference_field_aliased() {
    let balance = Cell::new(50);
    let original = SharedOrder {
        label: String::from("original"),
        balance: &balance,
    };

    let updated = SharedOrder {
        label: String::from("updated"),
        ..original
    };
    let before_write = original.balance.get();
    updated.balance.set(30);

    assert_eq!(
        (original.label.as_str(), updated.label.as_str()),
        ("original", "updated")
    );
    assert!(std::ptr::eq(updated.balance, original.balance));
    assert_eq!((before_write, original.balance.get()), (50, 30));
}

#[test]
fn cloning_an_outer_value_can_copy_owned_data_and_share_a_descendant() {
    let balance = Cell::new(50);
    let original = SharedOrder {
        label: String::from("original"),
        balance: &balance,
    };

    let mut cloned = original.clone();
    let before_write = original.balance.get();
    cloned.label.push_str(" copy");
    cloned.balance.set(30);

    assert_eq!(
        (original.label.as_str(), cloned.label.as_str()),
        ("original", "original copy")
    );
    assert!(std::ptr::eq(cloned.balance, original.balance));
    assert_eq!((before_write, original.balance.get()), (50, 30));
}

#[test]
fn copying_a_reference_wrapper_keeps_its_shared_descendant() {
    let balance = Cell::new(50);
    let original = WalletView { balance: &balance };

    let copied = original;
    let before_write = original.balance.get();
    copied.balance.set(30);

    assert!(std::ptr::eq(copied.balance, original.balance));
    assert_eq!((before_write, original.balance.get()), (50, 30));
}

#[test]
fn the_same_reference_result_type_can_forward_an_input_or_project_a_field() {
    let wallet = Wallet { balance: 50 };
    let order = Order {
        wallet: Box::new(Wallet { balance: 50 }),
        label: String::from("pending"),
    };

    let forwarded: &Wallet = forwarded_wallet(&wallet);
    let projected: &Wallet = projected_wallet(&order);

    assert!(std::ptr::eq(forwarded, &wallet));
    assert!(std::ptr::eq(projected, &*order.wallet));
    assert!(!std::ptr::eq(forwarded, projected));
    assert_eq!((forwarded.balance, projected.balance), (50, 50));
}

#[test]
fn matching_lifetime_signatures_can_return_different_input_references() {
    let first = Wallet { balance: 50 };
    let second = Wallet { balance: 50 };

    let from_first = first_wallet(&first, &second);
    let from_second = second_wallet(&first, &second);

    assert!(std::ptr::eq(from_first, &first));
    assert!(std::ptr::eq(from_second, &second));
    assert!(!std::ptr::eq(from_first, from_second));
    assert_eq!((from_first.balance, from_second.balance), (50, 50));
}

struct Wallet {
    balance: i32,
}

struct Order {
    wallet: Box<Wallet>,
    label: String,
}

#[derive(Clone)]
struct SharedOrder<'a> {
    label: String,
    balance: &'a Cell<i32>,
}

#[derive(Clone, Copy)]
struct WalletView<'a> {
    balance: &'a Cell<i32>,
}

fn forwarded_wallet(wallet: &Wallet) -> &Wallet {
    wallet
}

fn projected_wallet(order: &Order) -> &Wallet {
    &order.wallet
}

fn first_wallet<'a>(first: &'a Wallet, _second: &'a Wallet) -> &'a Wallet {
    first
}

fn second_wallet<'a>(_first: &'a Wallet, second: &'a Wallet) -> &'a Wallet {
    second
}
