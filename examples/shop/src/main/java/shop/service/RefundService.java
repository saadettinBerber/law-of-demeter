package shop.service;

import java.util.List;
import shop.domain.Account;
import shop.domain.Credit;
import shop.domain.Notifier;
import shop.domain.Order;
import shop.domain.Wallet;

public class RefundService {
    private final Wallet wallet;
    private final Notifier notifier;
    private final List<String> journal;

    public RefundService(Wallet wallet, Notifier notifier, List<String> journal) {
        this.wallet = wallet;
        this.notifier = notifier;
        this.journal = journal;
    }

    public void refund(int amount) {
        wallet.withdraw(amount);
        this.audit();
    }

    public void notifyCustomer(Order order) {
        notifier.send(order.getCustomer());
    }

    public void settle(Account account, int amount) {
        account.setBalance(account.getBalance() - amount);
    }

    private void audit() {
        journal.add("refund");
    }

    public void settleCredit(Credit credit, int amount) {
        credit.withdraw(amount);
    }
}
