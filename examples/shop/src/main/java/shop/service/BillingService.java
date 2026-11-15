package shop.service;

import java.util.List;
import shop.domain.Order;
import shop.domain.Receipt;

public class BillingService {
    private final List<Order> orders;

    public BillingService(List<Order> orders) {
        this.orders = orders;
    }

    public void charge(int amount) {
        orders.stream().map(Order::getCustomer).forEach(customer -> customer.getWallet().withdraw(amount));
    }

    public void chargeAll(int amount) {
        orders.stream().map(order -> order.getCustomer()).forEach(customer -> customer.getWallet().withdraw(amount));
    }

    public void closeReceipts() {
        orders.stream().map(order -> new Receipt(order)).forEach(receipt -> receipt.close());
    }

    public long paidCount(List<Order> batch) {
        return batch.stream().filter(Order::isPaid).count();
    }
}
