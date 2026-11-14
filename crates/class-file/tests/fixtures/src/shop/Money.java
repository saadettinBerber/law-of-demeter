package shop;

public final class Money implements Comparable<Money> {
    public static final Money ZERO = new Money(0, "EUR");

    private final long amount;
    private final String currency;

    public Money(long amount, String currency) {
        this.amount = amount;
        this.currency = currency;
    }

    public long getAmount() {
        return amount;
    }

    public String getCurrency() {
        return currency;
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof Money money
                && amount == money.amount
                && currency.equals(money.currency);
    }

    @Override
    public int hashCode() {
        return Long.hashCode(amount) * 31 + currency.hashCode();
    }

    @Override
    public int compareTo(Money other) {
        return Long.compare(amount, other.amount);
    }
}
