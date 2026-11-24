package shop;

import java.util.ArrayList;
import java.util.List;

class Order {
    private final List<Member> items = new ArrayList<>();
    private final Wallet wallet = new Wallet();

    void addMember(Member member) {
        items.add(member);
    }

    static Order create(Member member) {
        Order result = new Order();
        result.addMember(member);
        result.items.add(member);
        return result;
    }

    void copy(Order other, Member member) {
        other.addMember(member);
        other.items.add(member);
    }

    Wallet wallet() {
        return wallet;
    }
}

class Member {
}

class Billing {
    void charge(Order order) {
        Holder holder = new Holder(order.wallet());
        holder.charge();
        holder.wallet.withdraw();
    }
}

class Holder {
    final Wallet wallet;

    Holder(Wallet wallet) {
        this.wallet = wallet;
    }

    void charge() {
        wallet.withdraw();
    }
}

class Wallet {
    private int balance;

    void withdraw() {
        balance--;
    }
}
