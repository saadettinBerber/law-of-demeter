package shop;

import lombok.Getter;
import lombok.Setter;

@Getter
@Setter
public class Account {
    private int balance;
    private String owner;

    public void withdraw(int amount) {
        if (amount > balance) {
            throw new IllegalStateException("Insufficient balance");
        }
        balance -= amount;
    }
}
