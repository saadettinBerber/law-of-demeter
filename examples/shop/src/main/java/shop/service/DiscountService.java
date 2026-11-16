package shop.service;

import java.util.Map;
import shop.domain.Order;
import shop.domain.OrderLine;

public class DiscountService {
    private final Map<String, OrderLine> lines;

    public DiscountService(Map<String, OrderLine> lines) {
        this.lines = lines;
    }

    public void discount(String id, int rate) {
        findLine(id).applyDiscount(rate);
    }

    private OrderLine findLine(String id) {
        return lines.get(id);
    }

    public void discountOrder(Order order) {
        order.applyDiscount(10);
    }
}
