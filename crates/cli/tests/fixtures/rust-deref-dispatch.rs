pub struct Account;

impl Account {
    pub fn apply(&self) {}
}

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
    pub fn through_deref(&self, wrapper: &Wrapper) {
        wrapper.apply();
    }
}
