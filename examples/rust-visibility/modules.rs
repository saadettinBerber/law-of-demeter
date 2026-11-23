#[test]
fn another_type_in_the_defining_module_can_read_a_private_field() {
    let wallet = region::ledger::Wallet::new(50);

    let balance = region::ledger::Reader::field(&wallet);

    assert_eq!(balance, 50);
}

#[test]
fn another_type_in_the_defining_module_can_call_a_private_method() {
    let wallet = region::ledger::Wallet::new(50);

    let balance = region::ledger::Reader::method(&wallet);

    assert_eq!(balance, 50);
}

#[test]
fn a_descendant_can_access_a_private_field_and_method() {
    let wallet = region::ledger::Wallet::new(50);

    let balances = region::ledger::descendant::read(&wallet);

    assert_eq!(balances, (50, 50));
}

#[test]
fn another_type_in_the_defining_module_can_write_a_private_field() {
    let mut wallet = region::ledger::Wallet::new(50);

    region::ledger::Reader::replace(&mut wallet, 70);
    let balance_after_write = region::ledger::Reader::field(&wallet);

    assert_eq!(balance_after_write, 70);
}

#[test]
fn a_parent_module_can_call_a_pub_super_method() {
    let wallet = region::ledger::Wallet::new(50);

    let balance = region::read_parent_scope(&wallet);

    assert_eq!(balance, 50);
}

#[test]
fn a_named_ancestor_scope_can_access_a_pub_in_function() {
    let expected = 17;

    let amount = region::read_named_scope();

    assert_eq!(amount, expected);
}

#[test]
fn a_sibling_module_can_call_a_pub_crate_method() {
    let wallet = region::ledger::Wallet::new(50);

    let balance = reporting::read(&wallet);

    assert_eq!(balance, 50);
}

#[test]
fn a_public_function_remains_accessible_inside_its_private_ancestor() {
    let expected = 23;

    let amount = region::read_hidden();

    assert_eq!(amount, expected);
}

#[test]
fn a_private_inherent_method_is_callable_in_its_impl_module() {
    let expected = 31;

    let amount = split_impl::read();

    assert_eq!(amount, expected);
}

mod region {
    pub(crate) mod ledger {
        pub(crate) struct Wallet {
            balance: u32,
        }

        impl Wallet {
            pub(crate) fn new(balance: u32) -> Self {
                Self { balance }
            }

            fn private_balance(&self) -> u32 {
                self.balance
            }

            pub(super) fn parent_balance(&self) -> u32 {
                self.balance
            }

            pub(crate) fn crate_balance(&self) -> u32 {
                self.balance
            }
        }

        pub(crate) struct Reader;

        impl Reader {
            pub(crate) fn field(wallet: &Wallet) -> u32 {
                wallet.balance
            }

            pub(crate) fn method(wallet: &Wallet) -> u32 {
                wallet.private_balance()
            }

            pub(crate) fn replace(wallet: &mut Wallet, balance: u32) {
                wallet.balance = balance;
            }
        }

        pub(crate) mod descendant {
            use super::Wallet;

            pub(crate) fn read(wallet: &Wallet) -> (u32, u32) {
                (wallet.balance, wallet.private_balance())
            }

            pub(in crate::region) fn named_scope() -> u32 {
                17
            }
        }
    }

    pub(crate) fn read_parent_scope(wallet: &ledger::Wallet) -> u32 {
        wallet.parent_balance()
    }

    pub(crate) fn read_named_scope() -> u32 {
        ledger::descendant::named_scope()
    }

    pub(crate) fn read_hidden() -> u32 {
        hidden::amount()
    }

    mod hidden {
        pub fn amount() -> u32 {
            23
        }
    }
}

mod reporting {
    use super::region::ledger::Wallet;

    pub(crate) fn read(wallet: &Wallet) -> u32 {
        wallet.crate_balance()
    }
}

mod split_type {
    pub(crate) struct Marker;
}

mod split_impl {
    use super::split_type::Marker;

    impl Marker {
        fn amount(&self) -> u32 {
            31
        }
    }

    pub(crate) fn read() -> u32 {
        Marker.amount()
    }
}
