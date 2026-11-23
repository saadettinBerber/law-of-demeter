struct Wallet {
    balance: i32,
}

fn main() {
    let wallet = Wallet { balance: 50 };
    let moved = wallet;

    assert_eq!(moved.balance, 50);
    assert_eq!(wallet.balance, 50);
}
