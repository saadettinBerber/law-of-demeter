# Rust değişebilirliği ve değer anlamı

**Öneri:** Yazma yetkisi, belirli bir işlemin etkisi ve tipin değer anlamı ayrı değerlendirilsin. `&self`, `&mut self`, eşitlik/hash veya yeniden bağlanmayan alan bilgisi tek başına değişmezlik kanıtı olmasın. İlk Rust diliminde proje tipleri için otomatik `Value` çıkarımı ertelenmeye devam etsin; eksik bilgi açıkça `Unknown` kalsın.

İncelenen sürüm `e1b370e`, tarih 23 Kasım 2026. Bu çalışma [karar listesindeki](../decision.md) değişebilirlik araştırmasıdır. Üretim kodu, önceki belgeler ve karar listesi değiştirilmedi; Rust adaptörü veya etki çözümleyicisi uygulanmadı. Rust Reference dil davranışını tanımlar, Law of Demeter için değer politikası seçmez.

## Çekirdekte bugün ne var?

[`Members::is_value_object`](../crates/core/src/domain/classification.rs) tam üst sınıf bilgisi, en az bir örnek alanı, bütün örnek alanlarında `is_final`, setter yokluğu ve platformun istediği açık eşitlik metotlarıyla değer kararı verir. Alanın gösterdiği nesneyi veya metot etkisini incelemez. Değer kontrolü davranış/hibrit kararından önce gelir; hesaplayan bir metot bu sezgiyi engellemez.

[`FieldShape`](../crates/core/src/domain/type_shape.rs) yalnız alan adı/tipi, `is_static` ve `is_final` taşır. [`class-file`](../crates/class-file/src/shape.rs), `FINAL` bayrağını doğrudan aktarır. Bu alan Rust'taki `&self`, değişmez bağlama veya eşitlik türetmesiyle doldurulmamalıdır. [`MethodShape::writing`](../crates/core/src/domain/type_shape.rs) ise tek parametreyi alana yazan setter tanımına katılır; genel “bir yerlerde yazıyor” etkisini taşımak için kullanılmamalıdır.

[`Platform`](../crates/core/src/domain/classification/platform.rs) eşitlik metotlarını ad/parametre sayısıyla tanır; gövde veya trait kimliğiyle değer anlamını kanıtlamaz. [`java_platform`](../crates/java-source/src/platform.rs) bazı değişebilir standart tipleri de açıkça `Value` sayar. Bu istisnalar mevcut ürün politikasıdır; `Value` bütün dillerde fiziksel değişmezlik demek değildir.

Karar çağrıları etkiler: [`stranger_verdict`](../crates/core/src/domain/method/call.rs) yabancı alıcıda `Value` çağrısını serbest bırakır. Konteyner değişikliği ise platformun metot adı kurallarından geçer. Bu yüzden yanlış değer muafiyeti, korunması gereken nesne ilişkisini gizleyebilir. Mevcut [değer testleri](../crates/core/src/domain/classification/tests.rs) final alan, setter, eşitlik ve kalıtım sonuçlarını korur; bu araştırma onları değiştirmedi veya yeniden çalıştırmadı.

## Yeniden kullanılacak kanıt

Önceki öğrenme örnekleri kopyalanmadı veya yeniden çalıştırılmadı:

| Soru | Onaylanmış kanıt |
|---|---|
| `&self` değişmez mi? | [`rust-classification.rs`](../examples/rust-classification.rs): `equality_and_hash_allow_mutation_through_a_shared_reference` ve `a_refcell_can_change_state_through_a_shared_reference`. Paylaşılan erişim iç durumu değiştirebilir. |
| `&mut self` kesin yazıyor mu? | Aynı dosyada `a_mutable_receiver_does_not_have_to_change_state`, 50 okur ve 50 bırakır. İmza yetki verir; işlemin onu kullanıp kullanmadığı ayrıdır. |
| Sahip olunan kopyanın yerel alanına yazmak? | `an_owned_copy_receiver_can_change_locally_and_preserve_the_original`, yeni sonucu 2, çağıranın ilk değerini 1 doğrular. Yerel yazma mevcut takma referanstan gözlenen ortak durum değişikliği değildir. `Status::close` da yerinde enum değeri yenilemesidir; tek başına nesne kimliği veya değer uygunsuzluğu göstermez. |
| Paylaşılan sahiplik ve iç durum? | [`rust-semantics.rs`](../examples/rust-semantics.rs) içindeki `cloning_a_shared_handle_keeps_the_same_object`, `Rc` ile paylaşılan cüzdandaki değişikliğin ilk tutamaçtan da görüldüğünü sınar. Yeni `Arc<Mutex<_>>` örneği aşağıdadır. |
| Eşitlik/hash değişebilirliği önlüyor mu? | [`rust-classification.rs`](../examples/rust-classification.rs) içindeki `derived_equality_and_hash_allow_state_changing_methods` tersini gösterir. Elle yazılmış ve üretilmiş eşitliğin anlamı ayrıca çözülmelidir. |

Dil dayanağı: [iç durum değişikliği](https://doc.rust-lang.org/reference/interior-mutability.html), [referans yetkileri](https://doc.rust-lang.org/std/primitive.reference.html). `UnsafeCell` paylaşılan referans altında değişikliğin temelidir; bu, güvenli bütün kullanım örüntülerinin veya bilinmeyen `unsafe` kodun etkisinin çözüldüğü anlamına gelmez. [UnsafeCell](https://doc.rust-lang.org/std/cell/struct.UnsafeCell.html)

## Yeni, ayırt edici deneyler

[`effects.rs`](../examples/rust-mutability/effects.rs) yalnız dil/kütüphane öğrenme testidir. Hiçbir test bir analiz sınıflandırması etiketini doğrulamaz. Kaynaklar canlı resmî belgelerdir; sürüm kanıtı yerel Rust 1.99.1 çalıştırmasına aittir.

| Soru | Gözlenen sonuç ve karşı örnek |
|---|---|
| Mutex almak veri yazmak mı? | `acquiring_a_mutex_guard_can_preserve_the_protected_value`: dönen guard tutulurken ve bırakıldıktan sonra korunan sayı 50 kalır. Karşı örnek `a_mutex_guard_can_change_balance_seen_through_an_arc`: aynı tahsisi paylaşan `Arc` tutamacından guard alınır; yazmadan önce 50, yazdıktan sonra ilk tutamaçtan 30 görülür. |
| RwLock yazma guard'ı kesin değiştiriyor mu? | `acquiring_an_rwlock_write_guard_can_preserve_the_value`: sayı 50 kalır. `a_returned_rwlock_write_guard_can_change_shared_balance`: dönen guard'a atama yapılınca ilk paylaşılan referansın gösterdiği cüzdanda 30 okunur. Guard dönüşüyle yazma ayrı adımlardır. |
| Rc/Arc paylaşımı içerik değişikliği mi? | İki `cloning_*` testi, sahip sayısının 1'den 2'ye çıktığını, tutamaçların aynı tahsisi gösterdiğini ve saklanan sayıların `(50, 50)` kaldığını doğrular. Sahip sayısı gözlenebilir metadata'dır; saklanan sayı değişikliği değildir. Bu, `Rc`/`Arc` içeriğinin her kullanımda değişmez olduğunu kanıtlamaz. |
| Dönen `&mut` neyi gösterir? | `a_returned_mutable_reference_allows_writing_the_owners_state`: referans alındıktan sonra yazmadan önce 50 kaydedilir; sonra 30 yazılır ve ödünç süresi bittikten sonra sahibinden 30 okunur. Aynı anda yaşayan paylaşılan ve özel ödünçlere izin verildiği iddia edilmez. |
| Dönen içten değişebilir tutamaç? | `a_returned_shared_handle_exposes_mutation_without_changing_the_balance_on_return`: `Rc<RefCell<_>>` dönmesi bakiyeyi 50 bırakır. Tutamaç üzerinden sonraki yazma, önceden var olan paylaşılan referanstan 30 olarak görülür. Yeni bir tutamaç, bağımsız iç nesne demek değildir. |
| Önbellek mantıksal gözlemi değiştirmek zorunda mı? | `warming_a_cache_changes_administrative_state_without_changing_the_logical_observation`: `CachedName` önbelleği önce boş, ilk sorgudan sonra doludur. İlk ve önbellekten okunan uzunluk 3, metin `Ada` kalır. Önbelleği boş eş örnekle eşitlik ve hash aynıdır. |

[Mutex](https://doc.rust-lang.org/std/sync/struct.Mutex.html), [MutexGuard](https://doc.rust-lang.org/std/sync/struct.MutexGuard.html), [RwLock](https://doc.rust-lang.org/std/sync/struct.RwLock.html) ve [RwLockWriteGuard](https://doc.rust-lang.org/std/sync/struct.RwLockWriteGuard.html) erişim ve bırakma mekanizmasını açıklar. “Korunan sayı değişmedi” sonucu, kilit durumunun hiç değişmediği veya çağrının hiçbir yan etkisi olmadığı iddiası değildir. Örnekler tek iş parçacığında çalışır; her guard sonraki kilitlemeden önce bırakılır. Uyku, yarış veya iç içe kilit alma yoktur; zehirlenme hataları `Result` ile taşınır.

[Rc](https://doc.rust-lang.org/std/rc/struct.Rc.html) ve [Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html) kopyaları sahiplik metadatasını günceller. Bu işlem, iç veriye yazma yetkisi veya gerçekleşmiş proje durumu değişikliğiyle karıştırılmamalıdır. Benzersiz sahipte `get_mut` gibi başka yollar olabilir; burada bütün API için değişmezlik kanıtı verilmedi. Aynı şekilde [RwLockReadGuard](https://doc.rust-lang.org/std/sync/struct.RwLockReadGuard.html) ile doğrudan korunan sayıya yazılamaması, iç tipi `Cell` olan her okuma guard'ının derin değişmez olduğu anlamına gelmez.

`CachedName` eşitlik/hash'i yalnız metne göre tanımlar. [Hash sözleşmesi](https://doc.rust-lang.org/std/hash/trait.Hash.html) eşit değerlerin eş hash vermesini ister; fiziksel değişmezlik istemez. Test hash'in belirli bir sayısını veya bütün olası girdileri doğrulamaz. `is_cached` bilerek yönetim durumunu gözlemleyen bir metottur: bütün dış gözlemler aynı değildir. Değişen önbelleği “mantıksal olarak önemsiz” saymak bir ürün kararı gerektirir; örneğe otomatik `Value` verilmez.

## Önerilen sınır ve sonlu ilk yaklaşım

İzlenecek esas soru, mevcut bir nesneye ait **paylaşılan proje verisine veya nesne ilişkisine hangi erişimden yazılabildiği ve belirli işlemin ne yaptığı** olsun. Bütün erişilebilir baytları değişmez istemek sahip sayısını ve kilit metadatasını da kapsar. Yalnız `Eq`'nun seçtiği mantıksal alanları izlemek ise projedeki başka gözlemleri veya ilişkileri kaçırabilir. Bu yüzden fiziksel etki kaydedilsin; mantıksal önem ve LoD kararı ona otomatik bağlanmasın.

| Bilgi | Ayrı tutulacak anlam |
|---|---|
| Fiziksel değişiklik | Önbellek alanı, korunan sayı veya sahip sayacı değişmiş olabilir. Bu olguların hedefi ve kapsamı farklıdır. |
| Yazma yetkisi | Dönen `&mut`, guard veya içten değişebilir tutamaç sonraki çağırana bir yol açar. Dönüş sırasında bu yolun kullanıldığı sonucu çıkmaz. |
| İşlem etkisi | Bilinen gövdede korunan veriye atama veya çözülmüş bir yazma çağrısı ayrı kanıttır. İmza, metot adı veya yetkinin varlığı bu kanıtın yerine geçmez. |
| Mantıksal değer ve LoD ilişkisi | Önbelleğin önemsenmemesi, hangi gözlemlerin değeri tanımladığı ve yabancı nesne ilişkilerinin korunması ürün politikasıdır. Sayısal atama tek başına nesne ilişkisinin açılması değildir. |

İlk uygulama için sınırlı öneri:

1. Yalnız çözülmüş hedef ve mevcut gövde/standart işlem sözleşmesiyle yerel kanıt üret. Sayısal okuma ve yerel hesap, bağlı alana yazma, belirli dönüş yetkisi ve bu deneylerdeki standart işlemler ayrı değerlendirilebilir. Gözlenen tek çalıştırma, bütün yolların etkisi yok demek değildir.
2. İşlem etkisinde **bilinen yazma**, **belirtilen kapsamda yazmama kanıtı** ve **`Unknown`** ayrı kalsın. Yetki bilgisinde de bilinen mevcut/yok/`Unknown` ayrımı korunsun. “Yazmıyor” bilgisi hangi veri ve gövde yollarını kapsadığını söylemeli; genel yan etkisizlik anlamına gelmemeli.
3. Kilit alma/guard döndürme, korunan sayıya yazma diye etiketlenmesin. Varsayılan Rc/Arc'nin paylaşılan sayıyı taşıyan kopyası, metadata değişti diye otomatik proje verisi yazması sayılmasın. Proje sahip sayısını veya kilit durumunu iş verisi olarak kullanıyorsa bu ayrıca değerlendirilir; önemsenmediği varsayılmaz.
4. Çözülemeyen dış çağrı, eksik gövde, açılmamış makro, bilinmeyen dinamik hedef veya izlenemeyen `unsafe`/ham işaretçi etkisi **`Unknown`** kalsın. Kayıtlı bilgi bulunmaması “yazmıyor” veya “yazma yetkisi yok” demek değildir. Bunlar için çalıştırılmış bir etki çözümleyicisi yoktur.
5. Yerel kopyaya yazma, yerinde değer yenileme, paylaşılan korunan veriye yazma ve önbellek güncellemesi ayrı kanıt olsun. Bütün yazmaları veya bütün içten değişebilir tipleri `Value` dışına atan genel kural kurulmasın. Buna karşılık yeterli değer kanıtı yokken otomatik proje `Value` muafiyeti de verilmesin.

Bu `Unknown`, doğrudan `Classification::Unknown` sonucu demek değildir. Davranışı bilinen bir tip nesne olarak değerlendirilebilir; bilinmeyen etki yalnız başına değer muafiyetini kanıtlamaz. Bir işlemin sınırlı kapsamda yazmaması bütün tipin değişmez olduğunu göstermez. Önbellek/metadata için mantıksal istisna, saf işlem garantisi ve derin durum kanıtı bu çalışmada kurulmadı.

Nesne kökeni ayrıca korunur: yeni guard/tutamaç, taşıdığı mevcut nesneyi çağıranın oluşturduğu nesne yapmaz. [Görünürlük araştırmasındaki](rust-visibility.md) alıcı tipi/temsil modülü ayrımı ve adı yazılamayan tip üzerinden üye erişimi geçerlidir. Etkinin veya yetkinin bulunması tek başına LoD ihlali ya da hibritlik kararı değildir. Java'nın getter, akıcı çağrı ve konteyner yazma soruları açık kalır.

## En küçük sözleşme ihtiyacı

[`Call::returning`/`returning_from`](../crates/core/src/domain/method/call.rs), mevcut tip/argüman akışı ve yardımcı dönüş köprüleri önce kullanılmalı. Bunlar etkinin yokluğunu veya değişmezliği kanıtlamaz. `TypeShape`'e kilit adı, Rust referans sözdizimi veya bir genel etki grafiği eklemek gerekmez.

| Mevcut bilginin yetmediği yer | Gerekirse eklenecek dil bağımsız olgu |
|---|---|
| Guard/handle verilmesi ile korunan veriye yazma farklıdır. | Bağlı hedefi bilinen dönüşün yazma yetkisi ve çağrının o hedef üzerindeki yazma etkisi ayrı, eksikliği belirten bilgiler olmalıdır. `MethodShape::writing` bu iki bilgiden biri yerine kullanılmamalıdır. |
| Final bayrağı ve eşitlik varken erişilebilir durumun etkisi bilinmeyebilir. | Yeni sınıflandırma yolunda açık `Unknown` durum bilgisi eski değer sezgisinden ayrı taşınabilmelidir. Bunun yokluğu eski Java davranışını koruyabilir; açık eksiklik ise değer kanıtına dönüştürülmemelidir. Tam bir etki özeti şeması burada tasarlanmadı. |

İlk Rust diliminde `is_final` uydurmamak ve proje değer çıkarımını açmamak için yeni alan şart değildir. Yukarıdaki ihtiyaçlar gerçek entegrasyon senaryosu geldiğinde yalnız eklemeyle karşılanmalı. Adaptör dil bilgisini ve kanıtın kapsamını sağlar; son `Value`/`Object`/`Hybrid` kararını seçmez. Standart tipin veya trait'in yalnız adıyla eşleştirme yapılmaz.

## Sonraki dar kabul testi ve açık kararlar

Öneri: **`explicit_unknown_state_evidence_does_not_grant_a_value_exemption`**. Çağıran arayüzünden, eski sözleşmede bütün örnek alanları final olan, gerekli eşitlik metotlarını ve bir hesaplama davranışını taşıyan tip kurulsun. Alanın yeniden bağlanmaması, gösterdiği durumun değişmezliğiyle eşitlenmesin; yeni yol için açık durum kanıtı `Unknown` olsun. `Types::classify` sonucu `Value` yerine `Object` beklensin. Setter veya yeni hibrit kararı gerekmiyor.

Önce yalnız ek bilgi derlenebilir biçimde taşınsın. Test eksik API/kurulum nedeniyle değil, eski değer kestirmesi hâlâ çalıştığı için kırılmalı; sonra en az kararla yeşile geçmeli. Ek bilgi verilmediğinde mevcut Java final/eşitlik testi aynı sonucunu korumalı. Bu öneri `Unknown` için sınır testidir; ne `KnownStable` kanıtı ne etki analiz altyapısı uygular. Önceki [sınıflandırma](rust-classification.md#sonraki-odaklı-kabul-testi) ve [görünürlük](rust-visibility.md#sonraki-odaklı-kabul-testi) kabul testleri ayrı kalır.

Açık kararlar: değeri tanımlayan gözlemler; yönetim/önbellek durumunun hangi koşullarda önemsenmeyeceği; kaçan yazma yetkisinin hibritlikteki yeri; gövde yolları ve dış çağrılar için kanıt kapsamı; eksik analiz raporu. Genel metot seçimi, taşıma/ödünç kökeni, makro çözümü ve üretim uygulaması bu yolculuğun işi değildir.

## Doğrulama

Taban macOS ARM64, Rust 1.99.1, rustfmt 1.10.0-stable, clippy 0.1.99. 2015, 2018, 2021 ve 2024 edition'larının her birinde **9 yeni test geçti**, atlanan yok. Dört edition'da dört `.rs` dosyasının biçimi ve derlenebilir test hedefinin lint'i geçti. Olumsuz örnekler lint başarısı sayılmadı. Yeniden çalıştırma, repo kökünde:

```sh
mkdir -p target/rust-mutability
for edition in 2015 2018 2021 2024; do
    rustc --edition="$edition" --test -D warnings examples/rust-mutability/effects.rs -o "target/rust-mutability/effects-$edition" || exit 1
    "target/rust-mutability/effects-$edition" || exit 1
    rustfmt --edition "$edition" --check examples/rust-mutability/*.rs examples/rust-mutability/denied/*.rs || exit 1
    clippy-driver --edition="$edition" --test -D warnings examples/rust-mutability/effects.rs -o "target/rust-mutability/clippy-effects-$edition" || exit 1
done
```

Üç kalıcı olumsuz örnek dört edition'da yalnız amaçlanan hata koduyla reddedildi: **3 ret/edition**, toplam 12.

| Dosya | Beklenen hata ve sınırı |
|---|---|
| [`rc_direct_write.rs`](../examples/rust-mutability/denied/rc_direct_write.rs) | `E0594`: Rc üzerinden doğrudan korunan sayıya atama için `DerefMut` yoktur. `get_mut` veya iç durum değişikliği olmadığı söylenmez. |
| [`rwlock_read_guard_write.rs`](../examples/rust-mutability/denied/rwlock_read_guard_write.rs) | `E0594`: okuma guard'ı üzerinden doğrudan korunan sayıya atama yapılamaz. Deneyin iç tipi sayıdır. |
| [`overlapping_borrows.rs`](../examples/rust-mutability/denied/overlapping_borrows.rs) | `E0502`: canlı paylaşılan ödünç kullanılırken aynı cüzdanın `&mut` alan dönüşü alınamaz. |

`edition` dört edition'dan biri, `sample` tablodaki uzantısız ad olacak şekilde kullanılan komut:

```sh
rustc --edition="$edition" -D warnings --error-format=json "examples/rust-mutability/denied/$sample.rs" -o "target/rust-mutability/denied-$sample-$edition" 2> "target/rust-mutability/denied-$sample-$edition.jsonl"
```

Komut başarısız olmalı ve JSON hata kodu tabloyla eşleşmelidir; kurulum hatası kabul edilen ret değildir. Bu örnekler dil öğrenme kanıtıdır, üretim davranışı için kırmızı TDD değildir.

İlk koşuda yeşil olan dokuz test, 2024 edition'ında toplam **11 geçici bozulmayı** assertion ile yakaladı. Kilit/ref/handle döndüren örnek erişimciye yanlış yazma eklendi; sayısal yazma sonucu bozuldu; Rc/Arc kopyası bağımsız tahsisle değiştirildi. Handle'ın bağımsız kopya dönmesi ve önbelleğin hiç dolmaması/yanlış uzunluk saklaması da ayrı sınandı. Beklentiler değişmedi; bütün geçici kaynaklar `-D warnings` ile derlendi. Önbelleğin ilk sonucu doğru kalıp sonraki sonucu bozulduğunda da test kırıldı.

Geçici kaynaklar ve kayıtlar `target/rust-mutability/faults` altında tutuldu. Her `sample` ve ilgili test adıyla kullanılan komutlar:

```sh
rustc --edition=2024 --test -D warnings "target/rust-mutability/faults/$sample.rs" -o "target/rust-mutability/faults/$sample"
"target/rust-mutability/faults/$sample" --exact "$test_name"
```

Bu denetim örneklerin gözlemlerini korur; Rust derleyicisinin değiştirildiğini veya bütün hata yollarının sınandığını göstermez. İlk biçim kontrolünde uzun test imzaları edition'lar arasında farklı düzenlendi; adlar anlamı koruyarak kısaltıldı, ardından dört edition kontrolü geçti. Önceki araştırma örnekleri, çekirdek/Java testleri ve geniş workspace/Maven koşuları yeniden çalıştırılmadı. Bütün çıktılar bu worktree'nin `target/rust-mutability` dizininde kaldı.
