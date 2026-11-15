package warehouse.catalog;

import lombok.Getter;

public class Stock {
    @Getter
    private int onHand;

    public Stock(int onHand) {
        this.onHand = onHand;
    }

    public void add(int quantity) {
        onHand += quantity;
    }

    public boolean reserve(int quantity) {
        if (quantity > onHand) {
            return false;
        }
        onHand -= quantity;
        return true;
    }
}
