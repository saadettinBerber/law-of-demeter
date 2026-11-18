Yapılan bir satır, işin karşılığı başka bir belgeye bakmadan testlerden anlaşılıyorsa kaldırılır.

| Ne yapılacak? | Yapıldı mı? |
|---|---|
| [Hesaplayan getter'ı ve alanı sarıp döndüren getter'ı karara bağla: erişimci mi sayılsın, davranış mı kalsın? Bugün tek bir `getFullName()` sınıfı veri yapısı olmaktan çıkarıyor; `Collections.unmodifiableList(lines)` döndüren getter da öyle.](SCENARIOS.md#uygun-değil) | Yapılmadı |
| [Akıcı çağrıyı karara bağla: yalnızca zincirin devamında mı serbest olsun? Bugün `order.getBuilder().add(line)` serbest, aynı alıcıda `order.getBuilder().isEmpty()` ihlal.](SCENARIOS.md#uygun) | Yapılmadı |
| [f'nin oluşturduğu nesnenin alanını ve C'nin başka bir örneğinin `private` alanını karara bağla. Bugün statik fabrikanın oluşturduğu örneği doldurması ihlal: `index.byOwner.computeIfAbsent(owner, key -> new HashSet<>()).add(member)`.](SCENARIOS.md#uygun-değil) | Yapılmadı |
| [Veri yapısına yazmayı tek karara bağla. Bugün setter serbest (`request.getAddress().setCity(city)`), koleksiyona eklemek ihlal (`request.getItems().add(item)`).](SCENARIOS.md#uygun) | Yapılmadı |
| [Kütüphane nesnesinin verdiği koleksiyonu değiştirmeyi karara bağla. Bugün `builder.environment().put(key, value)` ihlal; Uygun 11 ile Uygun değil 4 çakışıyor.](SCENARIOS.md#uygun-değil) | Yapılmadı |
| [İhlal satırında alıcının tipini ve neden veri yapısı sayılmadığını göster; hibrit satırında davranışın geldiği üst sınıfı göster.](README.md#kullanım) | Yapılmadı |
| [Rust kurulu olmayan makine için sürüm ikilisi ya da `Dockerfile` sağla.](README.md#kullanım) | Yapılmadı |
