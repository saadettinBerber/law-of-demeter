#[cfg_attr(feature = "alternate", derive(Clone))]
struct Receipt(u8);

fn main() {
    let receipt = Receipt(7);

    let copied = receipt.clone();

    assert_eq!((receipt.0, copied.0), (7, 7));
}
