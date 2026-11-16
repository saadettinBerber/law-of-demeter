package shop.service;

import java.util.List;
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
}
