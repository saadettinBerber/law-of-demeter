package shop.service;

import java.util.List;
import java.util.function.Consumer;

public class Batch<T> {
    private final List<T> items;
    private final Consumer<T> each;

    public Batch(List<T> items, Consumer<T> each) {
        this.items = items;
        this.each = each;
    }

    public void run() {
        items.forEach(each);
    }
}
