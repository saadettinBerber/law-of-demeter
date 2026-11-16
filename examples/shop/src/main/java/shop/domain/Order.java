package shop.domain;

import java.util.ArrayList;
import java.util.List;

public class Order {
    private final String id;
    private final int total;
    private final Customer customer;
    private final List<OrderLine> lines;
    private final Version version;
    private Status status;

    private Order(Builder builder) {
        this.id = builder.id;
        this.total = builder.total;
        this.customer = builder.customer;
        this.status = builder.status;
        this.lines = new ArrayList<>();
        this.version = new Version(1, 0);
    }

    public static Builder builder() {
        return new Builder();
    }

    public String getId() {
        return id;
    }

    public int getTotal() {
        return total;
    }

    public Customer getCustomer() {
        return customer;
    }

    public List<OrderLine> getLines() {
        return lines;
    }

    public Version getVersion() {
        return version;
    }

    public Status getStatus() {
        return status;
    }

    public boolean isPaid() {
        return status == Status.PAID;
    }

    public void addLine(OrderLine line) {
        if (lines.size() >= 20) {
            throw new IllegalStateException("Too many lines");
        }
        lines.add(line);
    }

    public void ship() {
        if (isPaid()) {
            markShipped();
        }
    }

    public void markShipped() {
        status = Status.SHIPPED;
    }

    public void charge(int amount) {
        customer.charge(amount);
    }

    public int balance() {
        return customer.balance();
    }

    public void activateCustomer() {
        customer.activate();
    }

    public void reward(int bonus) {
        customer.deposit(bonus);
    }

    public void applyDiscount(int rate) {
        lines.forEach(line -> line.applyDiscount(rate));
    }

    public static class Builder {
        private String id;
        private int total;
        private Customer customer;
        private Status status;

        public Builder id(String id) {
            this.id = id;
            return this;
        }

        public Builder total(int total) {
            this.total = total;
            return this;
        }

        public Builder customer(Customer customer) {
            this.customer = customer;
            return this;
        }

        public Builder status(Status status) {
            this.status = status;
            return this;
        }

        public Order build() {
            return new Order(this);
        }
    }
}
