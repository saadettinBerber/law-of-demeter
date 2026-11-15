package shop;

import java.util.List;

public class Registry<V> extends Container<String, V> implements Repository<V, String> {
    public String keyOf(V value) {
        return null;
    }

    public V find(String id) {
        return null;
    }

    public List<V> findAll() {
        return List.of();
    }
}
