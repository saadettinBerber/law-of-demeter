package shop.domain;

import lombok.NonNull;

public class Customer {
    private final CustomerId id;
    private final Wallet wallet;
    private String name;
    private Address address;
    private boolean active;

    public Customer(CustomerId id, Wallet wallet, String name) {
        this.id = id;
        this.wallet = wallet;
        this.name = name;
    }

    public CustomerId getId() {
        return id;
    }

    public Wallet getWallet() {
        return wallet;
    }

    public Address getAddress() {
        return address;
    }

    public int balance() {
        return wallet.getBalance();
    }

    public void charge(int amount) {
        wallet.withdraw(amount);
    }

    public void deposit(int amount) {
        wallet.deposit(amount);
    }

    public void activate() {
        this.active = true;
    }

    public void rename(@NonNull String newName) {
        this.name = newName;
    }

    public void setAddress(@NonNull Address address) {
        this.address = address;
    }
}
