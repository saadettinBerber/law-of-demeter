# Senaryolar

---

**Kuralın raporlaması ve raporlamaması gereken durumlar.**

- Her örnek, C sınıfındaki f metodudur.

- Her senaryo ileride kuralın bir test vakasına dönüşür.

- "Raporlanmaz" senaryolar varsayılan raporda görünmez. Kullanıcı isterse, uydukları kuralla birlikte "Uygun" olarak gösterilir.

## Özet

---

**Bu repo hangi senaryoyu karşılıyor?**

- ✅ Kural bu senaryoda beklendiği gibi davranıyor.

- ❌ Henüz davranmıyor.

- Her senaryo, Java kaynağından karara uçtan uca bir testle sınanır.

| Bölüm | Senaryo | Beklenen | Durum |
|---|---|---|---|
| Uygun değil | 1. Getter'ın döndürdüğü nesnenin metodunu çağırmak | Raporlanır | ✅ |
| Uygun değil | 2. Zinciri yerel değişkene bölmek | Raporlanır | ✅ |
| Uygun değil | 3. Getter'ın döndürdüğü nesneyi setter ile değiştirmek | Raporlanır | ✅ |
| Uygun değil | 4. Getter'ın döndürdüğü koleksiyonu değiştirmek | Raporlanır | ✅ |
| Uygun değil | 5. Hibrit sınıf | Raporlanır | ✅ |
| Uygun değil | 6. İç koleksiyonun elemanlarını dışarıdan değiştirmek | Raporlanır | ✅ |
| Uygun değil | 7. Veri yapısının taşıdığı nesneye uzanmak | Raporlanır | ✅ |
| Uygun değil | 8. Tutan sınıfın getter'ıyla alınan nesneyle konuşmak | Raporlanır | ✅ |
| Uygun | 1. Parametreden değer okumak | Raporlanmaz | ✅ |
| Uygun | 2. Bir değer üzerinde işlem yapmak | Raporlanmaz | ✅ |
| Uygun | 3. Veri yapısında zincirle okumak | Raporlanmaz | ✅ |
| Uygun | 4. Builder ve fluent zincirler | Raporlanmaz | ✅ |
| Uygun | 5. Java koleksiyonları ve stream'ler | Raporlanmaz | ✅ |
| Uygun | 6. Factory'ler ve yeni nesneler | Raporlanmaz | ✅ |
| Uygun | 7. Yabancıyı yalnızca başka bir metoda iletmek | Raporlanmaz | ✅ |
| Uygun | 8. Kendi nesnesi ve alanları | Raporlanmaz | ✅ |
| Uygun | 9. Depo sorgusunun döndürdüğü nesneyle konuşmak | Raporlanmaz | ✅ |
| Uygun | 10. Arkadaşın koleksiyonundan alınan elemanla konuşmak | Raporlanmaz | ✅ |
| Uygun | 11. Kütüphane nesnesiyle konuşmak | Raporlanmaz | ✅ |
| Uygun | 12. Değişmez değer nesnesiyle konuşmak | Raporlanmaz | ✅ |
| Uygun | 13. Üst sınıftan gelen alanla konuşmak | Raporlanmaz | ✅ |
| Kapsam dışı | 1. Bir nesnenin hesabını dışarıda yapmak | Raporlanmaz | ✅ |
| Kapsam dışı | 2. Durumu okuyup kararı dışarıda vermek | Raporlanmaz | ✅ |

## Uygun değil

---

**Bu durumlar raporlanmalı.**

<details>
<summary><strong>1. Getter'ın döndürdüğü nesnenin metodunu çağırmak</strong></summary>

```java
void pay(Order order, int amount) {
    order.getCustomer().getWallet().withdraw(amount);
}
```

- **Neden:** `getCustomer()`'ın döndürdüğü müşteri f'nin arkadaşı değil. f, müşterinin ve cüzdanının yapısını bilmek zorunda kalıyor.

</details>

<details>
<summary><strong>2. Zinciri yerel değişkene bölmek</strong></summary>

```java
void pay(Order order, int amount) {
    Customer customer = order.getCustomer();
    customer.getWallet().withdraw(amount);
}
```

- **Neden:** Zinciri bölmek ilişkiyi değiştirmez. `customer` hâlâ başka bir nesneden alınmış bir yabancı.

</details>

<details>
<summary><strong>3. Getter'ın döndürdüğü nesneyi setter ile değiştirmek</strong></summary>

```java
void move(Order order, Address address) {
    order.getCustomer().setAddress(address);
}
```

- **Neden:** f, sahibine sormadan başka bir nesneye ait parçayı değiştiriyor. `Order` kendi kurallarını uygulayamıyor.

</details>

<details>
<summary><strong>4. Getter'ın döndürdüğü koleksiyonu değiştirmek</strong></summary>

```java
void addLine(Order order, OrderLine line) {
    order.getLines().add(line);
}
```

- **Neden:** Koleksiyon `Order`'ın iç yapısı. Satır sayısı sınırı gibi bir kural varsa atlanıyor.

</details>

<details>
<summary><strong>5. Hibrit sınıf</strong></summary>

```java
@Getter
@Setter
class Account {
    private int balance;

    void withdraw(int amount) {
        if (amount > balance) {
            throw new IllegalStateException("Insufficient balance");
        }
        balance -= amount;
    }
}
```

- **Neden:** Bütün alanlar dışarıdan değiştirilebiliyor. `setBalance()` ile `withdraw()` içindeki kontrol atlanabilir.

- **Not:** Getter ve setter'lar Lombok yerine elle yazılsa da durum aynı.

- **Kütüphane üst sınıfı:** Setter'lar kütüphanedeki bir üst sınıftan geliyorsa sınıf hibrit sayılmaz. Karar yalnızca projenin kendi metotlarına bakar.

</details>

<details>
<summary><strong>6. İç koleksiyonun elemanlarını dışarıdan değiştirmek</strong></summary>

```java
void discount(Order order, int rate) {
    order.getLines().forEach(line -> line.applyDiscount(rate));
}
```

- **Neden:** f, `Order`'ın iç nesnelerine tek tek uzanıyor. İndirim kuralı `Order`'ın dışına dağılıyor.

</details>

<details>
<summary><strong>7. Veri yapısının taşıdığı nesneye uzanmak</strong></summary>

```java
void onShipped(OrderShippedEvent event) {
    log(event.getOrder().getTotal());
}
```

- **Neden:** `OrderShippedEvent` bir veri yapısı, ama taşıdığı `Order` davranışı olan bir nesne. Veri yapısı yalnızca kendi alanlarını açar; içindeki nesneye uzanmak 1. senaryoyla aynı ihlal.

- **Not:** Olay değeri kendisi taşırsa sorun kalmaz: `event.getTotal()`.

</details>

<details>
<summary><strong>8. Tutan sınıfın getter'ıyla alınan nesneyle konuşmak</strong></summary>

```java
void checkout(Session session) {
    session.getCart().clear();
}
```

- **Neden:** `getCart()` ile alınan sepet `Session`'ın bir parçası. f sepete doğrudan uzanıyor; `Session`'ın sepet üzerindeki kuralları atlanabilir.

- **Not:** Kural, `Session`'ın sepeti yalnızca saklayıp saklamadığını bilemez. Getter ile ulaşılan nesneyi her zaman yabancı sayar. f bunu sahibine söylemeli: `session.checkout()`. Depo sorgusu ayrı bir durum: Uygun 9.

</details>

## Uygun

---

**Bu durumlar raporlanmamalı.**

<details>
<summary><strong>1. Parametreden değer okumak</strong></summary>

```java
boolean canShip(Order order) {
    return order.getStatus() == Status.PAID;
}
```

- **Neden:** `order` f'nin arkadaşı. Enum, sayı, metin ve zaman gibi değerler davranış taşımaz; onları okumak kuralı çiğnemez.

</details>

<details>
<summary><strong>2. Bir değer üzerinde işlem yapmak</strong></summary>

```java
Instant expiry(Instant issuedAt, Duration ttl) {
    return issuedAt.plus(ttl).truncatedTo(ChronoUnit.SECONDS);
}
```

- **Neden:** Zaman, süre ve sayı gibi değerlerin üzerinde işlem yapmak bir iç yapıya uzanmak değildir.

</details>

<details>
<summary><strong>3. Veri yapısında zincirle okumak</strong></summary>

```java
int portOf(Config config) {
    return config.server().port();
}

String cityOf(ShippingRequest request) {
    return request.getAddress().getCity();
}

String keyOf(Customer customer) {
    return customer.getId().getValue();
}
```

- **Neden:** Bu tiplerin hiçbiri davranış taşımıyor; gizlenecek bir iç yapıları yok.

  - `Config` ve `Server` record.

  - `ShippingRequest` ve `Address` yalnızca getter ve setter taşıyan DTO'lar.

  - `CustomerId` bir değer nesnesi.

- **Not:** Lombok `@Data` ile yazılmış DTO'lar da bu gruba girer.

</details>

<details>
<summary><strong>4. Builder ve fluent zincirler</strong></summary>

```java
Order create(String id, int total) {
    return Order.builder().id(id).total(total).build();
}
```

```java
private Order(Builder builder) {
    this.id = builder.id;
    this.total = builder.total;
}
```

- **Neden:** Builder'ın her adımı aynı builder'ı döndürür; zincir tek bir nesneyle konuşur.

- **Constructor için:** Okunan alanlar aynı dosyadaki builder'a ait.

</details>

<details>
<summary><strong>5. Java koleksiyonları ve stream'ler</strong></summary>

```java
long paidCount(List<Order> orders) {
    return orders.stream().filter(Order::isPaid).count();
}
```

- **Neden:** Stream adımları koleksiyonu dönüştürür; bir nesnenin iç yapısına uzanmaz.

</details>

<details>
<summary><strong>6. Factory'ler ve yeni nesneler</strong></summary>

```java
String report(ReportFactory factory) {
    return factory.create().render();
}

String greeting(String name) {
    return new StringBuilder().append("Hello ").append(name).toString();
}
```

- **Neden:** Factory'nin ve `new`'in döndürdüğü nesneler f'nin içinde oluşturulmuş sayılır; bunlar f'nin arkadaşı.

</details>

<details>
<summary><strong>7. Yabancıyı yalnızca başka bir metoda iletmek</strong></summary>

```java
void notifyCustomer(Order order) {
    notifier.send(order.getCustomer());
}
```

- **Neden:** f, müşterinin hiçbir metodunu çağırmıyor; onu yalnızca `notifier`'a veriyor.

- **Not:** `notifier` C'nin bir alanı.

</details>

<details>
<summary><strong>8. Kendi nesnesi ve alanları</strong></summary>

```java
void pay(int amount) {
    wallet.withdraw(amount);
    this.audit();
}
```

- **Neden:** `wallet` C'nin bir alanı, `audit()` C'nin kendi metodu.

</details>

<details>
<summary><strong>9. Depo sorgusunun döndürdüğü nesneyle konuşmak</strong></summary>

```java
void dispatch(OrderId id) {
    orders.findById(id).ship();
}
```

- **Neden:** Depo sorgusu, factory gibi f'ye kullanması için bir nesne verir. Dönen sipariş f'nin arkadaşı sayılır.

- **Not:** `orders` C'nin bir alanı. Adı `find`, `load` ya da `fetch` ile başlayan metot depo sorgusu sayılır; alıcı bir alan ya da parametre olmalı.

</details>

<details>
<summary><strong>10. Arkadaşın koleksiyonundan alınan elemanla konuşmak</strong></summary>

```java
void checkout(String id) {
    carts.get(id).checkout();
}
```

- **Neden:** `carts` C'nin bir alanı. Koleksiyondan alınan eleman koleksiyonun durumunu korur; arkadaşın koleksiyonundaki eleman da arkadaştır.

- **Not:** Döndürülen bir koleksiyondan alınan eleman yabancı kalır. `order.getLines().get(0).applyDiscount(rate)` raporlanır.

</details>

<details>
<summary><strong>11. Kütüphane nesnesiyle konuşmak</strong></summary>

```java
void update() {
    lock.writeLock().lock();
}
```

- **Neden:** `writeLock()`'un döndürdüğü nesne projenin değil, JDK'nın tipi. Kural projenin kendi tasarımına bakar; kütüphane tiplerinin yapısı projede değiştirilemez.

- **Not:** Proje tipi, denetlenen kaynaklarda tanımlanan tiptir. Çok modüllü projede bütün modüllerin kaynakları verilmeli; yoksa diğer modüllerin tipleri kütüphane sayılır.

</details>

<details>
<summary><strong>12. Değişmez değer nesnesiyle konuşmak</strong></summary>

```java
boolean compatible(Order order, Version required) {
    return order.getVersion().isCompatibleWith(required);
}
```

- **Neden:** `Version`'ın bütün alanları final, setter'ı yok, `equals` ve `hashCode`'u tanımlı; bir değer nesnesi. Bir karşılaştırma metodu taşıması onu nesne yapmaz.

</details>

<details>
<summary><strong>13. Üst sınıftan gelen alanla konuşmak</strong></summary>

```java
class PaymentService extends BaseService {
    void pay(int amount) {
        wallet.withdraw(amount);
    }
}
```

- **Neden:** `wallet`, `BaseService`'te tanımlı bir alan. C'nin üst sınıfından gelen alan, C'nin kendi alanı gibi arkadaştır.

</details>

## Kapsam dışı

---

**Bu durumlar başka bir kuralın konusu; bu kural raporlamaz.**

<details>
<summary><strong>1. Bir nesnenin hesabını dışarıda yapmak</strong></summary>

```java
int totalOf(Order order) {
    return order.getPrice() * order.getQuantity();
}
```

- **Neden:** f yalnızca arkadaşının değerlerini okuyor; Law of Demeter çiğnenmiyor.

- **Asıl sorun:** Hesap `Order`'a ait: `order.total()`. Bu durumun adı feature envy.

</details>

<details>
<summary><strong>2. Durumu okuyup kararı dışarıda vermek</strong></summary>

```java
void ship(Order order) {
    if (order.getStatus() == Status.PAID) {
        order.markShipped();
    }
}
```

- **Neden:** `order` f'nin arkadaşı ve durum bir değer; Law of Demeter çiğnenmiyor.

- **Asıl sorun:** Karar `Order`'ın içinde olmalı: `order.ship()`. Bu durumun adı Tell-Don't-Ask. Bulmak tahmine dayandığı ve çok yanlış alarm ürettiği için bu kural raporlamaz.

</details>
