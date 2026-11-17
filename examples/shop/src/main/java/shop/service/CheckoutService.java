package shop.service;

import java.util.Map;
import shop.domain.Cart;
import shop.domain.Order;
import shop.domain.Session;

public class CheckoutService {
    private final Map<String, Cart> carts;

    public CheckoutService(Map<String, Cart> carts) {
        this.carts = carts;
    }

    public void clearCart(Session session) {
        session.getCart().clear();
    }

    public void checkout(String id) {
        carts.get(id).checkout();
    }

    public void discountFirstLine(Order order, int rate) {
        order.getLines().get(0).applyDiscount(rate);
    }

    public void checkoutExisting(String id) {
        existingCart(id).checkout();
    }

    private Cart existingCart(String id) {
        Cart cart = carts.get(id);
        if (cart == null) {
            throw new IllegalStateException("no cart " + id);
        }
        return cart;
    }
}
