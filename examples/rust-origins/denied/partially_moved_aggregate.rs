struct Order {
    wallet: Box<i32>,
    label: String,
}

fn main() {
    let order = Order {
        wallet: Box::new(50),
        label: String::from("pending"),
    };
    let wallet = order.wallet;

    assert_eq!(*wallet, 50);
    let restored = order;
    assert_eq!(restored.label, "pending");
}
