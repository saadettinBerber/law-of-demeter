# Law of Demeter

Bir metot yalnızca yakın çevresiyle konuşur. Yakın çevre dört şeyden oluşur:

- Metodun ait olduğu nesne
- Metodun parametreleri
- Metodun içinde oluşturduğu nesneler
- Nesnenin kendi alanları

Bunlardan birinin döndürdüğü başka bir nesneyle konuşmak kuralı çiğner. Kısaca: arkadaşlarınla konuş, yabancılarla konuşma.

```java
order.getCustomer().getWallet().withdraw(amount);
```

Bu satır siparişten müşteriye, müşteriden cüzdana uzanıyor. Çağıran, siparişin bir müşterisi, müşterinin de bir cüzdanı olduğunu bilmek zorunda kalıyor. Cüzdanın yeri değişirse bu satır da değişir.

```java
order.charge(amount);
```

Burada çağıran yalnızca siparişe ne yapacağını söylüyor. Müşteriye ve cüzdana nasıl ulaşılacağını sipariş biliyor.

## Veri yapıları

Kural nesneler içindir. Nesne verisini gizler ve davranışını açar. Veri yapısı ise verisini açar ve davranış taşımaz. Veri yapısında zincirle okumak kuralı çiğnemez, çünkü gizlenecek bir iç yapı yoktur. `config` ve `server` yalnızca veri taşıyorsa şu satır sorun değildir:

```java
config.server().port();
```

Hem verisini dışarı açan hem davranış taşıyan hibrit sınıflar iki tarafın kötü yanını alır. Alanları dışarıdan değiştirilebildiği için kendi kurallarını koruyamazlar.

Kaynak: Robert C. Martin, *Clean Code*, 6. bölüm.
