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

    protected OrderLine findLine(String id) {
        return lines.get(id);
    }

    public void discountOrder(Order order) {
        order.applyDiscount(10);
    }

    public void discountLines(Order order) {
        new Batch<>(order.getLines(), line -> line.applyDiscount(10));
    }
}
