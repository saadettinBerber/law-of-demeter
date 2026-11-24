pub struct Parcel {
    account: Account,
}

impl Parcel {
    pub fn account(&self) -> &Account {
        &self.account
    }
}

pub struct Account;

impl Account {
    pub fn apply(&self, _amount: i32) {}
}
