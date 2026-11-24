package shop;

class OrderService {
    void pay(Order order, int amount) {
        order.charge(amount);
    }

    void reachThrough(Order order, int amount) {
        order.getWallet().withdraw(amount);
    }
}

class Order {
    private final Wallet wallet = new Wallet();

    Wallet getWallet() {
        return wallet;
    }

    void charge(int amount) {
        wallet.withdraw(amount);
    }
}

class Wallet {
    private int balance;

    void withdraw(int amount) {
        balance -= amount;
    }
}
