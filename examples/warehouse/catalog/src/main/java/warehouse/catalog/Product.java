package warehouse.catalog;

import lombok.Getter;

@Getter
public class Product {
    private final Sku sku;
    private final Weight weight;
    private final Stock stock;
    private final Shelf shelf;

    public Product(Sku sku, Weight weight, Stock stock, Shelf shelf) {
        this.sku = sku;
        this.weight = weight;
        this.stock = stock;
        this.shelf = shelf;
    }

    public boolean isAvailable(int quantity) {
        return stock.getOnHand() >= quantity;
    }
}
