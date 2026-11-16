package shop.service;

import static shop.domain.Defaults.*;

public class GuestService {
    public void deposit(int amount) {
        guest().getWallet().deposit(amount);
    }

    public void topUp(int amount) {
        guest().deposit(amount);
    }
}
