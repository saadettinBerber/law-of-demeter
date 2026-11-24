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

impl Wallet {
    pub fn balance(&self) -> i32 {
        self.balance
    }
}

impl Order {
    pub fn wallet_shared(&self) -> &Wallet {
        &self.customer.wallet
    }

    pub fn into_wallet(self) -> Wallet {
        self.customer.wallet
    }
}

pub struct Wrapper {
    wallet: Wallet,
}

impl std::ops::Deref for Wrapper {
    type Target = Wallet;

    fn deref(&self) -> &Wallet {
        &self.wallet
    }
}

impl std::ops::DerefMut for Wrapper {
    fn deref_mut(&mut self) -> &mut Wallet {
        &mut self.wallet
    }
}

impl PaymentService {
    pub fn through_local(&self, order: &mut Order, amount: i32) {
        let customer = order.customer();
        let wallet = customer.wallet();
        wallet.withdraw(amount);
    }

    pub fn through_reborrow(&self, order: &mut Order, amount: i32) {
        let customer = order.customer();
        let temporary = &mut *customer;
        let wallet = temporary.wallet();
        wallet.withdraw(amount);
    }

    pub fn parameter_reborrow(&self, customer: &mut Customer) {
        let temporary = &mut *customer;
        temporary.activate();
    }

    pub fn shared_copy(&self, order: &Order) {
        let wallet = order.wallet_shared();
        let copied = wallet;
        copied.balance();
    }

    pub fn owned_move(&self, order: Order, amount: i32) {
        let wallet = order.into_wallet();
        let mut moved = wallet;
        moved.withdraw(amount);
    }

    pub fn moved_parameter(&self, wallet: Wallet, amount: i32) {
        let mut moved = wallet;
        moved.withdraw(amount);
    }

    pub fn alias_self(&mut self, amount: i32) {
        let alias = &mut *self;
        alias.wallet.withdraw(amount);
    }

    pub fn alias_peer(&self, peer: &mut PaymentService, amount: i32) {
        let alias = &mut *peer;
        alias.wallet.withdraw(amount);
    }

    pub fn create_borrowed(&self, seed: &Seed, amount: i32) {
        let wallet = &mut Wallet {
            balance: seed.amount(),
        };
        wallet.withdraw(amount);
    }

    pub fn custom_projection(&self, wrapper: &mut Wrapper, amount: i32) {
        let inner = &mut **wrapper;
        inner.withdraw(amount);
    }

    pub fn coerced_projection(&self, wrapper: &Wrapper) {
        let inner: &Wallet = wrapper;
        inner.balance();
    }

    pub fn raw_projection(&self, wallet: &mut Wallet, amount: i32) {
        let pointer = wallet as *mut Wallet;
        unsafe {
            (*pointer).withdraw(amount);
        }
    }

    pub fn shadowed(&mut self, other: &mut PaymentService, amount: i32) {
        let current = &mut *self;
        current.wallet.withdraw(0);
        let current = &mut *other;
        current.wallet.withdraw(amount);
    }

    pub fn reassigned(&self, wallet: &mut Wallet, other: &mut Wallet, amount: i32) {
        let mut current = wallet;
        current.withdraw(0);
        current = other;
        current.withdraw(amount);
    }
}

impl PaymentService {
    pub fn shared_borrow(&self, order: &Order) {
        let wallet = order.wallet_shared();
        let borrowed = &(*wallet);
        borrowed.balance();
    }
}

pub struct Ticket {
    balance: i32,
}

impl Ticket {
    pub fn stamp(&mut self) {
        self.balance += 1;
    }
}

impl Clone for Ticket {
    fn clone(&self) -> Ticket {
        *self
    }
}

impl Copy for Ticket {}

pub struct Dispatch {
    ticket: Ticket,
}

impl Dispatch {
    pub fn ticket(&self) -> Ticket {
        self.ticket
    }
}

impl PaymentService {
    pub fn copy_value(&self, dispatch: &Dispatch) {
        let ticket = dispatch.ticket();
        let mut copied = ticket;
        copied.stamp();
    }
}
