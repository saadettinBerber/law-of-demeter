# Rust desteği

Rust kaynaklarını yeni bir adaptör okuyacak; Law of Demeter kararını mevcut çekirdek verecek. Bu belge kapsamı, araştırma sonuçlarını ve doğrulanmış sınırları kaydeder. Yapılacak işler [karar tablosunda](../decision.md) durur.

## Kapsam

İlk doğrulama tabanı Rust/Cargo **1.99.1**, macOS ARM64 ve Cargo projeleridir. 2015, 2018, 2021 ve 2024 edition'ları hedeflenir; edition her paketin kendi `Cargo.toml` bilgisinden alınır. Bir derleyici sürümünde geçen örnek, diğer sürümlerin doğrulandığı anlamına gelmez. Rust edition'ları paketler arasında birlikte kullanılabilir. [Rust Edition Guide](https://doc.rust-lang.org/edition-guide/editions/)

| Konu | Karar |
|---|---|
| Proje | Tek paket ve Cargo workspace'i; seçilen paketlerin kütüphane ve program hedefleri. |
| Test ve örnek | Test, örnek ve benchmark hedefleri açıkça seçilirse denetlenir; `cfg(test)` normal kütüphane bağlamına karıştırılmaz. |
| Derleme bağlamı | Seçilen özellikler, hedef platform, edition ve bağımlılık kimlikleri analiz boyunca korunur. Bir yapılandırmanın sonucu bütün özellik birleşimlerinin sonucu sayılmaz. |
| Proje sınırı | Workspace içindeki tipler proje tipi kalır. Rapor filtresi, tip çözümlemek için gereken kaynakları eksiltmez. Dış bağımlılık ayrı tanımlanır. |
| Standart kütüphane | `std` kullanan ve `no_std` kütüphaneler hedeflenir. `core` ve isteğe bağlı `alloc` çözülür. Özel hedef dosyaları ve kurulu olmayan hedefler ilk sürümün doğrulanmış kapsamına girmez. |
| Makro ve üretilmiş kod | Seçili yapılandırmanın açılımları hedeflenir. `build.rs` ve prosedürel makroları çalıştırmanın gerekip gerekmediği seçilecek altyapıya göre bildirilir. Açılmayan kod eksik analizdir. |
| Nightly | Kararsız dil özellikleri ilk kapsamda yoktur. Rust analizinin kurulumu için nightly gerektiren adayın bu maliyeti ayrıca değerlendirilir. |
| Tek başına `.rs` dosyası | Cargo bağlamı olmayan dosya ilk ürün kapsamına girmez; bağımlılıkları ve derleme koşulları tahmin edilmez. Araştırma örnekleri doğrudan `rustc` ile çalıştırılabilir. |
| Hatalı veya desteklenmeyen kod | Atlanan bölüm görünür olmalı. Eksik analizden kesin uygunluk sonucu çıkarılmaz; rapor ve çıkış kodu ayrı karar adımıdır. |
| `unsafe` ve dış dil | Rust tarafındaki bilinen çağrılar incelenebilir. Ham işaretçinin veya dış fonksiyonun bilinmeyen nesne ilişkileri çözümlenmiş sayılmaz. |

Bu tablo hedef kapsamdır. Rust adaptörü henüz uygulanmadı; mevcut CLI hâlâ Java içindir. Altyapı deneyinin gösterdiği sınırlar ilgili karar satırında açık tutulacak.

## Çekirdekle sınır

`TypeSource` tip şeklini ve platform bilgisini, `SourceReader` kaynak dosyasını ve metot modelini verir. `CheckProject` bunları birleştirip kuralları çalıştırır. Java ayrıştırıcısını yeniden kullanmak gerekmez. [Portlar](../crates/core/src/application/ports.rs), [kullanım senaryosu](../crates/core/src/application/check_project.rs)

Dil adının üretim kodunda geçmemesi, modelin başka bir dile eksiksiz uyduğunu kanıtlamıyor. İncelenen sürüm `132284baa980d9025f5dff2d5569a6de4e4a1adb`; aşağıdaki sınırlar bu koda aittir.

| Mevcut davranış | Rust için gereken ayrım |
|---|---|
| [`Method`](../crates/core/src/domain/method.rs) her fonksiyona `class_name` ve dolaylı olarak `this` bağlamı veriyor. | Bağımsız fonksiyonun kapsayıcı modülü ile gerçek metot alıcısı ayrı temsil edilmeli. Modül sahte bir nesneye dönüştürülmemeli. |
| [`TypeKind::Enum`](../crates/core/src/domain/classification.rs) doğrudan değer, `Record` doğrudan veri yapısı sayılıyor. | Rust enum'u nesne taşıyıp davranış uygulayabilir; her `struct` veri yapısı değildir. Türün sözdizimi tek başına karar vermemeli. |
| Sınıflandırıcı `private` ve statik metotları davranış saymıyor. | Rust görünürlüğü modül sınırıdır. Çağrılabilirlik, metot alıcısı ve davranışa katkı ayrı bilgiler olmalı. |
| Değer nesnesi kararı `final` alan ve platformun eşitlik metotlarına bağlı. | Paylaşılan referans ve eşitlik trait'i değişmezlik kanıtı değildir. İç durum değiştirme bilgisi ayrıca gerekir. |
| [`FieldShape`](../crates/core/src/domain/type_shape.rs) alanın dışarıya açık olup olmadığını taşımıyor; hibrit kararı setter arıyor. | `pub` alanla davranışın birlikte bulunması setter olmadan da ele alınmalı. Alan okuması, alan yazması ve davranış ayrı gözlemler olmalı. |
| Koleksiyon elemanı son tür parametresinden alınıyor. | `Result<T, E>` başarı değeri `T`'dir; iterator öğesi ilişkili `Item` tipidir. İçeriğin tipi ve hangi girişten geldiği açık taşınmalı. |
| Akıcı metot, ad/argüman sayısı ve kendi tipini döndürmesiyle bulunuyor. | Aynı adlı trait metotları farklı olabilir. `Self`, `&Self` ve başka bir örneği döndürme arasında fark var; çözülmüş çağrı bilgisi gerekir. |
| [Tip üzerinden çağrının sonucu](../crates/core/src/domain/method/call.rs) arkadaş sayılıyor. | `<Type as Trait>::method(&value)` bir alıcı çağrısı olabilir. Bağımsız fonksiyon da mevcut nesneyi döndürebilir; sözdizimi yeni nesne kanıtı değildir. |
| [`Expression::Operation`](../crates/core/src/domain/method/expression.rs) işlem sonucunu literal arkadaş sayıyor. | `match`, blok veya `?` ile alınan nesne bu yoldan geçirilirse köken kaybolabilir. Dalların birleşimi ayrı değerlendirilmelidir. |
| Yardımcı metot eşlemesi ad, sınıf ve satıra; arama aynı dosyanın metotlarına bağlı. | Farklı dosyalardaki `impl` blokları ve aynı adlı trait metotları için çözümlenmiş hedef kimliği gerekir. |
| Proje tipleri başarıyla okunan dosyaların bildirimlerinden çıkarılıyor. | Okunamayan dosyanın tipi dış kütüphane sayılıp serbest bırakılamaz; proje kataloğu ve analiz eksikleri birlikte korunmalı. |

Bu bulgular henüz çekirdek değişikliği değildir. Önce bir Rust senaryosu eksikliği gösterecek; sonra yalnız gereken dil bağımsız bilgi sözleşmeye eklenecek. Ayrıştırıcı nesneleri, Cargo bilgileri ve Rust tip çözümleyicisi adaptörde kalacak.

## Sınıflandırma ve görünürlük

Rust'ta enum varyantı alan taşıyabilir. Örnekte `Account::Open(Wallet)` kendi `charge` metoduyla cüzdanı değiştiriyor. Dolayısıyla `enum` etiketini mevcut `TypeKind::Enum` kararına doğrudan çevirmek güvenilir değil. Nesne/veri/değer kararı alan ve davranış bilgisine dayanmalı. [Enum tanımı](https://doc.rust-lang.org/reference/items/enumerations.html)

Rust'ta özel üyeye bulunduğu modül ve alt modülleri erişebilir. Aynı modüldeki başka tip özel bir metodu çağırabilir. `pub(crate)` ise crate sınırına kadar erişim verir. Java'nın sınıfa bağlı özel metot varsayımı bu görünürlüklere taşınamaz. [Görünürlük](https://doc.rust-lang.org/reference/visibility-and-privacy.html)

`&self` alan bir metot `Cell`, `RefCell`, atomik değer veya kilit üzerinden durumu değiştirebilir. `&T` tek başına değişmez değer kanıtı değildir. `&mut self` de belirli bir metodun mutlaka değişiklik yaptığı anlamına gelmez. Adaptör imzayı ve gövdeyi çözer; domain açık bilgiye göre karar verir. [İç durum değişikliği](https://doc.rust-lang.org/reference/interior-mutability.html)

Değer ve hibrit sınıflandırmasının kesin Rust kuralları, açık alan ve üretilmiş trait örnekleriyle henüz tamamlanmadı. Mevcut karar tablosundaki hesaplayan getter ve koleksiyon yazma soruları da sonuçları etkiliyor; bu araştırma onları kapatmaz.

## Çağrı hedefi ve nesnenin kökeni

Metot araması otomatik referans çözme ve ödünç alma adımları içerir. Örnekte `Dispatch::select(&mut self)` bulunmasına rağmen `value.select()` çağrısı önce `Selection::select(&self)` trait metodunu seçiyor. Açık `Dispatch::select(&mut value)` farklı hedefe gidiyor. Metot adını okumak yeterli değil. [Metot seçimi](https://doc.rust-lang.org/reference/expressions/method-call-expr.html)

Bir tipe ait birden fazla `impl` olabilir. Trait uygulamasının kimliği de metot adının parçası gibi değerlendirilmelidir. Generic sınırlar ve ilişkili tiplerin çözümü domain'e taşınmayacak. [Uygulamalar](https://doc.rust-lang.org/reference/items/implementations.html)

Taşıma, eldeki değerin sahipliğini değiştirir. `Box` taşındığında işaret ettiği nesne aynı kalır. `Rc::clone` aynı nesneyi paylaşır; örnekte sahip olunan `Wallet::clone` bağımsız durum üretir. Bu iki çağrıya yalnız `clone` adı üzerinden aynı nesne-kökeni kararı verilemez. [Taşıma ve değer ifadeleri](https://doc.rust-lang.org/reference/expressions.html), [Rc](https://doc.rust-lang.org/std/rc/struct.Rc.html)

Referans üzerinden alanı ayırma, özel `Deref` uygulaması ve `move` closure mevcut nesneye erişimi sürdürebilir. Closure'ın yeni bir değer olması, içine alınan her nesnenin yeni olduğunu göstermez. [Alan erişimi](https://doc.rust-lang.org/reference/expressions/field-expr.html), [örüntüler](https://doc.rust-lang.org/reference/patterns.html), [closure yakalaması](https://doc.rust-lang.org/reference/types/closure.html)

`Result<T, E>` içinden başarı ve hata değerleri farklı yollardan çıkar. `Result::iter` başarı tarafındaki `T` üzerinde dolaşır. Iterator'ın `map` adımı ise öğe tipini ve kökenini değiştirebilir. Sarmalayıcının serbest olması, içinden çıkan proje nesnesine sınırsız erişim anlamına gelmez. [Result](https://doc.rust-lang.org/std/result/enum.Result.html), [Iterator](https://doc.rust-lang.org/std/iter/trait.Iterator.html)

## Makrolar ve derleme bağlamı

Bildirim makrosu bir metot çağrısı üretebilir. Prosedürel makro derleme sırasında kod çalıştırır ve yeni sözdizimi üretir. `build.rs` de derlemede çalışabilir. Analiz altyapısı bu işlemleri gerektiriyorsa kullanıcıya kurulum ve çalışma davranışı açıkça anlatılacak; yalnız dosya okunduğu izlenimi verilmeyecek. [Prosedürel makrolar](https://doc.rust-lang.org/reference/procedural-macros.html), [Cargo build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html)

`#[cfg(...)]` etkin olmayan öğeyi kaldırabilir; araştırma örneğinde bu öğedeki çözülemeyen adlar derlemeyi bozmuyor. `cfg!` ise koşulu bir değere dönüştürür; dal gövdelerinin otomatik olarak analiz dışı kaldığı varsayılmayacak. Seçili özellikler ve hedef platform aynı proje için farklı kod üretir. [Koşullu derleme](https://doc.rust-lang.org/reference/conditional-compilation.html)

Prosedürel makro, `build.rs`, `async` ve `no_std` için uçtan uca analiz henüz denenmedi. Bunlar sessizce kapsam dışına çıkarılmayacak; ilgili uygulama ve doğrulama satırları açık kalacak.

## Çalıştırılan kanıt

[Dil davranışı örnekleri](../examples/rust-semantics.rs) 15 test içeriyor. Rust 1.99.1 ile dört edition'da ayrı derlenip çalıştırıldı: her koşuda 15 test geçti. Geçici kopyada dört davranış kasıtlı bozulunca 15 test başarısız oldu; asıl örnek yeniden geçti. Bu sonuçlar Rust dil davranışını doğrular; henüz Rust kodundan Law of Demeter raporu üretildiğini göstermez.

Repo kökünde yeniden çalıştırma:

```sh
mkdir -p target/rust-research
rustc --edition=2024 --test -D warnings examples/rust-semantics.rs -o target/rust-research/semantics-2024
target/rust-research/semantics-2024
```

Diğer edition'lar için `--edition` ve çıktı adı değiştirilir. Bu örnekler dış crate kullanmaz. Nihai adaptör testlerine geçerken ilgili öğrenme testleri oraya taşınacak.

## Başlangıç doğrulaması

23 Kasım 2026'da, üretim kodu değiştirilmeden:

| Kontrol | Sonuç |
|---|---|
| `cargo test --workspace` | 1233 test geçti; Maven isteyen 2 örnek testi varsayılan koşuda atlandı. |
| `cargo test -p law-of-demeter-cli --test examples -- --ignored` | Atlanan 2 test ayrıca geçti; tek ve çok modüllü Java projelerinin raporları beklenen çıktıyla aynı. |
| `cargo fmt --all --check` | Geçti. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Geçti. |

İlk koşuda gerçek JDK yoktu. Öğrenme testleri macOS'taki `/usr/bin/javac` başlatıcısını JDK sanıp bulunmayan dosyaları okumaya çalıştı. Temurin 21.0.13.1+2 ve Maven 3.9.16 resmî dağıtımlardan indirilip sağlama toplamları doğrulandı; yalnız `target/verification-tools` altına açıldı. Geçen koşularda bu JDK ve Maven süreç ortamında seçildi. Sistem kurulumu ve üretim kodu değiştirilmedi.

Bu sonuçlar Rust adaptörünün kanıtı değildir. Yeni adaptör ve ortak model değişiklikleri aynı Java başlangıç sonuçlarıyla karşılaştırılacak.
