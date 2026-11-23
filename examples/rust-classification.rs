use std::cell::{Cell, RefCell};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[test]
fn nested_public_fields_can_be_read_without_methods() {
    let delivery = data::Delivery {
        address: data::Address { zone: 7 },
    };

    let zone = delivery.address.zone;

    assert_eq!(zone, 7);
}

#[test]
fn a_public_field_can_be_written_through_a_mutable_reference() {
    let mut address = data::Address { zone: 7 };
    let borrowed = &mut address;

    borrowed.zone = 9;

    assert_eq!(address.zone, 9);
}

#[test]
fn a_struct_method_can_change_encapsulated_state() {
    let mut wallet = accounts::Wallet::new(50);

    let result = wallet.withdraw(20);

    assert_eq!(result, Ok(()));
    assert_eq!(wallet.balance(), 30);
}

#[test]
fn a_struct_method_can_reject_a_change_without_changing_state() {
    let mut wallet = accounts::Wallet::new(50);

    let result = wallet.withdraw(60);

    assert_eq!(result, Err("insufficient balance"));
    assert_eq!(wallet.balance(), 50);
}

#[test]
fn a_public_field_allows_a_balance_the_method_rejects() {
    let mut wallet = accounts::OpenWallet { balance: 50 };

    let result = wallet.withdraw(60);
    let balance_after_rejection = wallet.balance;
    wallet.balance = -10;

    assert_eq!(result, Err("insufficient balance"));
    assert_eq!(balance_after_rejection, 50);
    assert_eq!(wallet.balance, -10);
}

#[test]
fn a_fieldless_enum_method_can_query_its_variant() {
    let open = Status::Open;
    let closed = Status::Closed;

    let states = (open.is_closed(), closed.is_closed());

    assert_eq!(states, (false, true));
}

#[test]
fn a_fieldless_enum_method_can_replace_its_variant() {
    let mut status = Status::Open;

    status.close();

    assert_eq!(status, Status::Closed);
}

#[test]
fn a_payload_enum_without_methods_can_expose_data_by_matching() {
    let event = DeliveryEvent::Shipped(data::Address { zone: 7 });

    let DeliveryEvent::Shipped(address) = event;
    let zone = address.zone;

    assert_eq!(zone, 7);
}

#[test]
fn a_payload_enum_method_can_change_the_contained_object() {
    let mut account = accounts::Account::Open(accounts::Wallet::new(50));

    let result = account.withdraw(20);

    assert_eq!(result, Ok(()));
    assert_eq!(account.balance(), 30);
}

#[test]
fn a_payload_enum_can_be_changed_by_matching_outside_its_module() {
    let mut account = accounts::Account::Open(accounts::Wallet::new(50));

    let accounts::Account::Open(wallet) = &mut account;
    *wallet = accounts::Wallet::new(90);

    assert_eq!(account.balance(), 90);
}

#[test]
fn another_type_in_the_same_module_can_call_a_private_method() {
    let wallet = accounts::Wallet::new(50);

    let balance = accounts::Reader::balance(&wallet);

    assert_eq!(balance, 50);
}

#[test]
fn another_type_in_the_same_module_can_write_a_private_field() {
    let mut wallet = accounts::Wallet::new(50);

    accounts::Payment::replace_balance(&mut wallet, -10);

    assert_eq!(wallet.balance(), -10);
}

#[test]
fn a_crate_visible_field_can_be_changed_from_a_sibling_module() {
    let mut wallet = accounts::CrateWallet { balance: 50 };

    reporting::replace_balance(&mut wallet, 90);

    assert_eq!(wallet.balance, 90);
}

#[test]
fn derived_equality_and_hash_allow_state_changing_methods() {
    let mut revision = values::Revision::new(1);
    let expected = values::Revision::new(2);

    revision.advance();

    assert_eq!(revision, expected);
    assert_eq!(hash_of(&revision), hash_of(&expected));
}

#[test]
fn equality_and_hash_allow_mutation_through_a_shared_reference() {
    let wallet = SharedWallet {
        balance: Cell::new(50),
    };
    let shared = &wallet;
    let expected = SharedWallet {
        balance: Cell::new(30),
    };

    shared.withdraw(20);

    assert_eq!(wallet, expected);
    assert_eq!(hash_of(&wallet), hash_of(&expected));
}

#[test]
fn a_refcell_can_change_state_through_a_shared_reference() {
    let balance = RefCell::new(50);
    let shared = &balance;

    *shared.borrow_mut() = 30;

    assert_eq!(*balance.borrow(), 30);
}

#[test]
fn a_mutable_receiver_does_not_have_to_change_state() {
    let mut wallet = accounts::Wallet::new(50);

    let observed = wallet.balance_with_exclusive_access();

    assert_eq!(observed, 50);
    assert_eq!(wallet.balance(), 50);
}

#[test]
fn a_method_can_return_a_new_value_and_preserve_the_original() {
    let version = values::Version::new(1);
    let expected = values::Version::new(2);

    let next = version.next();

    assert_eq!(next, expected);
    assert_eq!(version, values::Version::new(1));
}

#[test]
fn an_owned_copy_receiver_can_change_locally_and_preserve_the_original() {
    let version = values::Version::new(1);
    let expected = values::Version::new(2);

    let next = version.next_owned();

    assert_eq!(next, expected);
    assert_eq!(version, values::Version::new(1));
}

fn hash_of(value: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

mod data {
    pub(super) struct Delivery {
        pub address: Address,
    }

    pub(super) struct Address {
        pub zone: u32,
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Status {
    Open,
    Closed,
}

impl Status {
    fn is_closed(&self) -> bool {
        matches!(self, Self::Closed)
    }

    fn close(&mut self) {
        *self = Self::Closed;
    }
}

enum DeliveryEvent {
    Shipped(data::Address),
}

mod accounts {
    pub(super) struct Wallet {
        balance: i32,
    }

    impl Wallet {
        pub(super) fn new(balance: i32) -> Self {
            Self { balance }
        }

        pub(super) fn balance(&self) -> i32 {
            self.read_balance()
        }

        pub(super) fn withdraw(&mut self, amount: i32) -> Result<(), &'static str> {
            if amount > self.balance {
                return Err("insufficient balance");
            }
            self.balance -= amount;
            Ok(())
        }

        pub(super) fn balance_with_exclusive_access(&mut self) -> i32 {
            self.balance
        }

        fn read_balance(&self) -> i32 {
            self.balance
        }
    }

    pub(super) struct OpenWallet {
        pub balance: i32,
    }

    impl OpenWallet {
        pub(super) fn withdraw(&mut self, amount: i32) -> Result<(), &'static str> {
            if amount > self.balance {
                return Err("insufficient balance");
            }
            self.balance -= amount;
            Ok(())
        }
    }

    pub(super) enum Account {
        Open(Wallet),
    }

    impl Account {
        pub(super) fn withdraw(&mut self, amount: i32) -> Result<(), &'static str> {
            let Self::Open(wallet) = self;
            wallet.withdraw(amount)
        }

        pub(super) fn balance(&self) -> i32 {
            let Self::Open(wallet) = self;
            wallet.balance()
        }
    }

    pub(super) struct Reader;

    impl Reader {
        pub(super) fn balance(wallet: &Wallet) -> i32 {
            wallet.read_balance()
        }
    }

    pub(super) struct Payment;

    impl Payment {
        pub(super) fn replace_balance(wallet: &mut Wallet, balance: i32) {
            wallet.balance = balance;
        }
    }

    pub(crate) struct CrateWallet {
        pub(crate) balance: i32,
    }
}

mod reporting {
    use super::accounts::CrateWallet;

    pub(super) fn replace_balance(wallet: &mut CrateWallet, balance: i32) {
        wallet.balance = balance;
    }
}

mod values {
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub(super) struct Revision {
        number: u32,
    }

    impl Revision {
        pub(super) fn new(number: u32) -> Self {
            Self { number }
        }

        pub(super) fn advance(&mut self) {
            self.number += 1;
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(super) struct Version {
        number: u32,
    }

    impl Version {
        pub(super) fn new(number: u32) -> Self {
            Self { number }
        }

        pub(super) fn next(&self) -> Self {
            Self {
                number: self.number + 1,
            }
        }

        pub(super) fn next_owned(mut self) -> Self {
            self.number += 1;
            self
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct SharedWallet {
    balance: Cell<i32>,
}

impl SharedWallet {
    fn withdraw(&self, amount: i32) {
        self.balance.set(self.balance.get() - amount);
    }
}

impl Hash for SharedWallet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.balance.get().hash(state);
    }
}
