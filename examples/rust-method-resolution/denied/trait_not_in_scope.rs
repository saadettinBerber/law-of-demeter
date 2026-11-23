mod ledger {
    pub struct Wallet;

    pub trait Quote {
        fn quote(&self) -> u32;
    }

    impl Quote for Wallet {
        fn quote(&self) -> u32 {
            50
        }
    }
}

fn main() {
    assert_eq!(ledger::Wallet.quote(), 50);
}
