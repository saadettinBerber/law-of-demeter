package shop.domain;

public class Wallet {
    private int balance;

    public int getBalance() {
        return balance;
    }

    public void withdraw(int amount) {
        if (amount > balance) {
            throw new IllegalStateException("Insufficient balance");
        }
        balance -= amount;
    }

    public void deposit(int amount) {
        balance += amount;
    }
}
