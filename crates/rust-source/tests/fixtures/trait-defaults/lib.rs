pub struct Wallet;

impl Wallet {
    pub fn debit(&self) {}
}

pub trait Checkout {
    fn wallet(&self) -> &Wallet;

    #[cfg(not(feature = "signatures-only"))]
    fn pay(&self) {
        self.wallet().debit();
    }

    #[cfg(any())]
    fn inactive(&self) {
        self.wallet().debit();
    }
}

#[cfg(feature = "generated-default")]
macro_rules! default_checkout {
    () => {
        pub trait GeneratedCheckout {
            fn execute(&self) {}
        }
    };
}

#[cfg(feature = "generated-default")]
default_checkout!();
