pub struct Account;

impl Account {
    pub fn apply(&self) {}
}

static ACCOUNT: Account = Account;

trait HasAccount {
    fn account(&self) -> &Account {
        &ACCOUNT
    }
}

pub struct Parcel;

impl Parcel {
    pub fn account(&mut self) -> &mut Parcel {
        self
    }
}

impl HasAccount for Parcel {}

pub struct Processor;

impl Processor {
    pub fn through_trait(&self, parcel: &Parcel) {
        parcel.account().apply();
    }
}
