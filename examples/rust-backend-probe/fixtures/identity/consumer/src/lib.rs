mod exposed;

pub use exposed::Wallet;
pub use ledger::Wallet as ImportedWallet;

pub fn debit_imported(wallet: &mut ImportedWallet) -> i32 {
    wallet.debit(7);
    wallet.balance
}

pub fn debit_returned() -> i32 {
    let mut wallet = ledger::hidden_wallet();
    wallet.balance = 60;
    wallet.debit(11);
    wallet.balance
}

pub fn debit_local(wallet: &mut Wallet) -> i32 {
    wallet.debit(13);
    wallet.balance
}
