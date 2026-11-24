package shop;

import java.util.Collections;
import java.util.List;

public class Order {
    private List<String> lines;

    public List<String> getLines() {
        return Collections.unmodifiableList(lines);
    }
}
