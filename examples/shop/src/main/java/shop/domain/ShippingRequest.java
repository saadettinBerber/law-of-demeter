package shop.domain;

import lombok.Data;

@Data
public class ShippingRequest {
    private String reference;
    private Address address;
}
