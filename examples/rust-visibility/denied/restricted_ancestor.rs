mod region {
    mod hidden {
        pub(crate) fn amount() -> u32 {
            17
        }
    }
}

fn main() {
    assert_eq!(region::hidden::amount(), 17);
}
