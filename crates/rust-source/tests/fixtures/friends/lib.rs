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
        self.customer.charge(amount);
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

pub struct Seed;

impl Seed {
    pub fn amount(&self) -> i32 {
        7
    }
}

pub struct Pocket(i32);

use Pocket as BalancePocket;

impl Pocket {
    pub fn withdraw(&mut self, amount: i32) {
        self.0 -= amount;
    }

    pub fn new(pocket: Pocket) -> Pocket {
        pocket
    }
}

pub struct Holder {
    wallet: Wallet,
}

impl PaymentService {
    pub fn create_named(&self, seed: &Seed, amount: i32) {
        let mut wallet = Wallet {
            balance: seed.amount(),
        };
        wallet.withdraw(amount);
    }

    pub fn create_tuple(&self, seed: &Seed, amount: i32) {
        let mut pocket = BalancePocket(seed.amount());
        pocket.withdraw(amount);
    }

    pub fn new_holder(&self, wallet: Wallet, amount: i32) {
        let mut holder = Holder { wallet };
        holder.wallet.withdraw(amount);
    }

    pub fn from_function(&self, pocket: Pocket, amount: i32) {
        let mut alias = Pocket::new(pocket);
        alias.withdraw(amount);
    }
}


impl Customer {
    pub fn charge(&mut self, amount: i32) {
        self.wallet.withdraw(amount);
    }
}
