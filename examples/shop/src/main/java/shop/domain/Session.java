package shop.domain;

public class Session {
    private final String id;
    private final Cart cart;

    public Session(String id, Cart cart) {
        this.id = id;
        this.cart = cart;
    }

    public String getId() {
        return id;
    }

    public Cart getCart() {
        return cart;
    }

    public void checkout() {
        cart.checkout();
    }
}
