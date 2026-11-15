package shop.domain;

public class Receipt {
    private final Order order;
    private boolean closed;

    public Receipt(Order order) {
        this.order = order;
    }

    public Order getOrder() {
        return order;
    }

    public void close() {
        closed = true;
    }
}
