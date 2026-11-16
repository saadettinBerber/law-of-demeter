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

- Her senaryo, Java kaynağından karara uçtan uca bir testle sınanır. Ayrıca [`examples/shop`](examples/shop) örnek projesinde gerçek kod olarak durur; proje Maven ile derlenir, araçtan geçirilir ve rapor [beklenen raporla](examples/shop/expected-report.txt) karşılaştırılır.

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
| Uygun değil | 9. Kendi metodunun döndürdüğü nesneyle konuşmak | Raporlanır | ✅ |
| Uygun değil | 10. Akışın yabancıdan ürettiği elemanla konuşmak | Raporlanır | ✅ |
| Uygun değil | 11. Yabancıyla metot referansıyla konuşmak | Raporlanır | ✅ |
| Uygun değil | 12. Gruplanan anahtarla konuşmak | Raporlanır | ✅ |
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

- **Görünürlük:** Sınıf ya da metotlar `public` olmasa da sonuç aynı. Paket içi görünür ve `protected` metotlar da davranış sayılır; yalnızca `private` metotlar sayılmaz. Örnek: [`Reconciler.java:5`](examples/shop/src/main/java/shop/ledger/Reconciler.java#L5).

- **Örnek projede:** [`PaymentService.java:13`](examples/shop/src/main/java/shop/service/PaymentService.java#L13)

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

- **Örnek projede:** [`PaymentService.java:17-18`](examples/shop/src/main/java/shop/service/PaymentService.java#L17-L18)

</details>

<details>
<summary><strong>3. Getter'ın döndürdüğü nesneyi setter ile değiştirmek</strong></summary>

```java
void move(Order order, Address address) {
    order.getCustomer().setAddress(address);
}
```

- **Neden:** f, sahibine sormadan başka bir nesneye ait parçayı değiştiriyor. `Order` kendi kurallarını uygulayamıyor.

- **Örnek projede:** [`OrderService.java:52`](examples/shop/src/main/java/shop/service/OrderService.java#L52)

</details>

<details>
<summary><strong>4. Getter'ın döndürdüğü koleksiyonu değiştirmek</strong></summary>

```java
void addLine(Order order, OrderLine line) {
    order.getLines().add(line);
}
```

- **Neden:** Koleksiyon `Order`'ın iç yapısı. Satır sayısı sınırı gibi bir kural varsa atlanıyor.

- **Örnek projede:** [`OrderService.java:56`](examples/shop/src/main/java/shop/service/OrderService.java#L56)

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

- **Örnek projede:** [`Account.java:6-17`](examples/shop/src/main/java/shop/domain/Account.java#L6-L17), [`RefundService.java:30`](examples/shop/src/main/java/shop/service/RefundService.java#L30)

</details>

<details>
<summary><strong>6. İç koleksiyonun elemanlarını dışarıdan değiştirmek</strong></summary>

```java
void discount(Order order, int rate) {
    order.getLines().forEach(line -> line.applyDiscount(rate));
}
```

- **Neden:** f, `Order`'ın iç nesnelerine tek tek uzanıyor. İndirim kuralı `Order`'ın dışına dağılıyor.

- **Örnek projede:** [`OrderService.java:60`](examples/shop/src/main/java/shop/service/OrderService.java#L60)

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

- **Örnek projede:** [`ShippingService.java:16`](examples/shop/src/main/java/shop/service/ShippingService.java#L16)

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

- **Örnek projede:** [`CheckoutService.java:16`](examples/shop/src/main/java/shop/service/CheckoutService.java#L16)

</details>

<details>
<summary><strong>9. Kendi metodunun döndürdüğü nesneyle konuşmak</strong></summary>

```java
void discount(String id, int rate) {
    findLine(id).applyDiscount(rate);
}
```

- **Neden:** `findLine` C'nin kendi metodu; onu çağırmak serbest. Döndürdüğü `OrderLine` ise C'nin alanı da f'nin argümanı da değil. f satıra doğrudan uzanıyor.

- **Not:** Kural metodun gövdesine bakmaz; döndürülen nesne bir alandan gelse de yabancı sayılır. Adı `find` ile başlasa da depo sorgusu sayılmaz, çünkü depo sorgusunun alıcısı bir alan ya da parametre olmalı: Uygun 9.

- **Örnek projede:** [`DiscountService.java:14`](examples/shop/src/main/java/shop/service/DiscountService.java#L14)

</details>

<details>
<summary><strong>10. Akışın yabancıdan ürettiği elemanla konuşmak</strong></summary>

```java
void charge(int amount) {
    orders.stream().map(Order::getCustomer).forEach(customer -> customer.getWallet().withdraw(amount));
}
```

- **Neden:** `orders` C'nin bir alanı; akışı da arkadaş. `map(Order::getCustomer)` ise her siparişin müşterisini döndürür ve müşteriler yabancıdır. f, `order.getCustomer().getWallet()` ile aynı yere uzanıyor; akış bunu gizlemez. `getWallet` ve `withdraw` raporlanır.

- **Not:** Lambda ile yazılsa da sonuç aynı: `map(order -> order.getCustomer())`. Eleman f'nin içinde oluşturulursa arkadaş kalır: `map(order -> new Receipt(order))`. `filter` gibi elemanı değiştirmeyen adımlar akışın durumunu korur: Uygun 5.

```java
void discountLines(Order order) {
    new Batch<>(order.getLines(), line -> line.applyDiscount(10));
}
```

- **Neden (kurucu):** `Batch(List<T> items, Consumer<T> each)` lambdanın girdisini `items`'ın elemanlarından alır. Elemanları `getLines()` çağrısı verdiği için yabancıdır; lambda yeni oluşturulan `Batch`'i değil, o elemanları alır. `applyDiscount` raporlanır. Aynısı statik çağrıda da geçerlidir: `Batches.each(order.getLines(), line -> ...)`.

- **Not (kurucu):** Lambdanın girdisini başka bir argüman beslemiyorsa lambda, çağrının alıcısına bağlı kalır; kurucuda bu, oluşturulan nesnedir: `new Worker(e -> ...)`. Metot referansı da aynı yoldan beslenir: `new Batch<>(order.getLines(), OrderLine::getId)`. Kurucu ya da metot belirlenemezse bağ kurulmaz.

- **Örnek projede:** [`BillingService.java:15`](examples/shop/src/main/java/shop/service/BillingService.java#L15), [`BillingService.java:19`](examples/shop/src/main/java/shop/service/BillingService.java#L19), [`ReportingService.java:71`](examples/shop/src/main/java/shop/service/ReportingService.java#L71)

</details>

<details>
<summary><strong>11. Yabancıyla metot referansıyla konuşmak</strong></summary>

```java
List<Wallet> wallets() {
    return orders.stream().map(Order::getCustomer).map(Customer::getWallet).toList();
}
```

- **Neden:** `Customer::getWallet`, `customer -> customer.getWallet()` lambdasıyla aynı çağrıyı yapar. Akışın elemanları `getCustomer()` çağrısından gelen yabancılardır; yabancının metodunu referansla çağırmak da kuralı çiğner. `getWallet` raporlanır. `Order::getCustomer` ise arkadaş olan siparişlerde çağrıldığı için uygundur.

- **Not:** İç içe çağrıda da sonuç aynı: `collect(Collectors.groupingBy(Customer::getWallet))`. `wallet::withdraw` gibi bağlı referans `wallet.withdraw(...)` gibi, `Optional::of` gibi statik referans statik çağrı gibi yargılanır. `Order::new` çağrı sayılmaz. Değişkene atanan ya da döndürülen referans da eşdeğer lambda gibi yargılanır: `return event.getReceipt().getOrder()::ship;`, `e -> e.getReceipt().getOrder().ship()` ile aynı ihlalleri verir.

- **Örnek projede:** [`ReportingService.java:44`](examples/shop/src/main/java/shop/service/ReportingService.java#L44), [`EventDispatcher.java:39`](examples/shop/src/main/java/shop/service/EventDispatcher.java#L39)

</details>

<details>
<summary><strong>12. Gruplanan anahtarla konuşmak</strong></summary>

```java
void rename(List<Order> orders, String prefix) {
    for (Customer buyer : orders.stream().collect(Collectors.groupingBy(Order::getCustomer)).keySet()) {
        buyer.rename(prefix);
    }
}
```

- **Neden:** Harita anahtarlarını sınıflandırıcı üretir; burada `Order::getCustomer`, yani anahtarlar `getCustomer()` çağrısından gelen yabancılar. f, `order.getCustomer().rename(prefix)` ile aynı yere uzanıyor; gruplama bunu gizlemez. `rename` raporlanır.

```java
void ship(List<Order> orders) {
    for (Map.Entry<Customer, Order> entry : orders.stream().collect(Collectors.toMap(Order::getCustomer, order -> order)).entrySet()) {
        entry.getKey().getWallet();
        entry.getValue().ship();
    }
}
```

- **Neden (girdiler):** `toMap(keyMapper, valueMapper)` anahtarları anahtar eşleyicinin, değerleri değer eşleyicinin sonucundan alır. `getKey()` anahtarların durumunu okur: `Order::getCustomer` yabancı verdiği için `getWallet` raporlanır. `getValue()` değerlerin durumunu okur: `order -> order` elemanı olduğu gibi verdiği için `ship` arkadaşla konuşur, raporlanmaz. Değer eşleyici `Order::getCustomer` olursa `getValue()` ve `values()` yabancı verir; örnek projedeki `renameByOrder` bu yüzden `values()` ile gelen müşteriyi raporlar.

- **Not:** Haritanın değerleri akışın elemanlarından gelir ve elemanların durumunu korur: `grouped.get(customer).get(0).getStatus()` arkadaş olan siparişle konuşur. Anahtarın durumu yalnızca anahtarları veren çağrıdan okunur (`keySet()`, `getKey()`), değerin durumu yalnızca değerleri veren çağrıdan (`values()`, `getValue()`, `get`); `entrySet()` ve onu süren çağrılar (`stream()` gibi) girdilerin ikisini de taşır. Haritanın kendisi değişmez.

- **Örnek projede:** [`ReportingService.java:53`](examples/shop/src/main/java/shop/service/ReportingService.java#L53), [`ReportingService.java:59`](examples/shop/src/main/java/shop/service/ReportingService.java#L59), [`ReportingService.java:65-66`](examples/shop/src/main/java/shop/service/ReportingService.java#L65-L66)

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

- **Örnek projede:** [`OrderService.java:22`](examples/shop/src/main/java/shop/service/OrderService.java#L22)

</details>

<details>
<summary><strong>2. Bir değer üzerinde işlem yapmak</strong></summary>

```java
Instant expiry(Instant issuedAt, Duration ttl) {
    return issuedAt.plus(ttl).truncatedTo(ChronoUnit.SECONDS);
}
```

- **Neden:** Zaman, süre ve sayı gibi değerlerin üzerinde işlem yapmak bir iç yapıya uzanmak değildir.

- **Örnek projede:** [`OrderService.java:26`](examples/shop/src/main/java/shop/service/OrderService.java#L26)

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

- **Örnek projede:** [`Connector.java:7`](examples/shop/src/main/java/shop/service/Connector.java#L7), [`ShippingService.java:20`](examples/shop/src/main/java/shop/service/ShippingService.java#L20), [`ShippingService.java:24`](examples/shop/src/main/java/shop/service/ShippingService.java#L24)

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

- **Örnek projede:** [`OrderService.java:34`](examples/shop/src/main/java/shop/service/OrderService.java#L34), [`Order.java:72-101`](examples/shop/src/main/java/shop/domain/Order.java#L72-L101)

</details>

<details>
<summary><strong>5. Java koleksiyonları ve stream'ler</strong></summary>

```java
long paidCount(List<Order> orders) {
    return orders.stream().filter(Order::isPaid).count();
}
```

- **Neden:** Stream adımları koleksiyonu dönüştürür; bir nesnenin iç yapısına uzanmaz.

- **Not:** `map` elemanı bir yabancıyla değiştirirse akışın elemanları da yabancı olur: Uygun değil 10.

- **Örnek projede:** [`BillingService.java:27`](examples/shop/src/main/java/shop/service/BillingService.java#L27)

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

- **Örnek projede:** [`ReportService.java:7`](examples/shop/src/main/java/shop/service/ReportService.java#L7), [`ReportService.java:11`](examples/shop/src/main/java/shop/service/ReportService.java#L11)

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

- **Örnek projede:** [`RefundService.java:26`](examples/shop/src/main/java/shop/service/RefundService.java#L26)

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

- **Örnek projede:** [`RefundService.java:21-22`](examples/shop/src/main/java/shop/service/RefundService.java#L21-L22)

</details>

<details>
<summary><strong>9. Depo sorgusunun döndürdüğü nesneyle konuşmak</strong></summary>

```java
void dispatch(OrderId id) {
    orders.findById(id).ship();
}
```

- **Neden:** Depo sorgusu, factory gibi f'ye kullanması için bir nesne verir. Dönen sipariş f'nin arkadaşı sayılır.

- **Not:** `orders` C'nin bir alanı. Adı `find`, `load` ya da `fetch` ile başlayan metot depo sorgusu sayılır; adı `Repository` ile biten tipte `get` ile başlayan metot da: `orders.getById(id).ship()`. Alıcı bir alan ya da parametre olmalı.

- **Örnek projede:** [`OrderService.java:38`](examples/shop/src/main/java/shop/service/OrderService.java#L38), [`OrderService.java:64`](examples/shop/src/main/java/shop/service/OrderService.java#L64)

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

- **Örnek projede:** [`CheckoutService.java:20`](examples/shop/src/main/java/shop/service/CheckoutService.java#L20), [`CheckoutService.java:24`](examples/shop/src/main/java/shop/service/CheckoutService.java#L24)

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

- **Örnek projede:** [`PriceCache.java:16`](examples/shop/src/main/java/shop/service/PriceCache.java#L16)

</details>

<details>
<summary><strong>12. Değişmez değer nesnesiyle konuşmak</strong></summary>

```java
boolean compatible(Order order, Version required) {
    return order.getVersion().isCompatibleWith(required);
}
```

- **Neden:** `Version`'ın bütün alanları final, setter'ı yok, `equals` ve `hashCode`'u tanımlı; bir değer nesnesi. Bir karşılaştırma metodu taşıması onu nesne yapmaz.

- **Örnek projede:** [`OrderService.java:30`](examples/shop/src/main/java/shop/service/OrderService.java#L30), [`Version.java:6-12`](examples/shop/src/main/java/shop/domain/Version.java#L6-L12)

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

- **Örnek projede:** [`PaymentService.java:22`](examples/shop/src/main/java/shop/service/PaymentService.java#L22), [`BaseService.java:6`](examples/shop/src/main/java/shop/service/BaseService.java#L6)

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

- **Örnek projede:** [`OrderService.java:42`](examples/shop/src/main/java/shop/service/OrderService.java#L42)

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

- **Örnek projede:** [`OrderService.java:46-47`](examples/shop/src/main/java/shop/service/OrderService.java#L46-L47)

</details>
