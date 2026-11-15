package shop.ledger;

class Statement {
    private final Entry entry;
    private final Memo memo;

    Statement(Entry entry, Memo memo) {
        this.entry = entry;
        this.memo = memo;
    }

    Entry getEntry() {
        return entry;
    }

    Memo getMemo() {
        return memo;
    }
}
