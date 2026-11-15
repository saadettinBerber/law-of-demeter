package warehouse.orders;

import warehouse.catalog.Product;

public record OrderLine(Product product, int quantity) {
}
