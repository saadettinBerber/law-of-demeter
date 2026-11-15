package shop.ledger;

class Reconciler {
    void settle(Statement statement, int amount) {
        statement.getEntry().getPosting().settle(amount);
    }

    String noteOf(Statement statement) {
        return statement.getMemo().getText();
    }
}
