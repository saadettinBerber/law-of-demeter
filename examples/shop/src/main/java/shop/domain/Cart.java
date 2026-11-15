package shop.domain;

import java.util.ArrayList;
import java.util.List;

public class Cart {
    private final List<OrderLine> lines;
    private boolean checkedOut;

    public Cart() {
        this.lines = new ArrayList<>();
    }

    public void add(OrderLine line) {
        lines.add(line);
    }

    public void clear() {
        lines.clear();
    }

    public void checkout() {
        checkedOut = true;
    }
}
