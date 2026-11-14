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

## Mimari

Proje altıgen mimariyle (ports & adapters) yazılır. Kural ortadaki domain'de durur. Java'yı, dosyaları ve ayrıştırıcıyı yalnızca adapter'lar bilir.

![law-of-demeter mimarisi: ortada Domain, çevresinde port'larıyla Application; solda CLI, sağda Java ayrıştırıcı, .class okuyucu ve metin raporu. Oklar içe yönelir.](assets/architecture.svg)

| Katman | Sorumluluk |
|---|---|
| **Domain** | Kuralın kendisi: arkadaş listesi, bir tipin veri yapısı, nesne, hibrit ya da değer olarak sınıflandırılması, yabancıya uzanma ve hibrit sınıf. Her çağrı için kararı ve nedenini verir. Dosya okumaz, Java ayrıştırmaz. |
| **Application** | "Projeyi denetle" kullanım senaryosu. Domain kurallarını yürütür; dışarıdan istediklerini port'larla ister. |
| **Adapter** | Port'ları gerçek teknolojiye bağlar: komut satırı, Java ayrıştırıcı, `.class` okuyucu ve rapor. |

| Port | Ne sağlar | Adapter |
|---|---|---|
| Denetim isteği | Denetlenecek kaynaklar ve derlenmiş sınıflar | CLI |
| Kaynak okuyucu | Metotlar ve çağrıları; her çağrının hedefinin nereden geldiği: parametre, alan, `new` ya da başka bir çağrının dönüşü | Java ayrıştırıcı |
| Tip kataloğu | Bir tipin alanları ve metotları; Lombok'un ürettikleri dahil | `.class` okuyucu |
| Rapor çıkışı | Bulgular: dosya, satır ve çiğnenen kural. Kullanıcı isterse uygun çağrılar da uydukları kuralla birlikte. | Metin raporu |

Bağımlılık yalnızca içe doğru akar: adapter → application → domain. Domain verinin hangi yoldan geldiğini bilmez. Bu yüzden domain testleri Java dosyası olmadan yazılır.

Her katman ayrı bir crate'tir: `core` domain ve application'ı, `class-file` ve `java-source` adapter'ları taşır. Bağımlılık yönünü derleyici korur.

<details>
<summary><strong>Örnek: bir çağrının yolu</strong></summary>

```java
class OrderService {
    void pay(Order order, Money amount) {
        order.getCustomer().getWallet().withdraw(amount);
    }
}
```

1. **Kaynak okuyucu:** `withdraw`, `getWallet()`'ın dönüşünde çağrılıyor. `getWallet()`, `getCustomer()`'ın dönüşünde; `getCustomer()` da `order` parametresinde çağrılıyor.
2. **Tip kataloğu:** `Wallet`'ın `withdraw` davranışı var.
3. **Domain:** `Wallet` bir nesne ve f'nin arkadaşı değil. Kural çiğneniyor.
4. **Rapor çıkışı:** Bulgu `OrderService.java` dosyasının 3. satırı için yazılır.

</details>

Kaynak: Alistair Cockburn, [Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture).
