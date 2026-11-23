# Rust tip sınıflandırması

Rust `struct` ve `enum` tiplerini alanları, metotları ve durum değişiklikleriyle değerlendirmek önerilir. Sözdizimi tek başına nesne, veri yapısı, değer veya hibrit kararı vermez. Bu bir **araç politikası önerisidir**; Rust Reference bir Law of Demeter sınıflandırması tanımlamaz.

İncelenen çekirdek: `3fed6746f039ac83f0e275f3134c89bf6b6cec50`. Araştırma tarihi: 23 Kasım 2026. Üretim kodu değişmedi; Rust adaptörü ve Rust analiz raporu henüz yok. Bu belge [karar listesindeki](../decision.md) sınıflandırma araştırmasını destekler; görünürlük ve değişmezlik araştırmalarını tamamlanmış saymaz.

## Bugünkü çekirdek

| Karar | Kodun ve testin gösterdiği |
|---|---|
| Öncelik | `Platform`'un bildiği tip veya ad alanı, tip şeklinden önce gelir. Bilinen değer/konteyner üst tipi de şekil kararından önce gelir. |
| Tip şekli | Öncelikli platform/üst tip sonucu yoksa `Enum` doğrudan `Value`, `Record` davranış taşısa da `DataStructure`, `Interface` ise `Object` olur. Yalnız `Class` üyeleriyle sınıflandırılır. |
| Davranış | Özel ve statik metotlar, platformun kurucu/ortak metotları ve kendi builder'ını veren metotlar ayıklanır. Kalanlarda parametresiz alan okuma getter, tek parametreli alan yazma setter, diğerleri davranıştır. |
| Değer nesnesi | En az bir örnek alanı, tamamlanmış üst sınıf bilgisi, bütün örnek alanlarında `is_final`, setter yokluğu ve platformun istediği açık eşitlik metotları gerekir. Bu kontrol metot etkisini veya alanın içindeki nesnenin değişmezliğini incelemez. |
| Nesne/veri/hibrit | Değer olmayan sınıfta proje davranışı ve proje setter'ı birlikteyse hibrit; davranış varsa nesne; davranış yok ve üst sınıf zinciri tam ise veri yapısı olur. Eksik zincir davranış yokken `Unknown` üretir. Kütüphane üyeleri tek başına proje hibriti oluşturmaz. |
| Üretilmiş metot | `is_generated` bütün metotları davranıştan çıkarmaz. Üretilmiş erişimciler ve eşitlik metotları elle yazılanlarla aynı kurallardan geçer. |

Kaynak: [sınıflandırıcı](../crates/core/src/domain/classification.rs), [sınıflandırma testleri](../crates/core/src/domain/classification/tests.rs). Özellikle `an_enum_is_a_value`, `a_record_with_behavior_stays_a_data_structure`, `a_class_with_a_mutable_field_is_not_a_value` ve `private_methods_do_not_count_as_behavior` mevcut davranışı sabitler.

[`TypeShape`](../crates/core/src/domain/type_shape.rs) alan adı/tipi, `is_static` ve `is_final` taşır; alan görünürlüğü veya iç durum değişikliği taşımaz. `MethodShape::reading` ve `writing` gövdeyi çözümleyen adaptörün sağladığı bilgilerdir. Domain gövde ayrıştırmaz. [`class-file`](../crates/class-file/src/shape.rs) bu bilgileri erişim bayrakları ve erişimci çözümlemesinden kurar.

[`Platform`](../crates/core/src/domain/classification/platform.rs) ortak/eşitlik metotlarını ad ve parametre sayısıyla eşler; trait kimliği veya etki bilgisi taşımaz. `eq`/`hash` adlı ilgisiz bir metodu eşitlik trait'i saymak güvenilir değildir. [`java_platform`](../crates/java-source/src/platform.rs) içindeki adlar Rust platformuna kopyalanmamalı. Proje tiplerini `with_type(..., Classification::...)` ile tek tek etiketlemek sınıflandırma kuralını adaptöre taşır.

Bu ayrım çağrı kararını etkiler: [yabancı alıcı üzerindeki çağrı](../crates/core/src/domain/method/call.rs) değer ve veri yapısında serbesttir; nesne ve hibritte genel olarak ihlaldir. Kütüphane ve akıcı çağrı istisnaları ayrıca uygulanır. Tipin veri yapısı olması, taşıdığı davranışlı nesnenin kökenini değiştirmez: [Uygun değil 7](../SCENARIOS.md#uygun-değil).

## Doğrulanmış Rust gerçekleri

[`examples/rust-classification.rs`](../examples/rust-classification.rs) yalnız dil öğrenme testidir. Testler derleyiciyi ve örneğin gözlenen sonucunu sınar; hiçbirine sabit bir sınıflandırma etiketi yazılmadı. Kaynaklar canlı Reference/standart kütüphane belgeleridir; aşağıdaki sürüm kanıtı yerel Rust 1.99.1 çalıştırmasına aittir.

| Ayrım | Deney ve yakın karşı örnek | Birincil kaynak |
|---|---|---|
| `struct` veri taşıyabilir | `nested_public_fields_can_be_read_without_methods`: iki açık alandan 7 okunur. `a_public_field_can_be_written_through_a_mutable_reference`: aynı tür alana 9 yazılır. | [Struct ve alan görünürlüğü](https://doc.rust-lang.org/reference/items/structs.html), [alan erişimi](https://doc.rust-lang.org/reference/expressions/field-expr.html) |
| `struct` davranış taşıyabilir | `a_struct_method_can_change_encapsulated_state`: `Wallet::withdraw(20)` bakiyeyi 50'den 30'a indirir. `a_struct_method_can_reject_a_change_without_changing_state`: 60 çekme isteği reddedilir, bakiye 50 kalır. | [Uygulamalar](https://doc.rust-lang.org/reference/items/implementations.html) |
| Açık alan davranışın denetimini aşabilir | `a_public_field_allows_a_balance_the_method_rejects`: 60 çekmeyi reddeden `OpenWallet`'ın bakiyesi önce 50 kalır; ardından dışarıdan -10 yazılır. Test, reddin hemen sonrasındaki bakiyeyi doğrudan yazmadan önce kaydeder; setter yoktur. | [Alan erişimi](https://doc.rust-lang.org/reference/expressions/field-expr.html) |
| Alansız enum yerinde yenilenebilir | `a_fieldless_enum_method_can_query_its_variant` iki varyantı ayırır; `a_fieldless_enum_method_can_replace_its_variant`, `&mut self` ile gösterilen yere `Closed` değerini atar. Bu, nesne kimliği veya Law of Demeter ihlali göstermez. | [Enum](https://doc.rust-lang.org/reference/items/enumerations.html), [atama](https://doc.rust-lang.org/reference/expressions/operator-expr.html#assignment-expressions) |
| Alanlı enum nesne ilişkisi taşıyabilir | `DeliveryEvent` metotsuz veri taşır. `Account::Open(Wallet)` ise metotla taşıdığı cüzdanın bakiyesini değiştirir. Başka modüldeki çağıran da örüntüyle yükün tutulduğu yeri yeni cüzdanla doldurabilir. Enum varyantına ayrıca `pub` yazmak reddedilir. | [Enum alanları ve varyantlar](https://doc.rust-lang.org/reference/items/enumerations.html), [görünürlük](https://doc.rust-lang.org/reference/visibility-and-privacy.html) |
| Özel üye tip sınırı değildir | `Reader`, aynı modüldeki `Wallet`'ın özel metodunu çağırır; `Payment` özel alanına -10 yazar. Üst modülden aynı iki erişim derlenmez. `pub(crate)` alan ise kardeş modülden yazılır. | [Modül gizliliği ve kapsamlı görünürlük](https://doc.rust-lang.org/reference/visibility-and-privacy.html) |
| Eşitlik/hash alan yazmayı engellemez | `Revision`, türetilmiş `PartialEq`, `Eq`, `Hash` ile birlikte alanına yazan `advance` metoduna sahiptir. `Version::next` ise yeni değer döndürür, ilkini korur. Bu dil gerçekleri tek başına sınıflandırma kararı değildir. | [Derive](https://doc.rust-lang.org/reference/attributes/derive.html), [Hash](https://doc.rust-lang.org/std/hash/trait.Hash.html) |
| Sahip olunan kopya yerel değişebilir | `an_owned_copy_receiver_can_change_locally_and_preserve_the_original`: `Copy` olan `Version`'ın `next_owned(mut self)` metodu yerel sayıyı 1'den 2'ye çıkarır ve kendisini döndürür; çağıranın değeri 1 kalır. Sonuç, `Version::new(2)` ile kurulan değere eşittir. | [Sahip olunan metot alıcısı](https://doc.rust-lang.org/reference/items/associated-items.html#methods) |
| `&self` değişmezlik sağlamaz | `SharedWallet`, `Cell` ile 50'den 30'a iner; eşitliği türetilmiş, hash'i elle yazılmıştır. `Cell<i32>` için doğrudan `Hash` türetme ayrıca reddedilir. `RefCell` de paylaşılan referanstan yazmayı sağlar. | [İç durum değişikliği](https://doc.rust-lang.org/reference/interior-mutability.html), [Cell](https://doc.rust-lang.org/std/cell/struct.Cell.html), [RefCell](https://doc.rust-lang.org/std/cell/struct.RefCell.html) |
| `&mut self` değişiklik kanıtı değildir | `balance_with_exclusive_access` 50 döndürür ve durumu değiştirmez. Buna karşılık normal alanı `&T` üzerinden doğrudan değiştirmek derlenmez. | [İlişkili metotlar](https://doc.rust-lang.org/reference/items/associated-items.html), [iç durum değişikliği](https://doc.rust-lang.org/reference/interior-mutability.html) |

Buradaki `pub` alanların kapsayıcı tipleri `pub(super)` ile sınırlıdır; testler ayrı bir modülden erişimi gösterir, dış crate deneyinin yerini tutmaz. `Wallet`'ın gizli alanı da aynı modüldeki koddan korunmaz. Bu nedenle yalnız `pub` anahtarını aramak veya `&self` imzasını `final` alan yerine kullanmak yeterli değildir.

Atama, bir değeri bellekteki bir yere taşır veya kopyalar. `Status::close` o yerde tutulan enum değerinin yenilenmesini gösterir; bundan nesne kimliği, dışa açılmış nesne ilişkisi veya değer sınıfına uygun olmama sonucu çıkmaz. `Version::next_owned` yalnız sayısal alanı olan yerel kopyasını değiştirir. `SharedWallet` deneyinde ise aynı cüzdanın paylaşılan referansından yapılan iç değişiklik, ilk bağlamadan da gözlenir. Bu üç gözlemin araç politikasındaki karşılığı ayrı kararlaştırılmalıdır. [Atama](https://doc.rust-lang.org/reference/expressions/operator-expr.html#assignment-expressions), [metot alıcıları](https://doc.rust-lang.org/reference/items/associated-items.html#methods)

## Önerilen en küçük politika

Bu sıra Rust'a ilişkin bir dil kuralı değildir; mevcut araç yaklaşımını yeni dile taşımak için öneridir:

1. Platform yalnız kimliği çözülmüş, ayrı doğrulanmış standart tipleri tanısın. Kullanıcı tipleri için `struct` → veri yapısı, `enum` → değer eşlemesi yapılmasın.
2. Üyeleri çözümlenen proje tipinde gerçek alıcılı metotlar değerlendirilsin. Alıcısız ilişkili fonksiyon, kendi başına örnek davranışı sayılmasın. Türetilmiş olmak bütün metotları muaf tutmasın; çözümlenmiş eşitlik/hash gibi ortak işlemler ayrı tanınsın. Elle yazılmış ortak işlem durum değiştiriyorsa bu etki kaybolmasın.
3. Tam üye bilgisiyle yalnız doğrudan alan erişimi/erişimciler taşıyan tip veri yapısı olsun. Alanındaki nesne ayrı sınıflandırılsın; alanı açmak o nesne üzerinde sınırsız çağrı hakkı vermesin.
4. Davranışla birlikte doğrulanmış dış yazma yolu varsa hibrit olsun. Açık struct alanına veya enum yüküne doğrudan yazma, setter olmadan da bu kanıtı verebilir. Salt okunur getter tek başına hibrit kanıtı sayılmasın; bu mevcut çekirdeğin yaklaşımını korur.
5. Davranış var, dış yazma yolu olmadığı doğrulanmışsa nesne olsun. Üye/erişim bilgisi eksikse kesin veri yapısı veya değer muafiyeti verilmesin; belirsizlik görünür kalsın.
6. Proje tipi için değer ölçütü ayrı kararlaştırılsın. Çözülmüş eşitlik/hash bilgisi tek başına karar vermesin; paylaşılan referanstan gözlenen iç değişiklik ve dışa verilen nesne ilişkileri ayrıca incelensin. Yerinde değer yenileme, sahip olunan alıcının yerel alanına yazma ve paylaşılan durum değişikliği aynı gözlem sayılmasın. Her alıcı/alan yazmasını değer olmaya engel sayan genel bir kural önerilmez.

İlk uygulama diliminde proje tipleri için otomatik `Value` çıkarımı açılmasın. Değer yenileme, sahip olunan alıcının yerel değişikliği, paylaşılan durum ve görünürlük politikası çözülene kadar `Version`, `Revision`, `Status` ve `SharedWallet` kesin etiket almaz. Eşitlik bilgisi veya bir yazma işleminin bulunması tek başına bu kararı vermez. Bu sınırlama bazı gerçek değerleri nesne sayabilir; eksik destek açıkça belirtilmelidir.

Örneklerde beklenen politika: `Delivery`/`Address`/metotsuz `DeliveryEvent` veri yapısı; `OpenWallet` ve dışarıdan yükü yazılabilen davranışlı `Account` hibrit adayıdır. Gizli alanlı `Wallet`'ın nesne mi hibrit mi olduğu, aynı modüldeki ayrı tipten yazmanın kapsülleme sınırına nasıl katılacağına bağlıdır. Bu görünürlük kararı burada kapanmaz. `Status::close` değer yenileme deneyidir; `Status`'ın değer sayılabilirliğini tek başına çürütmez.

## Adaptörün sağlayacağı bilgi

Adaptör kaynak/tip çözümlemesini yapar; son sınıflandırmayı domain hesaplar. Ayrıştırıcı nesneleri, Rust trait adları ve modül kuralları domain'e girmez. Aşağıdaki adlar sözleşme önerisidir; henüz eklenmedi.

| Bilgi | En küçük dil bağımsız ekleme ve gerekçe |
|---|---|
| Üyeleriyle değerlendirilen bileşik tip | `TypeKind::Aggregate` gibi ek bir seçenek `struct` ve alanlı/alansız enum için `Class`'ın davranış/veri/hibrit ölçütlerini kullanabilir. Eski `final` temelli değer sezgisi bu yola aktarılmamalı. Mevcut `Enum`/`Record` sonuçları korunur. `Account`, üye ve yük bilgisi inceleme ihtiyacının somut örneğidir. |
| Dış yazma olanağı | Alanın doğrudan yazılabildiği erişim kapsamı ve çözümlemenin tamlığı eklenmeli; bilinmeyen ile yazılamayan ayrılmalı. `OpenWallet`'ta setter yoktur. `Account` yükündeki alanın varyant kimliği kaybolmamalı; sınıflandırma için alan bilgisi, sonraki köken izlemesi için varyant bağı korunmalı. Sahte setter üretilmemeli. |
| Davranışa katılım | Çağrılabilirlik, gerçek alıcı ve davranışa katılım mevcut `is_private`/`is_static` ikilisinden ayrı taşınmalı. `Reader` özel metodu çağırabiliyor; özel anahtar sözcüğüyle metodu ayıklamak güvenilir değil. Detaylı kapsam modeli sonraki görünürlük araştırmasına bağlıdır. |
| Eşitlik ve etki | Çözülmüş ortak işlem rolü, alıcı/dışarı verilen alan üzerindeki değişiklik ve analiz eksikliği ayrı bilgiler olmalı. `Revision` alan yazmayı, `SharedWallet` paylaşılan referanstan gözlenen iç değişikliği gösterir; `Version::next_owned` yerel kopyaya yazmanın karşı örneğidir. `is_generated` veya `&mut self` bir etki özeti değildir. Bunların `Value` kararındaki rolü ayrıca kararlaştırılmalı. |

Son iki satırın bütün ayrıntıları ilk kabul testine yüklenmemeli. İlk test açık alan ve açık metot kullanarak görünürlük belirsizliğini ayırabilir. `TypeShape`, portlar ve `Platform` yalnız ekleme ile genişletilmeli; Java'nın mevcut sonuçları korunmalı. [TypeSource/SourceReader](../crates/core/src/application/ports.rs) → [CheckProject](../crates/core/src/application/check_project.rs) bağı değişmez.

## Sonraki odaklı kabul testi

Önerilen ilk davranış testi: **`an_aggregate_with_behavior_and_a_directly_writable_field_is_a_hybrid`**. Domain'in çağıran arayüzünden, `balance` alanı dışarıdan yazılabilen ve `withdraw(amount)` davranışı bulunan bir tip kurulsun. Setter ve eşitlik işlemi bulunmasın. `Types::classify` sonucu bağımsız beklenen `Hybrid` değeriyle karşılaştırılsın. Bu seçim getter, akıcı çağrı ve konteyner politikalarına bağlı değildir.

Önce gereken ek sözleşme bilgisi derlenebilir biçimde taşınsın; henüz yeni davranışa bağlanmasın. Ardından test çalıştırılsın: başarısızlık eksik API/derleme hatasından değil, hibrit kararının henüz üretilmemesinden gelmeli. En az domain değişikliğiyle yeşile geçilsin. Devamındaki ayrı karşı örnekte aynı açık alanlı tipten davranış kaldırılınca `DataStructure` beklenir. Mevcut Java `Enum`/`Record` ve setter testleri korunur.

Bu test Rust çözümlemesini kanıtlamaz. Rust adaptörü seçildiğinde kaynak örneğinden üretilen gerçek alan/metot bilgisi ve hibrit bulgusu ayrıca sınanmalıdır. Sınıflandırma etiketini adaptörde sabitleyen test kabul kanıtı değildir. Hibrit raporundaki mevcut yalnız-setter listesine doğrudan alanın nasıl ekleneceği de o uygulamanın kapsamına alınmalıdır.

## Açık kararlar

- Kapsülleme sınırı tip mi modül mü? Aynı modüldeki özel alan yazması ve `pub(crate)`/trait erişimi sınıflandırmaya nasıl katılacak? `Wallet`'a kesin etiket vermeden sonraki görünürlük araştırması tamamlanmalı.
- Değişmezlik hangi erişilebilir durumu kapsayacak? `Cell`, `RefCell`, kilitler, paylaşılan işaretçiler, döndürülen `&mut` ve bilinmeyen çağrıların etkisi ayrı araştırılmalı. Bu çalışma yalnız `Cell`/`RefCell` karşı örneklerini çalıştırdı; genel etki çözümleyicisi kurmadı.
- `&mut self` ile tutulan değeri yenilemek ve `Copy` alıcının `mut self` ile kendi yerel alanını değiştirmesi değer politikası açısından nasıl ayrılacak? `Status::close` ve `Version::next_owned` bu soruyu kapatmaz; her ikisi de tek başına nesne kimliği veya Law of Demeter ihlali kanıtı değildir.
- Alansız saf enum ve salt veri yükü olan enum ne zaman `Value` olabilir? Değer eşitliği için `Hash` zorunlu mu, `PartialEq` yeterli mi? İlk dilimde otomatik proje değeri muafiyeti önerilmez.
- Hesaplayan/saran getter, kendi tipini döndüren akıcı metot ve konteyner değişikliği için [mevcut Java soruları](../decision.md) açık kalır. `Version::next` deneyi akıcı çağrı izni verildiğini göstermez. Rust örneklerinin böyle bir beklentisi bu kararlara bağlanmalıdır.
- Makro/trait açılımı eksikse sınıflandırma ve rapor nasıl eksik sayılacak? Üretilmiş iş davranışı eşitlik türetmesiyle karıştırılmamalı. Varyanttan çıkarılan nesnenin kökeni ayrıca izlenmeli.

## Doğrulama

Rust/Cargo 1.99.1, rustfmt 1.10.0-stable ve clippy 0.1.99 kullanıldı. İnceleme düzeltmesinden sonra her edition'da 19 test geçti; atlanan test yok. Dört edition'da biçim ve lint yeniden geçti. Öğrenme deneyi çıktıları bu worktree'nin `target/rust-classification` dizinine yazıldı.

Repo kökünden:

```sh
mkdir -p target/rust-classification
for edition in 2015 2018 2021 2024; do
    rustc --edition="$edition" --test -D warnings examples/rust-classification.rs -o "target/rust-classification/classification-$edition" || exit 1
    "target/rust-classification/classification-$edition" || exit 1
    rustfmt --edition "$edition" --check examples/rust-classification.rs || exit 1
    clippy-driver --edition="$edition" --test -D warnings examples/rust-classification.rs -o "target/rust-classification/clippy-$edition" || exit 1
done
```

İnceleme düzeltmesinde 2024 edition'ında şu hedefli deneyler çalıştırıldı. Her geçici kopya `rustc --edition=2024 --test -D warnings` ile derlendi; kırmızı sonuçlar assertion hatasıdır.

| Deney | Sonuç |
|---|---|
| Eski açık alan testi + reddederken bakiyeyi de azaltan hatalı `OpenWallet::withdraw` | 1 geçti; inceleyicinin bulduğu kaçan mutasyon yeniden üretildi. |
| Güçlendirilmiş test + aynı hatalı örnek | 1 kırıldı; reddin hemen sonrasında beklenen 50 yerine -10 gözlendi. |
| Güçlendirilmiş test + doğru örnek | 1 geçti. Asıl dosyada doğru örnek korundu. |
| Yeni sahip olunan kopya testi + doğru `next_owned` | 1 geçti. |
| Aynı yeni test + yerel sayıyı 1 yerine 2 artıran geçici örnek | 1 kırıldı. |

Geçici kopyalar `target/rust-classification/review-fixes` altında oluşturuldu. Açık alan deneyi için `sample`, sırasıyla `old_test_faulty_fixture`, `strengthened_test_faulty_fixture`, `strengthened_test_correct_fixture`; yeni kopya deneyi için `owned_copy_correct_fixture`, `owned_copy_faulty_fixture` dosya adını alır. Kullanılan komutlar:

```sh
rustc --edition=2024 --test -D warnings "target/rust-classification/review-fixes/$sample.rs" -o "target/rust-classification/review-fixes/$sample"
"target/rust-classification/review-fixes/$sample" --exact a_public_field_allows_a_balance_the_method_rejects
```

Yeni kopya deneyi son komutta `--exact an_owned_copy_receiver_can_change_locally_and_preserve_the_original` filtresini kullanır. Bu dosyalar geçici kanıttır; kalıcı örneğin yerini tutmaz. Deneyler Rust dil gözlemlerini ve örnek denetimlerini sınar; analiz aracının sınıflandırması veya yeni üretim davranışı için TDD kanıtı değildir.

Önceki sürümdeki 18 geçici bozulma deneyi ve dört edition'daki altı olumsuz dil deneyi bu düzeltmede yeniden çalıştırılmadı. Önceki çekirdek koşusu `cargo test -p law-of-demeter-core domain::classification::tests -- --quiet` ile 110 test geçirmişti; o da yeniden çalıştırılmadı. Workspace'in tamamı ve Maven örnekleri bu araştırmada yeniden çalıştırılmadı; önceki [başlangıç doğrulaması](rust-support.md#başlangıç-doğrulaması) yeni sonuç sayılmaz.
