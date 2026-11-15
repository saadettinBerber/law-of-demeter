package shop.ledger;

class Posting {
    private int balance;

    void settle(int amount) {
        if (amount > balance) {
            throw new IllegalStateException("Insufficient balance");
        }
        balance -= amount;
    }
}
