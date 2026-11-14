package shop;

import java.util.List;

public interface Repository<T, ID> {
    T find(ID id);

    List<T> findAll();
}
