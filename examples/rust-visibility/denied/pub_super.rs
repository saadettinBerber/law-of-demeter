mod region {
    pub mod ledger {
        pub(super) fn amount() -> u32 {
            17
        }
    }
}

fn main() {
    assert_eq!(region::ledger::amount(), 17);
}
