mod split_type {
    pub struct Marker;

    pub fn read() -> u32 {
        Marker.amount()
    }
}

mod split_impl {
    use super::split_type::Marker;

    impl Marker {
        fn amount(&self) -> u32 {
            31
        }
    }
}

fn main() {
    assert_eq!(split_type::read(), 31);
}
