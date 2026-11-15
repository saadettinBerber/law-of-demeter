package shop.service;

import java.util.Map;
import java.util.concurrent.locks.ReentrantReadWriteLock;

public class PriceCache {
    private final ReentrantReadWriteLock lock;
    private final Map<String, Integer> prices;

    public PriceCache(ReentrantReadWriteLock lock, Map<String, Integer> prices) {
        this.lock = lock;
        this.prices = prices;
    }

    public void update(String sku, int price) {
        lock.writeLock().lock();
        try {
            prices.put(sku, price);
        } finally {
            lock.writeLock().unlock();
        }
    }
}
