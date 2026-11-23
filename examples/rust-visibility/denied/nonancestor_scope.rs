mod reporting {}

mod ledger {
    pub(in crate::reporting) fn amount() -> u32 {
        17
    }
}

fn main() {
    assert_eq!(ledger::amount(), 17);
}
