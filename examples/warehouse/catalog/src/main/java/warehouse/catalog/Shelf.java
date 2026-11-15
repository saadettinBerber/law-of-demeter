package warehouse.catalog;

import lombok.Getter;
import lombok.Setter;

@Getter
@Setter
public class Shelf {
    private int capacity;
    private int used;

    public boolean fits(int quantity) {
        return used + quantity <= capacity;
    }
}
