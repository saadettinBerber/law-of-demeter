mod ledger {
    pub struct Wallet {
        balance: u32,
    }

    pub fn wallet() -> Wallet {
        Wallet { balance: 50 }
    }
}

mod reporting {
    pub fn balance() -> u32 {
        super::ledger::wallet().balance
    }
}

fn main() {
    assert_eq!(reporting::balance(), 50);
}
