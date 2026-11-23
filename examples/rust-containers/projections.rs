use std::cell::Cell;
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, DefaultHasher};

#[test]
fn a_hashmap_has_distinct_key_value_and_hasher_roles() {
    let key = String::from("main");
    let key_address = key.as_ptr();
    let wallet = Wallet {
        balance: Cell::new(50),
    };
    let wallet_address = &wallet as *const Wallet;
    let mut wallets: HashMap<String, &Wallet, BuildHasherDefault<DefaultHasher>> =
        HashMap::default();
    wallets.insert(key, &wallet);

    let (key, value): (&String, &&Wallet) = wallets.get_key_value("main").expect("key exists");
    let _: &BuildHasherDefault<DefaultHasher> = wallets.hasher();
    let before_write = value.balance.get();
    value.balance.set(30);
    let from_keys: &String = wallets.keys().next().expect("one key exists");
    let from_values: &&Wallet = wallets.values().next().expect("one value exists");

    assert_eq!(key.as_ptr(), key_address);
    assert!(std::ptr::eq(key, from_keys));
    assert!(std::ptr::eq(*value, wallet_address));
    assert!(std::ptr::eq(value, from_values));
    assert_eq!((before_write, from_values.balance.get()), (50, 30));
}

#[test]
fn hashmap_iteration_yields_key_value_pairs_without_relying_on_order() {
    let first = Box::new(Wallet {
        balance: Cell::new(50),
    });
    let second = Box::new(Wallet {
        balance: Cell::new(50),
    });
    let first_address = &*first as *const Wallet;
    let second_address = &*second as *const Wallet;
    let mut wallets = HashMap::new();
    wallets.insert(String::from("first"), first);
    wallets.insert(String::from("second"), second);

    let entries: Vec<(&String, &Box<Wallet>)> = wallets.iter().collect();

    assert_eq!(entries.len(), 2);
    for (name, address) in [("first", first_address), ("second", second_address)] {
        let (_, value) = entries
            .iter()
            .find(|(key, _)| key.as_str() == name)
            .expect("key exists");
        assert!(std::ptr::eq(&***value, address));
        assert_eq!(value.balance.get(), 50);
    }
}

#[test]
fn filtering_keeps_selected_items_when_collecting_borrowed_content() {
    let orders = [
        Order {
            code: 1,
            wallet: Wallet {
                balance: Cell::new(50),
            },
        },
        Order {
            code: 2,
            wallet: Wallet {
                balance: Cell::new(50),
            },
        },
    ];

    let selected: Vec<&Order> = orders.iter().filter(|order| order.code == 2).collect();
    let before_write = selected[0].wallet.balance.get();
    selected[0].wallet.balance.set(30);

    assert_eq!(selected.len(), 1);
    assert!(std::ptr::eq(selected[0], &orders[1]));
    assert_eq!(before_write, 50);
    assert_eq!(
        (
            orders[0].wallet.balance.get(),
            orders[1].wallet.balance.get()
        ),
        (50, 30)
    );
}

#[test]
fn mapping_can_keep_the_item_type_and_change_its_origin() {
    let source = Wallet {
        balance: Cell::new(50),
    };
    let replacement = Wallet {
        balance: Cell::new(50),
    };
    let input: std::iter::Once<&Wallet> = std::iter::once(&source);

    let replaced: Vec<&Wallet> = input.map(|_| &replacement).collect();
    replaced[0].balance.set(30);

    assert_eq!(replaced.len(), 1);
    assert!(std::ptr::eq(replaced[0], &replacement));
    assert!(!std::ptr::eq(replaced[0], &source));
    assert_eq!((source.balance.get(), replacement.balance.get()), (50, 30));
}

#[test]
fn mapping_new_outer_items_can_keep_existing_aliased_descendants() {
    let wallets = [
        Wallet {
            balance: Cell::new(50),
        },
        Wallet {
            balance: Cell::new(50),
        },
    ];

    let snapshots: Vec<Snapshot<'_>> = wallets.iter().map(|wallet| Snapshot { wallet }).collect();
    let before_write = wallets[1].balance.get();
    snapshots[1].wallet.balance.set(30);

    assert_eq!(snapshots.len(), 2);
    assert!(std::ptr::eq(snapshots[0].wallet, &wallets[0]));
    assert!(std::ptr::eq(snapshots[1].wallet, &wallets[1]));
    assert_eq!(before_write, 50);
    assert_eq!(
        (wallets[0].balance.get(), wallets[1].balance.get()),
        (50, 30)
    );
}

#[test]
fn an_alias_uses_standard_option_but_a_project_option_has_its_own_map() {
    let first = Wallet {
        balance: Cell::new(50),
    };
    let second = Wallet {
        balance: Cell::new(50),
    };
    let standard: Maybe<&Wallet> = Some(&first);
    let custom = project::Option { value: &first };
    let calls = Cell::new(0);

    let from_standard = standard.map(|wallet| {
        calls.set(calls.get() + 1);
        (wallet, wallet.balance.get())
    });
    let (standard_wallet, standard_balance) = from_standard.expect("value is present");
    let from_project = custom.map(&second);

    assert_eq!(calls.get(), 1);
    assert!(std::ptr::eq(standard_wallet, &first));
    assert!(std::ptr::eq(from_project, &second));
    assert_eq!(standard_balance, 50);
    assert_eq!((first.balance.get(), second.balance.get()), (50, 50));
}

struct Wallet {
    balance: Cell<i32>,
}

struct Order {
    code: u8,
    wallet: Wallet,
}

struct Snapshot<'a> {
    wallet: &'a Wallet,
}

type Maybe<T> = std::option::Option<T>;

mod project {
    pub(super) struct Option<T> {
        pub(super) value: T,
    }

    impl<T> Option<T> {
        pub(super) fn map(self, replacement: T) -> T {
            let _ = self.value;
            replacement
        }
    }
}
