# Senaryolar

---

**Kuralın raporlaması ve raporlamaması gereken durumlar.**

- Her örnek, C sınıfındaki f metodudur.

- Her senaryo ileride kuralın bir test vakasına dönüşür.

## Özet

---

**Bu repo hangi senaryoyu karşılıyor?**

- ✅ Kural bu senaryoda beklendiği gibi davranıyor.

- ❌ Henüz davranmıyor. Kural henüz yazılmadığı için şimdilik bütün senaryolar ❌.

- — Karar verilmeden beklenen davranış belli değil.

| Bölüm | Senaryo | Beklenen | Durum |
|---|---|---|---|
| Uygun değil | 1. Getter'ın döndürdüğü nesnenin metodunu çağırmak | Raporlanır | ❌ |
| Uygun değil | 2. Zinciri yerel değişkene bölmek | Raporlanır | ❌ |
| Uygun değil | 3. Getter'ın döndürdüğü nesneyi setter ile değiştirmek | Raporlanır | ❌ |
| Uygun değil | 4. Getter'ın döndürdüğü koleksiyonu değiştirmek | Raporlanır | ❌ |
| Uygun değil | 5. Hibrit sınıf | Raporlanır | ❌ |
| Uygun değil | 6. İç koleksiyonun elemanlarını dışarıdan değiştirmek | Raporlanır | ❌ |
| Uygun | 1. Parametreden değer okumak | Raporlanmaz | ❌ |
| Uygun | 2. Bir değer üzerinde işlem yapmak | Raporlanmaz | ❌ |
| Uygun | 3. Veri yapısında zincirle okumak | Raporlanmaz | ❌ |
| Uygun | 4. Builder ve fluent zincirler | Raporlanmaz | ❌ |
| Uygun | 5. Java koleksiyonları ve stream'ler | Raporlanmaz | ❌ |
| Uygun | 6. Factory'ler ve yeni nesneler | Raporlanmaz | ❌ |
| Uygun | 7. Yabancıyı yalnızca başka bir metoda iletmek | Raporlanmaz | ❌ |
| Uygun | 8. Kendi nesnesi ve alanları | Raporlanmaz | ❌ |
| Karar bekleyen | 1. Davranışı olan bir nesneyi taşıyan olay | Karara bağlı | — |
| Karar bekleyen | 2. Bir nesneyi tutan sınıf: depo mu, sahip mi | Karara bağlı | — |
| Karar bekleyen | 3. Durumu okuyup kararı dışarıda vermek | Karara bağlı | — |
| Kapsam dışı | 1. Bir nesnenin hesabını dışarıda yapmak | Raporlanmaz | ❌ |

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

## Karar bekleyen

---

**Bu durumların sonucu tasarıma bağlı.**

<details>
<summary><strong>1. Davranışı olan bir nesneyi taşıyan olay</strong></summary>

```java
void onShipped(OrderShippedEvent event) {
    log(event.getOrder().getTotal());
}
```

- **Olay `Order`'ı taşırsa:** `Order` davranışı olan bir nesne olduğu için bu okuma kuralı çiğner.

- **Olay değerleri kopyalarsa:** `event.getTotal()` yeterli olur. Olay bir veri yapısına dönüşür ve sorun kalmaz.

</details>

<details>
<summary><strong>2. Bir nesneyi tutan sınıf: depo mu, sahip mi</strong></summary>

```java
void checkout(Session session) {
    session.getCart().clear();
}
```

- **`Session` yalnızca saklıyorsa:** `getCart()` bir depodan okuma gibidir. Sepeti alıp onunla konuşmak serbest.

- **`Session` sepetin sahibiyse:** Sepetin kuralları `Session`'a aittir. f bunu sahibine söylemeli: `session.checkout()`.

</details>

<details>
<summary><strong>3. Durumu okuyup kararı dışarıda vermek</strong></summary>

```java
void ship(Order order) {
    if (order.getStatus() == Status.PAID) {
        order.markShipped();
    }
}
```

- **Law of Demeter açısından:** Uygun. `order` f'nin arkadaşı ve durum bir değer.

- **Tell-Don't-Ask açısından:** Karar `Order`'ın içinde olmalı: `order.ship()`.

- **Karar:** Kural yalnızca Law of Demeter'e mi bakacak, Tell-Don't-Ask ihlallerini de mi raporlayacak?

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
