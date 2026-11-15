package shop.ledger;

import lombok.NonNull;

class Entry {
    private final Posting posting;
    private String label;

    Entry(Posting posting, String label) {
        this.posting = posting;
        this.label = label;
    }

    Posting getPosting() {
        return posting;
    }

    void relabel(@NonNull String newLabel) {
        this.label = newLabel;
    }
}
