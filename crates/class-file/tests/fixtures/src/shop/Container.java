package shop;

public abstract class Container<K, V> {
    public abstract K keyOf(V value);
}
