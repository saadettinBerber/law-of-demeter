package shop;

import java.util.ArrayList;
import java.util.List;

public class Order implements Comparable<Order> {
    private static int created;

    private final Customer customer;
    private final List<OrderLine> lines = new ArrayList<>();
    private int[] history = new int[0];

    public Order(Customer customer) {
        this.customer = customer;
        created++;
    }

    public static Order of(Customer customer) {
        return new Order(customer);
    }

    public Customer getCustomer() {
        return customer;
    }

    public List<OrderLine> getLines() {
        return lines;
    }

    public Customer customerOf(Order other) {
        return other.customer;
    }

    public int total() {
        return lines.stream().mapToInt(line -> line.getPrice()).sum();
    }

    public void addLine(OrderLine line) {
        lines.add(line);
        audit();
    }

    private void audit() {
        history = new int[] {total()};
    }

    @Override
    public int compareTo(Order other) {
        return Integer.compare(total(), other.total());
    }

    public static class Tag {
        private final String label;

        public Tag(String label) {
            this.label = label;
        }

        public String getLabel() {
            return label;
        }
    }

    public class Receipt {
        public Order issuer() {
            return Order.this;
        }
    }
}
