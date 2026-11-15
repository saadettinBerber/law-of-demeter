package shop.domain;

public class Event {
    private final Receipt receipt;

    public Event(Receipt receipt) {
        this.receipt = receipt;
    }

    public Receipt getReceipt() {
        return receipt;
    }

    public void acknowledge() {
        receipt.close();
    }
}
