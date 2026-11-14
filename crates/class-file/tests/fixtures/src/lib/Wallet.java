package lib;

public class Wallet {
    private int funds;

    public void withdraw(int amount) {
        funds -= amount;
    }
}
