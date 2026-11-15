package warehouse.orders;

import java.util.List;

public class Order {
    private final List<OrderLine> lines;

    public Order(List<OrderLine> lines) {
        this.lines = lines;
    }

    public List<OrderLine> getLines() {
        return lines;
    }

    public int count() {
        return lines.size();
    }
}
