package shop.domain;

public interface OrderRepository {
    Order findById(OrderId id);

    Order getById(OrderId id);
}
