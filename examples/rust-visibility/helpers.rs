#[test]
fn a_private_inherent_helper_returns_the_stored_wallet() {
    let wallet = Wallet { balance: 50 };
    let other = Wallet { balance: 90 };
    let order = orders::Order::new(&wallet, &other);

    let selected = order.via_private_helper();

    assert!(std::ptr::eq(selected, &wallet));
    assert_eq!(selected.balance, 50);
}

#[test]
fn a_public_inherent_helper_returns_the_stored_wallet() {
    let wallet = Wallet { balance: 50 };
    let other = Wallet { balance: 90 };
    let order = orders::Order::new(&wallet, &other);

    let selected = order.public_wallet();

    assert!(std::ptr::eq(selected, &wallet));
}

#[test]
fn a_private_helper_can_return_different_stored_wallets() {
    let wallet = Wallet { balance: 50 };
    let other = Wallet { balance: 90 };
    let order = orders::Order::new(&wallet, &other);

    let first = order.pick(false);
    let second = order.pick(true);

    assert!(std::ptr::eq(first, &wallet));
    assert!(std::ptr::eq(second, &other));
}

#[test]
fn a_trait_default_can_return_different_wallets_for_different_implementations() {
    let wallet = Wallet { balance: 50 };
    let other = Wallet { balance: 90 };
    let primary = Primary(&wallet);
    let alternate = Alternate(&other);

    let first = WalletSource::selected(&primary);
    let second = WalletSource::selected(&alternate);

    assert!(std::ptr::eq(first, &wallet));
    assert!(std::ptr::eq(second, &other));
}

#[test]
fn a_trait_implementation_can_replace_a_default_helper() {
    let wallet = Wallet { balance: 50 };
    let other = Wallet { balance: 90 };
    let source = Replaced {
        wallet: &wallet,
        other: &other,
    };

    let selected = WalletSource::selected(&source);

    assert!(std::ptr::eq(selected, &other));
}

#[test]
fn a_public_trait_object_dispatches_to_the_receivers_implementation() {
    let wallet = Wallet { balance: 50 };
    let other = Wallet { balance: 90 };
    let primary = Primary(&wallet);
    let alternate = Alternate(&other);

    let first = from_public_source(&primary);
    let second = from_public_source(&alternate);

    assert!(std::ptr::eq(first, &wallet));
    assert!(std::ptr::eq(second, &other));
}

#[test]
fn a_public_trait_object_uses_an_overridden_selected_method() {
    let wallet = Wallet { balance: 50 };
    let other = Wallet { balance: 90 };
    let source = Replaced {
        wallet: &wallet,
        other: &other,
    };

    let selected = from_public_source(&source);

    assert!(std::ptr::eq(selected, &other));
    assert_eq!(selected.balance, 90);
}

#[test]
fn a_private_trait_object_can_dispatch_to_different_implementations() {
    let wallet = Wallet { balance: 50 };
    let other = Wallet { balance: 90 };
    let primary = Primary(&wallet);
    let alternate = Alternate(&other);

    let first = from_private_source(&primary);
    let second = from_private_source(&alternate);

    assert!(std::ptr::eq(first, &wallet));
    assert!(std::ptr::eq(second, &other));
}

fn from_public_source(source: &dyn WalletSource) -> &Wallet {
    source.selected()
}

fn from_private_source(source: &dyn LocalSource) -> &Wallet {
    source.selected()
}

pub struct Wallet {
    balance: u32,
}

mod orders {
    use super::Wallet;

    pub(super) struct Order<'a> {
        wallet: &'a Wallet,
        other: &'a Wallet,
    }

    impl<'a> Order<'a> {
        pub(super) fn new(wallet: &'a Wallet, other: &'a Wallet) -> Self {
            Self { wallet, other }
        }

        pub fn via_private_helper(&self) -> &Wallet {
            self.private_wallet()
        }

        pub fn public_wallet(&self) -> &Wallet {
            self.wallet
        }

        pub fn pick(&self, alternate: bool) -> &Wallet {
            self.private_pick(alternate)
        }

        fn private_wallet(&self) -> &Wallet {
            self.wallet
        }

        fn private_pick(&self, alternate: bool) -> &Wallet {
            if alternate {
                return self.other;
            }
            self.wallet
        }
    }
}

pub trait WalletSource {
    fn source(&self) -> &Wallet;

    fn selected(&self) -> &Wallet {
        self.source()
    }
}

struct Primary<'a>(&'a Wallet);

impl WalletSource for Primary<'_> {
    fn source(&self) -> &Wallet {
        self.0
    }
}

struct Alternate<'a>(&'a Wallet);

impl WalletSource for Alternate<'_> {
    fn source(&self) -> &Wallet {
        self.0
    }
}

struct Replaced<'a> {
    wallet: &'a Wallet,
    other: &'a Wallet,
}

impl WalletSource for Replaced<'_> {
    fn source(&self) -> &Wallet {
        self.wallet
    }

    fn selected(&self) -> &Wallet {
        self.other
    }
}

trait LocalSource {
    fn selected(&self) -> &Wallet;
}

impl LocalSource for Primary<'_> {
    fn selected(&self) -> &Wallet {
        self.0
    }
}

impl LocalSource for Alternate<'_> {
    fn selected(&self) -> &Wallet {
        self.0
    }
}
