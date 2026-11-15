package shop.service;

import java.time.Duration;
import java.time.Instant;
import java.time.temporal.ChronoUnit;
import shop.domain.Address;
import shop.domain.Order;
import shop.domain.OrderId;
import shop.domain.OrderLine;
import shop.domain.OrderRepository;
import shop.domain.Status;
import shop.domain.Version;

public class OrderService {
    private final OrderRepository orders;

    public OrderService(OrderRepository orders) {
        this.orders = orders;
    }

    public boolean canShip(Order order) {
        return order.getStatus() == Status.PAID;
    }

    public Instant expiry(Instant issuedAt, Duration ttl) {
        return issuedAt.plus(ttl).truncatedTo(ChronoUnit.SECONDS);
    }

    public boolean compatible(Order order, Version required) {
        return order.getVersion().isCompatibleWith(required);
    }

    public Order create(String id, int total) {
        return Order.builder().id(id).total(total).build();
    }

    public void dispatch(OrderId id) {
        orders.findById(id).ship();
    }

    public int totalOf(OrderLine line) {
        return line.getPrice() * line.getQuantity();
    }

    public void ship(Order order) {
        if (order.getStatus() == Status.PAID) {
            order.markShipped();
        }
    }

    public void move(Order order, Address address) {
        order.getCustomer().setAddress(address);
    }

    public void addLine(Order order, OrderLine line) {
        order.getLines().add(line);
    }

    public void discount(Order order, int rate) {
        order.getLines().forEach(line -> line.applyDiscount(rate));
    }
}
