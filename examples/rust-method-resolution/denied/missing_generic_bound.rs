struct Wallet;

impl Wallet {
    fn quote(&self) -> u32 {
        50
    }
}

fn quote_from_generic<T>(source: &T) -> u32 {
    source.quote()
}

fn main() {
    assert_eq!(quote_from_generic(&Wallet), 50);
}
