package warehouse.catalog;

import lombok.Value;

@Value
public class Weight {
    int grams;

    public int toKilograms() {
        return grams / 1000;
    }
}
