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

```java
class OrderService {
    void pay(Order order, Money amount) {
        order.getCustomer().getWallet().withdraw(amount);
    }
}
```

Burada C `OrderService`, f `pay` metodu. `order`, f'ye argüman olarak verildiği için f onun metodunu çağırabilir; `order.getCustomer()` serbest. Bu çağrının döndürdüğü müşteri ise listede yok. `getWallet()` ve `withdraw()` yabancılarla konuşuyor. Cüzdanın yeri değişirse `pay` de değişmek zorunda kalır.

```java
class OrderService {
    void pay(Order order, Money amount) {
        order.charge(amount);
    }
}
```

f artık yalnızca argümanıyla konuşuyor. Müşteriye ve cüzdana nasıl ulaşılacağını `Order` biliyor.

## Veri yapıları

Kural nesneler içindir. Nesne verisini gizler ve davranışını açar. Veri yapısı ise verisini açar ve davranış taşımaz. Veri yapısında zincirle okumak kuralı çiğnemez, çünkü gizlenecek bir iç yapı yoktur. `config` ve `server` yalnızca veri taşıyorsa şu satır sorun değildir:

```java
config.server().port();
```

Hem verisini dışarı açan hem davranış taşıyan hibrit sınıflar iki tarafın kötü yanını alır. Alanları dışarıdan değiştirilebildiği için kendi kurallarını koruyamazlar.

Kaynak: Robert C. Martin, *Clean Code*, 6. bölüm.
