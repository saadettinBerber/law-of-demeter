| Ne yapılacak? | Yapıldı mı? |
|---|---|
| [Kuralı C ve f üzerinden tanımla; nesneyi, veri yapısını ve hibriti ayır.](README.md) | Yapıldı |
| [Raporlanması ve raporlanmaması gereken senaryoları yaz.](SCENARIOS.md) | Yapıldı |
| [Altıgen mimariyi, katmanları ve port'ları belirle.](README.md#mimari) | Yapıldı |
| [Karar bekleyen üç senaryoyu karara bağla: veri yapısının taşıdığı nesne, tutan sınıfın getter'ı, Tell-Don't-Ask'in kapsamı.](SCENARIOS.md) | Yapıldı |
| [Domain her çağrı için kararı ve nedenini versin: uygun mu, ihlal mi, hangi kural yüzünden.](SCENARIOS.md) | Yapıldı |
| [Katmanları ayrı crate'lere böl; `.class` okuyucu ile domain arasındaki tip şekli sözleşmesini kur.](AGENTS.md#mimari) | Yapıldı |
| [Arkadaş listesini TDD ile yaz: yerel değişken, argüman, lambda, `new` ve statik çağrı; döndürülen nesneyi ya da koleksiyonu değiştirmek ihlal.](SCENARIOS.md#uygun-değil) | Yapıldı |
| [Tipi veri yapısı, nesne, hibrit, değer ya da konteyner olarak sınıflandır; akıcı çağrıyı, factory ve depo sorgusu adlarını tanı.](README.md#veri-yapıları) | Yapıldı |
| [`.class` okuyucuyu yaz: alanlar, metotlar, generic imzalar, alan okuyan ve yazan metotlar.](README.md#mimari) | Yapıldı |
| [Java ayrıştırıcıyı learning test ile seç; kaynağı metot, çağrı ve ifade modeline ayrıştır.](README.md#mimari) | Yapıldı |
| [Domain kararlarını tip bilgisiyle ver: veri yapısı, değer, konteyner, akıcı zincir, factory, depo sorgusu, döngü değişkeni ve hibrit sınıf; bütün senaryolar domain testinde.](SCENARIOS.md) | Yapıldı |
| [JDK tiplerini oku; tek bozuk sınıf bütün okumayı durdurmasın.](README.md#mimari) | Yapıldı |
| [Java ayrıştırıcıdaki boşlukları kapat: anonim ve yerel sınıf gövdeleri, alan başlangıç değerleri.](README.md#mimari) | Yapıldı |
| [Generic tip argümanlarını çöz; eksik bir arayüz, davranışsız sınıfı bilinmeyen yapmasın.](README.md#veri-yapıları) | Yapıldı |
| ["Projeyi denetle" kullanım senaryosunu, port'ları ve ayrıştırıcı modelinin domain'e çevrilmesini yaz; sahte adapter'larla sına.](README.md#mimari) | Yapıldı |
| [CLI'yi ve metin raporunu bağla; uygun çağrıları kullanıcı isterse uydukları kuralla göster.](README.md#mimari) | Yapıldı |
| [Gerçek projede çıkan yanlış alarmları kapat: konteyner sonucu, kütüphane tipi, değer nesnesi, miras alınan alan.](SCENARIOS.md) | Yapıldı |
| [Gerçek bir Java projesinde dene; her yanlış alarmı yeni bir senaryoya dönüştür.](SCENARIOS.md#uygun) | Yapıldı |
| [C'nin kendi metodunun döndürdüğü nesneyi yabancı say.](SCENARIOS.md#uygun-değil) | Yapıldı |
| Generic metodun tip argümanını değer argümanından çıkar: `Arrays.stream(orders)` akışının elemanı `Order` olsun. | Yapıldı |
| Lambda parametresinin tipini hedef fonksiyonel arayüzden al; hedef tip atamadan da gelebilir: `Map<Key, Consumer<Event>> handlers = Map.of(key, e -> e.getRecord())`. | Yapıldı |
| Generic metodun sonucunu lambda ile metot referansından çıkar: `orders.stream().map(Order::getCustomer)` akışının elemanı `Customer` olsun. | Yapıldı |
| Metodun tip parametresini sınırına çevir: `<T extends Shape>` içinde `T`, `Shape` sayılsın. | Yapıldı |
| [Akışın yabancıdan ürettiği elemanı yabancı say: `map(Order::getCustomer)`.](SCENARIOS.md#uygun-değil) | Yapıldı |
| Değişken sayıda argüman alan metodu çöz: `Stream.of(first, second)`, `String.format("%s %s", a, b)`. | Yapıldı |
| İç içe generic çağrıyı çöz; içteki lambda dış akışın elemanından durum alsın: `collect(Collectors.toList())`, `collect(Collectors.groupingBy(customer -> customer.getWallet()))`. | Yapıldı |
| Değişkene atanan, döndürülen ya da kurucuya verilen lambdanın parametre tipini bul: `Consumer<Event> handler = e -> ...`, `new Worker(e -> ...)`. | Yapıldı |
| [Örnek Java projeleriyle uçtan uca sına: bütün senaryolar gerçek kodda; Maven ile derle, gerçek JDK ile CLI'den geçir, raporu beklenen çıktıyla karşılaştır.](examples) | Yapıldı |
| [Paket içi görünür ile `protected` metotları da davranış say; yalnızca `private` metotlar sayılmasın.](SCENARIOS.md#uygun-değil) | Yapıldı |
| [Metot referansını, eşdeğer lambdanın yaptığı çağrı gibi yargıla: `Customer::getWallet`, `customer -> customer.getWallet()` ile aynı kararı alsın; iç içe çağrıda da: `collect(Collectors.groupingBy(Customer::getWallet))`.](SCENARIOS.md#uygun-değil) | Yapıldı |
| Depo sorgusunu tip adından da tanı: adı `Repository` ile biten tipteki alanın `get` çağrısı da depo sorgusu sayılsın. | Yapılmadı |
| Aşırı yükleme seçiminde kutulama ile genişletmeyi destekle: `int` argüman `long` ya da `Integer` parametresine uysun. | Yapılmadı |
| Değişmez argümanın tipini kullan: `Map.of("created", handler)` içinde `K`, `String` olsun. | Yapılmadı |
| Blok gövdeli lambdada son `return` dışındaki `return` ifadelerinin tipini de kullan. | Yapılmadı |
| İç içe generic çağrının maliyetini sınırla: her iç içe seviye yargılamayı ikiye katlıyor. | Yapılmadı |
| Rapor metninde statik sabiti metot çağrısı gibi yazma: `PAID() statik çağrısı` yerine `PAID statik sabiti`. | Yapılmadı |
| Gruplama anahtarının durumunu sınıflandırıcıdan al: `groupingBy(Order::getCustomer)` haritasının anahtarları yabancı, değerleri arkadaş olsun. | Yapılmadı |
| Değer konumundaki metot referansını da yargıla: `Function<Order, Customer> f = Order::getCustomer;`, `return this::handle;`. | Yapılmadı |
| Yıldızlı statik import ile gelen sabitleri çöz: `import static shop.Status.*;` sonrası `PAID.name()`. | Yapıldı |
| [JDK tip kataloğunu domain'den Java adapter'ına taşı: değer, konteyner, veri yapısı ve davranışsız tipleri dil versin; domain yalnızca bilinen tiplerin sınıfını alsın.](README.md#mimari) | Yapıldı |
| [Java'nın nesne modelini domain'den çıkar: gizli üst tipler (`Object`, `Enum`, `Record`), `equals`, `hashCode`, `toString`, `canEqual`, kurucu adı `<init>`, Lombok'un `toBuilder`'ı ve değiştirici metot adları dilden gelsin.](README.md#mimari) | Yapıldı |
| [Tip çıkarımı için güvenlik ağı kur: domain'deki çıkarım testlerinden Java kaynağıyla karşılığı olmayanları java-source'ta uçtan uca test olarak yaz.](SCENARIOS.md) | Yapıldı |
| [Kaynak okuyucu port'u tip şekillerini ve platformu da versin; Java adapter'ı tipleri çözebilsin.](README.md#mimari) | Yapıldı |
| [Domain modeline çözülmüş tip alanları ekle: çağrının dönüş tipi, lambda parametresinin tipi; alan doluysa domain onu kullansın, boşsa kendi çıkarımını sürdürsün.](README.md#mimari) | Yapıldı |
| [Generic olmayan çağrının dönüş tipini adapter bulsun: üst tiplerde ve kök tipte arama, argüman sayısına göre metot.](README.md#mimari) | Yapılmadı |
| [Generic sınıfın tip argümanlarını adapter yerine koysun: `List<Order>.get()` → `Order`.](README.md#mimari) | Yapılmadı |
| [Aşırı yükleme seçimini adapter yapsın: argüman tiplerine göre doğru metot.](README.md#mimari) | Yapılmadı |
| [Değişken sayıda argümanı adapter çözsün: `Stream.of(a, b)`, `String.format("%s %s", a, b)`.](README.md#mimari) | Yapılmadı |
| [Generic metodun tip değişkenini adapter çıkarsın: argümandan, beklenen tipten ve sınırından (`<T extends Shape>`).](README.md#mimari) | Yapılmadı |
| [Lambda parametresinin tipini adapter bulsun: fonksiyonel arayüzün tek soyut metodundan; hedef tip atamadan, dönüşten ya da kurucu parametresinden.](README.md#mimari) | Yapılmadı |
| [Metot referansının tipini adapter çözsün: `Order::getCustomer`, `this::handle`, kurucu referansı.](README.md#mimari) | Yapılmadı |
| [İç içe generic çağrının tiplerini adapter çözsün: `collect(Collectors.groupingBy(...))`.](README.md#mimari) | Yapılmadı |
| [Veri akışı bağlarını adapter versin: hangi lambda parametresi alıcının elemanlarıyla besleniyor, lambda sonucu dönüşe akıyor mu; domain arkadaş ya da yabancı durumunu bu bağlar boyunca taşısın.](README.md#mimari) | Yapılmadı |
| [Tip çıkarımını domain'den sil: `invocation.rs`, `functional.rs`, aşırı yükleme ve generic bağlama; domain testleri çözülmüş tiplerle sadeleşsin.](README.md#mimari) | Yapılmadı |
| [Domain'in hiçbir dili bilmediğini kural olarak yaz: AGENTS.md ve README mimari bölümü.](AGENTS.md#mimari) | Yapılmadı |
