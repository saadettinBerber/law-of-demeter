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

pub struct Wrapper {
    account: Account,
}

impl std::ops::Deref for Wrapper {
    type Target = Account;

    fn deref(&self) -> &Account {
        &self.account
    }
}

pub struct Processor;

impl Processor {
    pub fn through_trait(&self, parcel: &Parcel) {
        parcel.account().apply();
    }

    pub fn through_deref(&self, wrapper: &Wrapper) {
        wrapper.apply();
    }

    pub fn through_references(&self, account: &&Account) {
        account.apply();
    }

    pub fn through_autoref(&self, account: Account) {
        account.apply();
    }

    pub fn through_inherent(&self, parcel: &mut Parcel) {
        parcel.account();
    }
}
