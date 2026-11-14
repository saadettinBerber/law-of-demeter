package shop;

import java.util.List;
import java.util.Map;
import java.util.function.Function;

public class Catalog<T extends Comparable<T>> extends Entity implements Repository<T, Long> {
    private Map<String, List<? extends T>> index;
    private List<?> anything;
    private T[] items;
    private Catalog<String>.Node<Integer> node;

    public T find(Long id) {
        return null;
    }

    public List<T> findAll() {
        return List.of();
    }

    public <R> R convert(Function<? super T, R> converter) {
        return null;
    }

    public <X extends Exception> void rethrow(X failure) throws X {
        throw failure;
    }

    public class Node<U> {
    }
}
