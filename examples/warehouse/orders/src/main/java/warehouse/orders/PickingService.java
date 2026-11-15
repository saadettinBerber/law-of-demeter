package warehouse.orders;

import warehouse.catalog.Product;
import warehouse.catalog.Stock;

public class PickingService {
    private final Stock overflow;

    public PickingService(Stock overflow) {
        this.overflow = overflow;
    }

    public boolean canPick(Product product, int quantity) {
        return product.isAvailable(quantity);
    }

    public int onHand(Product product) {
        return product.getStock().getOnHand();
    }

    public boolean pick(Product product, int quantity) {
        Stock stock = product.getStock();
        return stock.reserve(quantity);
    }

    public boolean spill(int quantity) {
        return overflow.reserve(quantity);
    }

    public boolean fits(Product product, int quantity) {
        return product.getShelf().fits(quantity);
    }

    public String codeOf(Product product) {
        return product.getSku().code();
    }

    public int kilograms(Product product) {
        return product.getWeight().toKilograms();
    }

    public int onHandOf(OrderLine line) {
        return line.product().getStock().getOnHand();
    }

    public void append(Order order, OrderLine line) {
        order.getLines().add(line);
    }
}
