pub struct Wallet {
    pub balance: i32,
}

impl Wallet {
    pub fn debit(&mut self, amount: i32) {
        self.balance -= amount;
    }
}
