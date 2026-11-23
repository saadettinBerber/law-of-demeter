use std::cell::Cell;
use std::ops::Deref;
use std::rc::Rc;

macro_rules! charge {
    ($wallet:expr, $amount:expr) => {
        $wallet.charge($amount)
    };
}

#[test]
fn an_enum_can_hold_an_object_and_change_it_through_a_method() {
    let account = Account::Open(Wallet::new(50));

    account.charge(20);

    assert_eq!(account.balance(), 30);
}

#[test]
fn a_private_method_can_be_called_from_another_type_in_the_same_module() {
    let balance = visibility::pay(50, 20);

    assert_eq!(balance, 30);
}

#[test]
fn a_shared_reference_does_not_imply_an_immutable_object() {
    let wallet = Wallet::new(50);
    let shared = &wallet;

    shared.charge(20);

    assert_eq!(wallet.balance(), 30);
}

#[test]
fn a_shared_trait_receiver_can_be_selected_before_a_mutable_inherent_receiver() {
    let wallet = Dispatch;

    let selected = wallet.select();

    assert_eq!(selected, "trait");
}

#[test]
fn a_fully_qualified_call_can_choose_a_different_method_with_the_same_name() {
    let mut wallet = Dispatch;

    let inherent = Dispatch::select(&mut wallet);
    let from_trait = <Dispatch as Selection>::select(&wallet);

    assert_eq!((inherent, from_trait), ("inherent", "trait"));
}

#[test]
fn moving_a_handle_preserves_the_object_it_points_to() {
    let wallet = Box::new(Wallet::new(50));
    let address = &*wallet as *const Wallet;

    let moved = move_wallet(wallet);

    assert_eq!(&*moved as *const Wallet, address);
}

#[test]
fn cloning_a_shared_handle_keeps_the_same_object() {
    let wallet = Rc::new(Wallet::new(50));

    let copied = Rc::clone(&wallet);
    copied.charge(20);

    assert!(Rc::ptr_eq(&wallet, &copied));
    assert_eq!(wallet.balance(), 30);
}

#[test]
fn cloning_an_owned_value_can_create_independent_state() {
    let wallet = Wallet::new(50);

    let copied = wallet.clone();
    copied.charge(20);

    assert_eq!((wallet.balance(), copied.balance()), (50, 30));
}

#[test]
fn a_result_iterator_yields_the_success_type_and_not_the_last_type_parameter() {
    let result: Result<Wallet, String> = Ok(Wallet::new(50));

    let balances: Vec<i32> = result.iter().map(Wallet::balance).collect();

    assert_eq!(balances, [50]);
}

#[test]
fn an_iterator_item_type_can_differ_from_its_generic_parameter() {
    let order = Order {
        wallet: Wallet::new(50),
    };
    let iterator = std::iter::once(&order).map(Order::wallet);

    let balances: Vec<i32> = iterator.map(Wallet::balance).collect();

    assert_eq!(balances, [50]);
}

#[test]
fn dereferencing_a_wrapper_reaches_the_wrapped_object() {
    let order = Order {
        wallet: Wallet::new(50),
    };
    let wrapped = WalletRef(order.wallet());

    wrapped.charge(20);

    assert_eq!(order.wallet.balance(), 30);
}

#[test]
fn a_move_closure_can_keep_a_shared_object() {
    let wallet = Rc::new(Wallet::new(50));
    let captured = Rc::clone(&wallet);
    let pay = move || captured.charge(20);

    pay();

    assert_eq!(wallet.balance(), 30);
}

#[test]
fn destructuring_a_reference_borrows_the_existing_field() {
    let order = Order {
        wallet: Wallet::new(50),
    };

    let Order { wallet } = &order;
    wallet.charge(20);

    assert!(std::ptr::eq(wallet, &order.wallet));
    assert_eq!(order.wallet.balance(), 30);
}

#[test]
fn a_macro_can_generate_a_method_call() {
    let wallet = Wallet::new(50);

    charge!(wallet, 20);

    assert_eq!(wallet.balance(), 30);
}

#[test]
fn an_inactive_cfg_item_does_not_need_to_resolve_its_body() {
    let value = selected_by_cfg();

    assert_eq!(value, 42);
}

#[derive(Clone)]
struct Wallet {
    balance: Cell<i32>,
}

impl Wallet {
    fn new(balance: i32) -> Self {
        Self {
            balance: Cell::new(balance),
        }
    }

    fn balance(&self) -> i32 {
        self.balance.get()
    }

    fn charge(&self, amount: i32) {
        self.balance.set(self.balance.get() - amount);
    }
}

enum Account {
    Open(Wallet),
}

impl Account {
    fn charge(&self, amount: i32) {
        let Self::Open(wallet) = self;
        wallet.charge(amount);
    }

    fn balance(&self) -> i32 {
        let Self::Open(wallet) = self;
        wallet.balance()
    }
}

struct Order {
    wallet: Wallet,
}

impl Order {
    fn wallet(&self) -> &Wallet {
        &self.wallet
    }
}

mod visibility {
    use super::Wallet;

    struct Payment;

    impl Payment {
        fn pay(wallet: &Wallet, amount: i32) {
            wallet.charge(amount);
        }
    }

    pub fn pay(balance: i32, amount: i32) -> i32 {
        let wallet = Wallet::new(balance);
        Payment::pay(&wallet, amount);
        wallet.balance()
    }
}

struct Dispatch;

impl Dispatch {
    fn select(&mut self) -> &'static str {
        "inherent"
    }
}

trait Selection {
    fn select(&self) -> &'static str;
}

impl Selection for Dispatch {
    fn select(&self) -> &'static str {
        "trait"
    }
}

fn move_wallet(wallet: Box<Wallet>) -> Box<Wallet> {
    wallet
}

struct WalletRef<'a>(&'a Wallet);

impl Deref for WalletRef<'_> {
    type Target = Wallet;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

#[cfg(any())]
fn selected_by_cfg() -> i32 {
    missing_type::missing_function()
}

#[cfg(not(any()))]
fn selected_by_cfg() -> i32 {
    42
}
