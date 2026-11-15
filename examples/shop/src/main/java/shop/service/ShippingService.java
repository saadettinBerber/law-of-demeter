package shop.service;

import java.util.List;
import shop.domain.Customer;
import shop.domain.OrderShippedEvent;
import shop.domain.ShippingRequest;

public class ShippingService {
    private final List<String> journal;

    public ShippingService(List<String> journal) {
        this.journal = journal;
    }

    public void onShipped(OrderShippedEvent event) {
        log(event.getOrder().getTotal());
    }

    public String cityOf(ShippingRequest request) {
        return request.getAddress().getCity();
    }

    public String keyOf(Customer customer) {
        return customer.getId().getValue();
    }

    private void log(int total) {
        journal.add("shipped " + total);
    }
}
