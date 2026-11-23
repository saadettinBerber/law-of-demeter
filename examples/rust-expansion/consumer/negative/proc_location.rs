fn main() {
    let wallet = Wallet;

    let balance = expansion_generator::generated_charge!(wallet);

    assert_eq!(balance, 50);
}

struct Wallet;

impl Wallet {
    fn balance(&self) -> i32 {
        50
    }
}
