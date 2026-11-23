struct Wallet {
    balance: i32,
}

fn main() {
    let mut wallet = Wallet { balance: 50 };
    let parent = &mut wallet;
    let child = &mut *parent;

    parent.balance = 10;
    child.balance = 30;
    assert_eq!(wallet.balance, 30);
}
