package shop.service;

import shop.domain.Wallet;

public abstract class BaseService {
    protected final Wallet wallet;

    protected BaseService(Wallet wallet) {
        this.wallet = wallet;
    }
}
