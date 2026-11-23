struct Wallet {
    balance: i32,
}

impl Wallet {
    fn balance_mut(&mut self) -> &mut i32 {
        &mut self.balance
    }
}

fn main() {
    let mut wallet = Wallet { balance: 50 };
    let shared = &wallet;

    let balance = wallet.balance_mut();
    *balance = 30;

    assert_eq!(shared.balance, 30);
}
