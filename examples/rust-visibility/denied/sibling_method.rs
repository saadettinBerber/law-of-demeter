mod ledger {
    pub struct Wallet;

    impl Wallet {
        fn balance(&self) -> u32 {
            50
        }
    }
}

mod reporting {
    pub fn balance() -> u32 {
        super::ledger::Wallet.balance()
    }
}

fn main() {
    assert_eq!(reporting::balance(), 50);
}
