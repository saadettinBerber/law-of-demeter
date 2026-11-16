package shop.domain;

public final class Defaults {
    private Defaults() {
    }

    public static Customer guest() {
        return new Customer(new CustomerId("guest"), new Wallet(), "guest");
    }
}
