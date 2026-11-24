mod parcel;

use parcel::{Account, Parcel};

pub struct Processor;

impl Processor {
    pub fn direct(&self, account: &Account) {
        account.apply(3);
    }

    pub fn chained(&self, parcel: &Parcel) {
        parcel.account().apply(3);
    }

    #[cfg(feature = "missing")]
    pub fn missing(&self, account: &Account) {
        account.missing_debit(3);
    }

    #[cfg(feature = "wrong-type")]
    pub fn bad_type(&self, account: &Account) {
        account.apply(true);
    }
}

#[cfg(feature = "unknown-type")]
pub struct Broken {
    absent: Unknown,
}

#[cfg(feature = "missing-file")]
mod missing;

#[cfg(feature = "free-function")]
pub fn free_call(account: &Account) {
    account.apply(3);
}

#[cfg(feature = "aggregate")]
pub enum Choice {
    Account(Account),
}
