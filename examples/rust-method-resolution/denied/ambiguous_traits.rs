trait BalanceQuote {
    fn quote(&self) -> u32;
}

trait CreditQuote {
    fn quote(&self) -> u32;
}

struct Wallet;

impl BalanceQuote for Wallet {
    fn quote(&self) -> u32 {
        50
    }
}

impl CreditQuote for Wallet {
    fn quote(&self) -> u32 {
        90
    }
}

fn main() {
    assert_eq!(Wallet.quote(), 50);
}
