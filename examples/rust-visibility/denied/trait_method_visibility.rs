trait Balance {
    fn amount(&self) -> u32;
}

struct Wallet;

impl Balance for Wallet {
    pub fn amount(&self) -> u32 {
        50
    }
}

fn main() {
    assert_eq!(Wallet.amount(), 50);
}
