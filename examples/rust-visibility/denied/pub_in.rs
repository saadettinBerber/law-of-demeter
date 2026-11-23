mod region {
    pub mod ledger {
        pub mod descendant {
            pub(in crate::region) fn amount() -> u32 {
                17
            }
        }
    }
}

fn main() {
    assert_eq!(region::ledger::descendant::amount(), 17);
}
