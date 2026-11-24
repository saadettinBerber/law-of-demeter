pub struct Wallet;
pub struct Reserve;

impl Wallet {
    pub fn debit(&self) -> u32 {
        17
    }
}

impl Reserve {
    pub fn missing_debit(&self) -> u32 {
        23
    }
}

pub fn known_call(wallet: &Wallet) -> u32 {
    wallet.debit()
}

#[cfg(feature = "alternate")]
pub fn conditional_call(wallet: &Wallet) -> u32 {
    missing_function();
    let balance: u32 = wallet.missing_debit();
    balance
}
