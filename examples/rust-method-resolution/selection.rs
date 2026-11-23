use std::ops::Deref;

#[test]
fn a_wrapper_method_is_selected_before_dereferencing_to_the_wallet() {
    let wrapper = WalletWrapper {
        wallet: Wallet { balance: 50 },
    };

    let (method, returned) = wrapper.selected();

    assert_eq!(method, "wrapper");
    assert!(std::ptr::eq(returned, &wrapper.wallet));
    assert_eq!(returned.balance, 50);
}

#[test]
fn automatic_dereferencing_can_select_the_referents_method() {
    let wallet = Wallet { balance: 50 };
    let wrapper = WalletRef(&wallet);

    let (method, returned) = wrapper.selected();

    assert_eq!(method, "wallet");
    assert!(std::ptr::eq(returned, &wallet));
}

#[test]
fn array_method_lookup_can_reach_a_trait_implemented_for_slices() {
    let amounts = [20, 30];

    let total = amounts.slice_total();

    assert_eq!(total, 50);
}

#[test]
fn traits_in_scope_can_select_different_methods_with_the_same_name() {
    let wallet = Wallet { balance: 50 };

    let balance_method = balance_scope::quoted(&wallet);
    let credit_method = credit_scope::quoted(&wallet);

    assert_eq!((balance_method, credit_method), ("balance", "credit"));
}

#[test]
fn explicit_trait_paths_choose_the_requested_declarations() {
    let wallet = Wallet { balance: 50 };

    let from_dot = balance_scope::quoted(&wallet);
    let balance_method = <Wallet as BalanceQuote>::quote(&wallet);
    let credit_method = <Wallet as CreditQuote>::quote(&wallet);

    assert_eq!(
        (from_dot, balance_method, credit_method),
        ("balance", "balance", "credit")
    );
}

#[test]
fn a_generic_bound_selects_the_trait_method_instead_of_the_concrete_inherent_method() {
    let source = QuoteSource { amount: 50 };

    let concrete = source.quote();
    let generic: u32 = quote_from_bound(&source);

    assert_eq!(concrete, "inherent");
    assert_eq!(generic, 50);
}

#[test]
fn trait_qualification_can_differ_from_dot_and_type_qualified_receiver_calls() {
    let source = QuoteSource { amount: 50 };

    let from_dot = source.quote();
    let from_type = QuoteSource::quote(&source);
    let from_trait: u32 = <QuoteSource as Quote>::quote(&source);

    assert_eq!(
        (from_dot, from_type, from_trait),
        ("inherent", "inherent", 50)
    );
}

#[test]
fn a_receiverless_associated_function_can_return_an_existing_argument() {
    let wallet = Wallet { balance: 50 };

    let returned = QuoteSource::forwarded_wallet(&wallet);

    assert!(std::ptr::eq(returned, &wallet));
    assert_eq!(returned.balance, 50);
}

#[test]
fn the_trait_path_for_array_iteration_yields_owned_items_in_every_edition() {
    let wallets = [Wallet { balance: 50 }, Wallet { balance: 90 }];

    let iterator: std::array::IntoIter<Wallet, 2> = IntoIterator::into_iter(wallets);
    let balances: Vec<i32> = iterator.map(|wallet| wallet.balance).collect();

    assert_eq!(balances, [50, 90]);
}

struct Wallet {
    balance: i32,
}

impl Wallet {
    fn selected(&self) -> (&'static str, &Self) {
        ("wallet", self)
    }
}

struct WalletWrapper {
    wallet: Wallet,
}

impl WalletWrapper {
    fn selected(&self) -> (&'static str, &Wallet) {
        ("wrapper", &self.wallet)
    }
}

impl Deref for WalletWrapper {
    type Target = Wallet;

    fn deref(&self) -> &Self::Target {
        &self.wallet
    }
}

struct WalletRef<'a>(&'a Wallet);

impl Deref for WalletRef<'_> {
    type Target = Wallet;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

trait SliceTotal {
    fn slice_total(&self) -> i32;
}

impl SliceTotal for [i32] {
    fn slice_total(&self) -> i32 {
        self.iter().sum()
    }
}

trait BalanceQuote {
    fn quote(&self) -> &'static str;
}

impl BalanceQuote for Wallet {
    fn quote(&self) -> &'static str {
        "balance"
    }
}

trait CreditQuote {
    fn quote(&self) -> &'static str;
}

impl CreditQuote for Wallet {
    fn quote(&self) -> &'static str {
        "credit"
    }
}

mod balance_scope {
    use super::{BalanceQuote, Wallet};

    pub(super) fn quoted(wallet: &Wallet) -> &'static str {
        wallet.quote()
    }
}

mod credit_scope {
    use super::{CreditQuote, Wallet};

    pub(super) fn quoted(wallet: &Wallet) -> &'static str {
        wallet.quote()
    }
}

trait Quote {
    type Output;

    fn quote(&self) -> Self::Output;
}

struct QuoteSource {
    amount: u32,
}

impl QuoteSource {
    fn quote(&self) -> &'static str {
        "inherent"
    }

    fn forwarded_wallet(wallet: &Wallet) -> &Wallet {
        wallet
    }
}

impl Quote for QuoteSource {
    type Output = u32;

    fn quote(&self) -> Self::Output {
        self.amount
    }
}

fn quote_from_bound<T: Quote>(source: &T) -> T::Output {
    source.quote()
}
