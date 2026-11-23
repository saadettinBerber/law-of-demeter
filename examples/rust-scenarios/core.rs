use std::cell::Cell;

pub struct PaymentService {
    wallet: Wallet,
}

impl PaymentService {
    pub fn on_self(&self) {
        self.debit(7);
    }

    pub fn on_parameter(&self, wallet: &Wallet) {
        wallet.debit(7);
    }

    pub fn on_direct_field(&self) {
        self.wallet.debit(7);
    }

    pub fn on_created(&self) -> Wallet {
        let wallet = Wallet {
            balance: Cell::new(50),
        };
        wallet.debit(7);
        wallet
    }

    pub fn on_returned_chain(&self, order: &Order) {
        order.wallet().debit(7);
    }

    pub fn on_returned_local(&self, order: &Order) {
        let wallet = order.wallet();
        wallet.debit(7);
    }

    #[allow(clippy::borrow_deref_ref)]
    pub fn on_returned_reborrow(&self, order: &Order) {
        let wallet = order.wallet();
        let borrowed = &*wallet;
        borrowed.debit(7);
    }

    pub fn read_data(&self, data: &DeliveryData) -> i32 {
        data.address.zone
    }

    pub fn through_data_wrapper(&self, data: &WalletData) {
        data.wallet.debit(7);
    }

    pub fn pass_returned(&self, order: &Order, receiver: &Receiver) {
        receiver.accept(order.wallet());
    }

    pub fn on_order_parameter(&self, order: &Order) {
        order.debit(7);
    }

    pub fn debit(&self, amount: i32) {
        self.wallet.debit(amount);
    }

    pub fn balance(&self) -> i32 {
        self.wallet.balance()
    }
}

pub struct Wallet {
    balance: Cell<i32>,
}

impl Wallet {
    pub fn debit(&self, amount: i32) {
        self.balance.set(self.balance.get() - amount);
    }

    pub fn balance(&self) -> i32 {
        self.balance.get()
    }
}

pub struct Order {
    wallet: Wallet,
}

impl Order {
    pub fn wallet(&self) -> &Wallet {
        &self.wallet
    }

    pub fn debit(&self, amount: i32) {
        self.wallet.debit(amount);
    }
}

pub struct AddressData {
    pub zone: i32,
}

pub struct DeliveryData {
    pub address: AddressData,
}

pub struct WalletData {
    pub wallet: Wallet,
}

pub struct Receiver {
    accepted: Cell<usize>,
}

impl Receiver {
    pub fn accept(&self, wallet: &Wallet) {
        self.accepted.set(self.accepted.get() + 1);
        wallet.debit(7);
    }

    pub fn accepted(&self) -> usize {
        self.accepted.get()
    }
}

#[cfg(test)]
mod tests {
    use super::{AddressData, DeliveryData, Order, PaymentService, Receiver, Wallet, WalletData};
    use std::cell::Cell;

    #[test]
    fn the_self_case_changes_the_services_wallet() {
        let service = service();

        service.on_self();

        assert_eq!(service.balance(), 43);
    }

    #[test]
    fn the_parameter_case_changes_only_its_input_wallet() {
        let service = service();
        let wallet = wallet();

        service.on_parameter(&wallet);

        assert_eq!((wallet.balance(), service.balance()), (43, 50));
    }

    #[test]
    fn the_field_case_changes_the_services_wallet() {
        let service = service();

        service.on_direct_field();

        assert_eq!(service.balance(), 43);
    }

    #[test]
    fn the_creation_case_changes_only_the_created_wallet() {
        let service = service();

        let created = service.on_created();

        assert_eq!((created.balance(), service.balance()), (43, 50));
    }

    #[test]
    fn chain_local_and_reborrow_have_the_same_observable_effect() {
        let service = service();
        let chained = order();
        let local = order();
        let reborrowed = order();

        service.on_returned_chain(&chained);
        service.on_returned_local(&local);
        service.on_returned_reborrow(&reborrowed);

        assert_eq!(
            (
                chained.wallet().balance(),
                local.wallet().balance(),
                reborrowed.wallet().balance()
            ),
            (43, 43, 43)
        );
        assert_eq!(service.balance(), 50);
    }

    #[test]
    fn the_data_case_reads_the_nested_scalar() {
        let service = service();
        let data = DeliveryData {
            address: AddressData { zone: 7 },
        };

        let zone = service.read_data(&data);

        assert_eq!(zone, 7);
    }

    #[test]
    fn the_wrapper_case_changes_only_the_contained_wallet() {
        let service = service();
        let data = WalletData { wallet: wallet() };

        service.through_data_wrapper(&data);

        assert_eq!((data.wallet.balance(), service.balance()), (43, 50));
    }

    #[test]
    fn passing_the_returned_wallet_lets_the_receiver_use_its_parameter() {
        let service = service();
        let order = order();
        let receiver = Receiver {
            accepted: Cell::new(0),
        };

        service.pass_returned(&order, &receiver);

        assert_eq!(receiver.accepted(), 1);
        assert_eq!((order.wallet().balance(), service.balance()), (43, 50));
    }

    #[test]
    fn forwarding_to_the_order_has_the_same_effect_as_the_chain() {
        let service = service();
        let direct = order();
        let chained = order();

        service.on_order_parameter(&direct);
        service.on_returned_chain(&chained);

        assert_eq!(
            (direct.wallet().balance(), chained.wallet().balance()),
            (43, 43)
        );
        assert_eq!(service.balance(), 50);
    }

    fn service() -> PaymentService {
        PaymentService { wallet: wallet() }
    }

    fn order() -> Order {
        Order { wallet: wallet() }
    }

    fn wallet() -> Wallet {
        Wallet {
            balance: Cell::new(50),
        }
    }
}
