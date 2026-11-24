pub struct Wallet;

impl Wallet {
    pub fn debit(&self) {}
}

#[cfg(all(not(feature = "plain"), not(feature = "disabled")))]
macro_rules! payment {
    () => {
        pub fn pay(wallet: &Wallet) {
            wallet.debit();
        }
    };
}

#[cfg(all(not(feature = "plain"), not(feature = "disabled")))]
payment!();

#[cfg(all(feature = "plain", not(feature = "disabled")))]
pub fn pay(wallet: &Wallet) {
    wallet.debit();
}

#[cfg(any())]
pub fn inactive(wallet: &Wallet) {
    wallet.debit();
}
