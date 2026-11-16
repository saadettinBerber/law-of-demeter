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

    public List<Wallet> walletsOf(Order[] batch) {
        return Arrays.stream(batch).map(Order::getCustomer).map(Customer::getWallet).collect(Collectors.toList());
    }

    public void printAll(List<Wallet> wallets) {
        wallets.stream().map(Optional::of).forEach(System.out::println);
    }

    public void renameBuyers(List<Order> orders, String prefix) {
        for (Customer buyer : orders.stream().collect(Collectors.groupingBy(Order::getCustomer)).keySet()) {
            buyer.rename(prefix);
        }
    }

    public void renameByOrder(List<Order> orders, String prefix) {
        for (Customer buyer : orders.stream().collect(Collectors.toMap(Order::getId, Order::getCustomer)).values()) {
            buyer.rename(prefix);
        }
    }

    public void shipByBuyer(List<Order> orders) {
        for (Map.Entry<Customer, Order> entry : orders.stream().collect(Collectors.toMap(Order::getCustomer, order -> order)).entrySet()) {
            entry.getKey().getWallet();
            entry.getValue().ship();
        }
    }

    public void discountLines(Order order) {
        new Batch<>(order.getLines(), line -> line.applyDiscount(10));
    }

    public List<Integer> balancesOf(Order[] batch) {
        return Arrays.stream(batch).map(Order::balance).toList();
    }
}
