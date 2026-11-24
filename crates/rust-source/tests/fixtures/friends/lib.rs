pub struct Wallet {
    balance: i32,
}

impl Wallet {
    pub fn withdraw(&mut self, amount: i32) {
        self.balance -= amount;
    }
}

pub struct Customer {
    wallet: Wallet,
    active: bool,
}

impl Customer {
    pub fn wallet(&mut self) -> &mut Wallet {
        &mut self.wallet
    }

    pub fn activate(&mut self) {
        self.active = true;
    }
}

pub struct Order {
    customer: Customer,
}

impl Order {
    pub fn customer(&mut self) -> &mut Customer {
        &mut self.customer
    }

    pub fn charge(&mut self, amount: i32) {
        self.customer.wallet.withdraw(amount);
    }
}

pub struct PaymentService {
    wallet: Wallet,
}

impl PaymentService {
    pub fn direct(&self, order: &mut Order, amount: i32) {
        order.charge(amount);
    }

    pub fn through_chain(&self, order: &mut Order, amount: i32) {
        order.customer().wallet().withdraw(amount);
    }
}

impl PaymentService {
    pub fn on_self(&mut self, amount: i32) {
        self.debit(amount);
    }

    pub fn debit(&mut self, amount: i32) {
        self.wallet.withdraw(amount);
    }

    pub fn on_held_field(&mut self, amount: i32) {
        self.wallet.withdraw(amount);
    }

    pub fn on_parameter(&self, wallet: &mut Wallet, amount: i32) {
        wallet.withdraw(amount);
    }

    pub fn on_peer_field(&self, peer: &mut PaymentService, amount: i32) {
        peer.wallet.withdraw(amount);
    }
}
