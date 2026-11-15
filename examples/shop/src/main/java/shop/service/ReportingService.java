package shop.service;

import static shop.domain.Status.*;

import java.util.Arrays;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.stream.Collectors;
import java.util.stream.Stream;
import shop.domain.Customer;
import shop.domain.Order;
import shop.domain.Shape;
import shop.domain.Wallet;

public class ReportingService {
    private final List<Customer> customers;

    public ReportingService(List<Customer> customers) {
        this.customers = customers;
    }

    public void rename(Order[] batch, String prefix) {
        Arrays.stream(batch).forEach(order -> order.getCustomer().rename(prefix));
    }

    public List<Wallet> wallets(Optional<Wallet> first, Optional<Wallet> second, Optional<Wallet> third) {
        return Stream.of(first, second, third).flatMap(Optional::stream).collect(Collectors.toList());
    }

    public Map<Wallet, List<Customer>> byWallet() {
        return customers.stream().collect(Collectors.groupingBy(customer -> customer.getWallet()));
    }

    public <T extends Shape> double area(List<T> shapes) {
        return shapes.get(0).getOutline().length();
    }

    public String paidName() {
        return PAID.name();
    }
}
