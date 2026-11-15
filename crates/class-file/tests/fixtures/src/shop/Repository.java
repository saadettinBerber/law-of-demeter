package shop;

import java.util.List;

public interface Repository<T, ID> {
    T find(ID id);

    List<T> findAll();

    default boolean exists(ID id) {
        return find(id) != null;
    }

    static <T> List<T> none() {
        return List.of();
    }
}
