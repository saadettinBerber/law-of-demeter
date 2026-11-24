package shop;

public class Builder {
    private Builder next;
    private int count;

    public Builder same() { return this; }
    public Builder add(int amount) { count += amount; return this; }
    public Builder other() { return next; }
    public Builder choose(boolean same) { return same ? this : next; }
    public final Builder stable() { return this; }
    public static Builder fresh() { return new Builder(); }
    public Builder alias(Builder other) { Builder local = other; return this; }
    public Builder chooseOther(boolean other) { if (other) return next; return this; }
    public Builder sameFromEither(boolean first) { if (first) return this; return this; }

    public static final class Closed {
        public Closed same() { return this; }
    }
}
