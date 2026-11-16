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

    public static <T extends Entity> T asEntity(T value) {
        return value;
    }

    public static <T extends Comparable<T>> T asComparable(T value) {
        return value;
    }

    public static <T extends Entity & Comparable<T>> T asBoth(T value) {
        return value;
    }

    public static <T> T asIs(T value) {
        return value;
    }
}
