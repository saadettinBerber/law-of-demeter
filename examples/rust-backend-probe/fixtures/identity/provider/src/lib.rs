mod exposed;
mod hidden;

pub use exposed::Wallet;

pub fn hidden_wallet() -> hidden::Wallet {
    hidden::Wallet { balance: 50 }
}
