Yapılan bir satır, işin karşılığı başka bir belgeye bakmadan testlerden anlaşılıyorsa kaldırılır.

| Ne yapılacak? | Yapıldı mı? |
|---|---|
| [Kuralı C ve f üzerinden tanımla; nesneyi, veri yapısını ve hibriti ayır.](README.md) | Yapıldı |
| [Raporlanması ve raporlanmaması gereken senaryoları yaz.](SCENARIOS.md) | Yapıldı |
| [Altıgen mimariyi, katmanları ve port'ları belirle.](README.md#mimari) | Yapıldı |
| [Karar bekleyen üç senaryoyu karara bağla: veri yapısının taşıdığı nesne, tutan sınıfın getter'ı, Tell-Don't-Ask'in kapsamı.](SCENARIOS.md) | Yapıldı |
| [Gerçek bir Java projesinde dene; her yanlış alarmı yeni bir senaryoya dönüştür.](SCENARIOS.md#uygun) | Yapıldı |
| Generic metodun tip argümanını değer argümanından çıkar: `Arrays.stream(orders)` akışının elemanı `Order` olsun. | Yapıldı |
| Lambda parametresinin tipini hedef fonksiyonel arayüzden al; hedef tip atamadan da gelebilir: `Map<Key, Consumer<Event>> handlers = Map.of(key, e -> e.getRecord())`. | Yapıldı |
| Generic metodun sonucunu lambda ile metot referansından çıkar: `orders.stream().map(Order::getCustomer)` akışının elemanı `Customer` olsun. | Yapıldı |
| Metodun tip parametresini sınırına çevir: `<T extends Shape>` içinde `T`, `Shape` sayılsın. | Yapıldı |
| Değişken sayıda argüman alan metodu çöz: `Stream.of(first, second)`, `String.format("%s %s", a, b)`. | Yapıldı |
| İç içe generic çağrıyı çöz; içteki lambda dış akışın elemanından durum alsın: `collect(Collectors.toList())`, `collect(Collectors.groupingBy(customer -> customer.getWallet()))`. | Yapıldı |
| Değişkene atanan, döndürülen ya da kurucuya verilen lambdanın parametre tipini bul: `Consumer<Event> handler = e -> ...`, `new Worker(e -> ...)`. | Yapıldı |
| Depo sorgusunu tip adından da tanı: adı `Repository` ile biten tipteki alanın `get` çağrısı da depo sorgusu sayılsın. | Yapıldı |
| Aşırı yükleme seçiminde kutulama ile genişletmeyi destekle: `int` argüman `long` ya da `Integer` parametresine uysun. | Yapıldı |
| Değişmez argümanın tipini kullan: `Map.of("created", handler)` içinde `K`, `String` olsun. | Yapıldı |
| Blok gövdeli lambdada son `return` dışındaki `return` ifadelerinin tipini de kullan. | Yapıldı |
| İç içe generic çağrının maliyetini sınırla: her iç içe seviye yargılamayı ikiye katlıyor. | Yapıldı |
| Rapor metninde statik sabiti metot çağrısı gibi yazma: `PAID() statik çağrısı` yerine `PAID statik sabiti`. | Yapıldı |
| Gruplama anahtarının durumunu sınıflandırıcıdan al: `groupingBy(Order::getCustomer)` haritasının anahtarları yabancı, değerleri arkadaş olsun. | Yapıldı |
| Değer konumundaki metot referansını da yargıla: `Function<Order, Customer> f = Order::getCustomer;`, `return this::handle;`. | Yapıldı |
| Yıldızlı statik import ile gelen sabitleri çöz: `import static shop.Status.*;` sonrası `PAID.name()`. | Yapıldı |
| [Tip çıkarımını domain'den sil: `invocation.rs`, `functional.rs`, aşırı yükleme ve generic bağlama; domain testleri çözülmüş tiplerle sadeleşsin.](README.md#mimari) | Yapıldı |
