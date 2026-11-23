struct QuoteSource;

impl QuoteSource {
    fn forwarded_amount(amount: &u32) -> &u32 {
        amount
    }
}

fn main() {
    let source = QuoteSource;

    assert_eq!(*source.forwarded_amount(&50), 50);
}
