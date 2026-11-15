package shop.domain;

public class OrderLine {
    private final String id;
    private int price;
    private final int quantity;

    public OrderLine(String id, int price, int quantity) {
        this.id = id;
        this.price = price;
        this.quantity = quantity;
    }

    public String getId() {
        return id;
    }

    public int getPrice() {
        return price;
    }

    public int getQuantity() {
        return quantity;
    }

    public void applyDiscount(int rate) {
        price -= price * rate / 100;
    }
}
