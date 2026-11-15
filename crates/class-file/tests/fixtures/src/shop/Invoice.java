package shop;

public class Invoice {
    private int total;

    public int getTotal() {
        return total * 2;
    }

    public String describe(String pattern, Object... arguments) {
        return pattern;
    }

    public String label(String[] names) {
        return names[0];
    }

    private int discount() {
        return 0;
    }

    int rounding() {
        return 1;
    }

    protected int tax() {
        return 2;
    }
}
