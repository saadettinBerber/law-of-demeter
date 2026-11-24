pub struct PaymentService;
pub struct Wallet;
pub struct Order {
    wallet: Wallet,
    backup: Wallet,
    prefer_backup: bool,
}

impl Wallet {
    pub fn debit(&self, _amount: i32) {}
}

impl Order {
    pub fn wallet(&self) -> &Wallet {
        if self.prefer_backup {
            &self.backup
        } else {
            &self.wallet
        }
    }
}

impl PaymentService {
    pub fn on_parameter(&self, wallet: &Wallet) {
        wallet.debit(7);
    }

    pub fn on_returned_chain(&self, order: &Order) {
        order.wallet().debit(7);
    }
}
