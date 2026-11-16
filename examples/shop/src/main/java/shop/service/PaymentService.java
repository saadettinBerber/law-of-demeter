package shop.service;

import shop.domain.Customer;
import shop.domain.Order;
import shop.domain.Wallet;

public class PaymentService extends BaseService {
    public PaymentService(Wallet wallet) {
        super(wallet);
    }

    public void pay(Order order, int amount) {
        order.getCustomer().getWallet().withdraw(amount);
    }

    public void payFromCustomer(Order order, int amount) {
        Customer customer = order.getCustomer();
        customer.getWallet().withdraw(amount);
    }

    public void withdraw(int amount) {
        wallet.withdraw(amount);
    }

    public void charge(Order order, int amount) {
        order.charge(amount);
    }
}
