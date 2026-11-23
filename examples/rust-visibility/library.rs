pub use self::hidden::{Balance, Wallet};

pub fn read_crate_method(wallet: &Wallet) -> u32 {
    wallet.crate_balance()
}

pub fn returned_wallet(balance: u32) -> unnameable::Wallet {
    unnameable::Wallet { balance }
}

mod hidden {
    pub struct Wallet {
        balance: u32,
    }

    impl Wallet {
        pub fn new(balance: u32) -> Self {
            Self { balance }
        }

        pub fn balance(&self) -> u32 {
            self.balance
        }

        pub(crate) fn crate_balance(&self) -> u32 {
            self.balance
        }
    }

    pub trait Balance {
        fn amount(&self) -> u32;
    }

    impl Balance for Wallet {
        fn amount(&self) -> u32 {
            self.balance
        }
    }
}

pub mod private_traits {
    use super::Wallet;

    pub fn read(wallet: &Wallet) -> u32 {
        LocalBalance::local_amount(wallet)
    }

    trait LocalBalance {
        fn local_amount(&self) -> u32;
    }

    impl LocalBalance for Wallet {
        fn local_amount(&self) -> u32 {
            self.balance()
        }
    }
}

mod unnameable {
    pub struct Wallet {
        pub balance: u32,
    }

    impl Wallet {
        pub fn balance(&self) -> u32 {
            self.balance
        }
    }
}
