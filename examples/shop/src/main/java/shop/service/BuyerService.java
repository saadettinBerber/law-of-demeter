package shop.service;

import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;
import shop.domain.Customer;
import shop.domain.Order;

public class BuyerService {
    public void activate(List<Order> orders) {
        orders.forEach(Order::activateCustomer);
    }

    public void rewardBuyers(List<Order> orders, int bonus) {
        for (Order order : orders) {
            order.reward(bonus);
            order.ship();
        }
    }

    public void renameBuyers(List<Order> orders, String prefix) {
        for (Customer buyer : orders.stream().collect(Collectors.groupingBy(Order::getCustomer)).keySet()) {
            buyer.rename(prefix);
        }
    }

    public void renameByOrder(List<Order> orders, String prefix) {
        for (Customer buyer : orders.stream().collect(Collectors.toMap(Order::getId, Order::getCustomer)).values()) {
            buyer.rename(prefix);
        }
    }

    public void shipByBuyer(List<Order> orders) {
        for (Map.Entry<Customer, Order> entry : orders.stream().collect(Collectors.toMap(Order::getCustomer, order -> order)).entrySet()) {
            entry.getKey().getWallet();
            entry.getValue().ship();
        }
    }
}
