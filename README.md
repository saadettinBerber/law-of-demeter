# Law of Demeter

Kural iki kısaltmayla anlatılır:

- **C:** Bir sınıf.
- **f:** C sınıfındaki bir metot.

f yalnızca şu nesnelerin metotlarını çağırabilir:

- C'nin kendisi
- f'nin içinde oluşturulan bir nesne
- f'ye argüman olarak verilen bir nesne
- C'nin bir alanında tutulan nesne

f, bu nesnelerin döndürdüğü nesnelerin metotlarını çağırmaz. Kısaca: arkadaşlarınla konuş, yabancılarla konuşma.

Örneklerde C `OrderService` sınıfı, f de `pay` metodu.

<details>
<summary><strong>Uygun değil</strong></summary>

```java
class OrderService {
    void pay(Order order, Money amount) {
        order.getCustomer().getWallet().withdraw(amount);
    }
}
```

Zincirdeki her çağrı ayrı değerlendirilir:

1. `order.getCustomer()`: Serbest. `order`, f'ye argüman olarak verildi.
2. `.getWallet()`: Kuralı çiğner. Bu metot, `getCustomer()`'ın döndürdüğü müşteride çağrılıyor ve müşteri listede yok.
3. `.withdraw(amount)`: Kuralı çiğner. Bu metot, `getWallet()`'ın döndürdüğü cüzdanda çağrılıyor ve cüzdan da listede yok.

Sonuç: `pay`, siparişin bir müşterisi, müşterinin de bir cüzdanı olduğunu bilmek zorunda. Cüzdanın yeri değişirse `pay` de değişir.

</details>

<details>
<summary><strong>Uygun</strong></summary>

```java
class OrderService {
    void pay(Order order, Money amount) {
        order.charge(amount);
    }
}
```

1. `order.charge(amount)`: Serbest. `order`, f'ye argüman olarak verildi.

Sonuç: f yalnızca argümanıyla konuşuyor. Müşteriye ve cüzdana nasıl ulaşılacağını `Order` biliyor.

</details>

## Veri yapıları

Kural nesneler içindir. Nesne verisini gizler ve davranışını açar. Veri yapısı ise verisini açar ve davranış taşımaz.

<details>
<summary><strong>Uygun değil</strong></summary>

Hem verisini açan hem davranış taşıyan sınıfa hibrit denir. Örnekte C `PaymentService` sınıfı, f de `pay` metodu.

```java
class Account {
    private int balance;

    int getBalance() {
        return balance;
    }

    void setBalance(int balance) {
        this.balance = balance;
    }

    void withdraw(int amount) {
        if (amount > balance) {
            throw new IllegalStateException("Insufficient balance");
        }
        balance -= amount;
    }
}

class PaymentService {
    void pay(Account account, int amount) {
        account.setBalance(account.getBalance() - amount);
    }
}
```

1. `Account` verisini `getBalance()` ve `setBalance()` ile açıyor, bir yandan da `withdraw()` davranışını taşıyor.
2. `account.setBalance(account.getBalance() - amount)`: `withdraw()` içindeki bakiye kontrolünü atlıyor. Bakiye eksiye düşebilir.

Sonuç: Alanları dışarıdan değiştirilebildiği için hibrit sınıf kendi kuralını koruyamaz.

</details>

<details>
<summary><strong>Uygun</strong></summary>

Veri yapısında zincirle okumak kuralı çiğnemez, çünkü gizlenecek bir iç yapı yoktur. Örnekte C `Connector` sınıfı, f de `portOf` metodu.

```java
record Server(String host, int port) {
}

record Config(Server server) {
}

class Connector {
    int portOf(Config config) {
        return config.server().port();
    }
}
```

1. `config.server()`: Serbest. `config`, f'ye argüman olarak verildi.
2. `.port()`: Serbest. `server()`'ın döndürdüğü `Server` bir veri yapısı; yalnızca veri taşıyor.

Sonuç: `Config` ve `Server` davranış taşımadığı için yapılarını açmaları beklenen bir durum.

</details>

Kaynak: Robert C. Martin, *Clean Code*, 6. bölüm.
