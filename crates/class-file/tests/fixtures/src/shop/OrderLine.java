package shop;

public class OrderLine {
    private int price;

    public int getPrice() {
        return price;
    }

    public void applyDiscount(int rate) {
        price = price - price * rate / 100;
    }
}
