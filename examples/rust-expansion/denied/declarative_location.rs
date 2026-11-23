macro_rules! invoke_missing {
    ($receiver:expr) => {
        $receiver.absent()
    };
}

fn main() {
    let wallet = Wallet;

    invoke_missing!(wallet);
}

struct Wallet;
