# İlk Rust davranış senaryoları

Bu katalog, gerçek Rust adaptörü için ilk kaynak→rapor testlerini tanımlar. **Beklentiler ürünün bugünkü çıktısı değildir:** Rust analiz adaptörü henüz uygulanmadı. Yeni kaynaklar derlendi ve durum denetimleri çalıştı; hiçbir test sabit LoD etiketi döndürmez veya bu tabloyu kendisiyle karşılaştırmaz.

Taban `fb6b096`, tarih 23 Kasım 2026. Yalnız bu belge ve [`examples/rust-scenarios`](../examples/rust-scenarios/Cargo.toml) eklendi. Üretim, `decision.md`, `SCENARIOS.md`, önceki araştırmalar ve kök Cargo dosyaları değişmedi. Altyapı karşılaştırması ve seçimi sonraki iştir.

## Ortak kaynak ve kesinlik sınırı

On bir hedefin tamamı [`core.rs`](../examples/rust-scenarios/core.rs) içindeki `PaymentService` metotlarıdır. Kütüphane hedefi edition **2024**, özellik/build script/dış bağımlılık yoktur. Küçük manifest yalnız bağımsız Cargo proje bağlamını hazırlar; alan ve metotlar doğrudan derlenen kaynaktır.

Her satırın **kaynak durumu:** normal kütüphane ve `cfg(test)` derlemesi geçti, ilgili gerçek durum etkisi dokuz testten biriyle gözlendi. Her satırın **analiz durumu:** uygulanmadı/çalıştırılmadı. Derlenen Rust ile doğru analiz beklentisi ayrı şeylerdir. Normal analiz hedefinde `cfg(test)` kapalıdır; testler, kurulum yardımcıları ve çağrılan başka metotların gövdeleri aşağıdaki hedefin bulgu sayısına eklenmez.

Bilinen kaynak olguları ve çağrı beklentilerinin kapsamı:

- `PaymentService`, `Wallet`, `Order`, `Receiver` açık, alıcılı `debit`/`accept` davranışları taşıyan proje tipleridir; raw durum alanları özeldir. `debit` bir alanı argümana atayan setter değildir; kendi tipini döndürmez. Eşitlik/hash/Copy türetmesi yoktur. Çağrı beklentileri Value/konteyner/factory/akıcılık muafiyeti dışındaki davranışlı Object **veya Hybrid** alıcılar içindir; ikisine de burada aynı yabancı-çağrı kuralı uygulanır. [Sınıflandırma](rust-classification.md) ve [değişebilirlik](rust-mutability.md) önerisindeki otomatik proje Value çıkarımını erteleme sınırı korunur; `Cell` var diye genel Value/Hybrid kuralı icat edilmez.
- `AddressData`, `DeliveryData`, `WalletData` metotsuz veri yapılarıdır. Açık alanları vardır, kendi davranışları yoktur. İçerdiği Wallet'ın davranışı WalletData'nın davranışı yapılmaz; Wallet da veri sarmalayıcısı nedeniyle veri/değer sayılmaz.
- [Görünürlük önerisine](rust-visibility.md) göre aynı kök modüldeki özel alan erişimi tek başına hibritlik kanıtı değildir. Ancak `Order::wallet` → `&Wallet` → açık `debit(&self)` yolu Cell içindeki durumu değiştirme yetkisini dışarı taşır. Paylaşılan referans veya doğrudan `&mut`/Cell alanı verilmemesi, bu yetkinin kaçmadığını kanıtlamaz. Bu kaçan yetkinin hibritlikteki yeri, özellikle Order'ın Object/Hybrid ayrımı, [açık değişebilirlik politikasıdır](rust-mutability.md); fixture için kesin hibrit sayısı verilmez.
- `Order::wallet`, başkasının alanını döndüren açık metottur; Service'in kendi yardımcısı değildir. Bilinen gövde, dönen Wallet'ı Service'in alanı yapmaz. `balance` gözlem metotlarının getter mı davranış mı sayıldığı başka açık davranışların bulunduğu olgusunu değiştirmez; Object/Hybrid ayrımını çözmez. Hesaplayan/saran getter kararı burada kapanmaz.

Bu kapsam adaptöre elle Object/Hybrid etiketi vermek için değildir. Gerçek testte tip/bildirim/erişim bilgisi kaynaktan gelmeli, domain sınıflandırmalıdır. Object/Hybrid ayrımındaki açık karar aşağıdaki on bir çağrı beklentisini değiştirmez. Sınıflandırma veya kaynak okuma hatası beklenen çağrı ihlali yerine kabul edilmemelidir. Hibrit bulgusu ile çağrı ihlali ayrı konulardır; bu katalog hibritlik doğrulaması yapmaz.

## Kesin çağrı beklentileri

Yol bütün satırlarda `examples/rust-scenarios/core.rs`'dir. Satırlar şu kaynak sürümüne aittir. “İzinli” çağrı sayısı ve “ihlal” sayısı yalnız yazılan hedef metodun kendi gövdesidir. Alan okumak, `let`, borrow veya struct ifadesi ayrıca metot çağrısı sayılmaz.

| ID ve tam hedef | İzinli çağrılar | İhlal ve kök neden | Yakın karşı örnek / eşdeğer biçim |
|---|---|---|---|
| S1 `PaymentService::on_self` | 1: satır 9 `self.debit`; C'nin kendisi. | 0. | S5 aynı adlı davranışı dönen Wallet'ta çağırır; yalnız metot adına göre izin verilmez. |
| S2 `PaymentService::on_parameter` | 1: satır 13 `wallet.debit`; doğrudan parametre. | 0. | S5/S6'da aynı Wallet davranışı parametrenin döndürdüğü nesnededir. Parametre ve dönüşü aynı köken sayılmamalı. |
| S3 `PaymentService::on_direct_field` | 1: satır 17 `self.wallet.debit`; C'nin doğrudan alanı. | 0. | S9'daki `data.wallet` C'nin alanı değildir. Her alanı arkadaş sayan eşleme ayrımı kaçırır. |
| S4 `PaymentService::on_created` | 2: satır 22 `Cell::new` ilişkili fonksiyon çağrısı; satır 24 `wallet.debit`, f içinde struct ifadesiyle oluşturulan Wallet. | 0. | S6'da `let wallet` yeni nesne değil, dönen nesnenin bağlamasıdır. Wallet'ın oluşturulma kanıtı `Cell::new` adı değil, `Wallet { ... }` ifadesidir. |
| S5 `PaymentService::on_returned_chain` | 1: satır 29 `order.wallet`; doğrudan parametre üzerinde. | 1: aynı satırdaki `debit`. Kök, `wallet` çağrısının döndürdüğü davranışlı Wallet; mevcut domain karşılığı `return_of("wallet")`. | S2 doğrudan Wallet parametresi; S11 işi Order'a iletir. S6/S7 zincirin eşdeğer yerel/borrow biçimleridir. |
| S6 `PaymentService::on_returned_local` | 1: satır 33 `order.wallet`. | 1: satır 34 `wallet.debit`. Yerel gerekçesinin altında aynı `return_of("wallet")` kökü. | S4 gerçek oluşturma; S5 zincir. Bütün yerelleri oluşturulmuş sayan eşleme yanlış izin verir. |
| S7 `PaymentService::on_returned_reborrow` | 1: satır 39 `order.wallet`. | 1: satır 41 `borrowed.debit`. S6 ile aynı kök; `&*wallet` yeni Wallet değildir. | S5/S6 ile aynı sonuç. Borrow'u literal/oluşturma yapan eşleme yanlış izin verir. |
| S8 `PaymentService::read_data` | 0 çağrı; satır 45'te yalnız veri alanları okunur. | 0. | S9 son alanın davranışlı nesnesinde çağrı yapar. Her noktayı çağrı/ihlal sayan yaklaşım S8'i yanlış raporlar. Bu satır tek başına sınıflandırıcıyı doğrulamaz. |
| S9 `PaymentService::through_data_wrapper` | 0. | 1: satır 49 `debit`. Kök, parametre veri yapısındaki `wallet` alanıdır; mevcut karşılık `field_of("wallet")`. | S3 C'nin doğrudan Wallet alanı, S2 doğrudan Wallet parametresi. Veri sarmalayıcısı alt nesneye muafiyet kazandırmaz. |
| S10 `PaymentService::pass_returned` | 2: satır 53 `order.wallet` ve `receiver.accept`; ikisinin alıcısı da doğrudan parametre. | 0. Dönen Wallet argüman olarak verilir; bu hedefte Wallet üzerinde davranış çağrılmaz. | S5 aynı dönüş nesnesinde `debit` çağırır. Yabancı argüman ile yabancı alıcı karıştırılmamalı. |
| S11 `PaymentService::on_order_parameter` | 1: satır 57 `order.debit`; doğrudan parametre. | 0. | S5 ile aynı bakiye etkisi, farklı nesne ilişkisi. `Order::debit` kendi doğrudan Wallet alanını kullanır; ihlali başka yardımcıya saklayan bir karşı örnek değildir. |

S4'teki `Cell::new` sonucu üzerinde bu hedefte çağrı yoktur; static/factory **sonucuna** verilecek muafiyet bu beklentiyi belirlemez. S5–S7/S9'da metot adı, Self dönüşü veya standart tip istisnası ihlali yanlışlıkla örtemez: çağrılan Wallet davranışı `debit` birim döndürür ve proje tipidir.

Dayanak, [README kuralı](../README.md#kural) ve [mevcut çağrı kararıdır](../crates/core/src/domain/method/call.rs). [Köken araştırması](rust-origins.md), S4/S6/S7 ayrımını destekler; [sınıflandırma](rust-classification.md), S8/S9'da veri ile taşıdığı nesneyi ayırır. Bunlar Rust Reference'ın seçtiği bir LoD politikası değildir.

## Mevcut senaryolarla kısa eşleme

| Rust hedefi | [SCENARIOS.md](../SCENARIOS.md) bağı ve sınır |
|---|---|
| S1/S3 | Uygun 8: kendi nesnesi ve alanları. S2/S11 ayrıca README'deki doğrudan parametre kuralıdır; Uygun 1'in yalnız değer okuma örneğine indirgenmez. |
| S4 | Uygun 6'nın yerel oluşturma kısmı. Factory kısmı bu satırla kararlaştırılmaz. |
| S5 | Uygun değil 1: getter'ın döndürdüğü nesneyle konuşmak. |
| S6/S7 | Uygun değil 2: zinciri yerele bölmek. S7 borrow sözdizimi ekler, kuralı değiştirmez. |
| S8 | Uygun 3: veri yapısında okuma. Setter/koleksiyon yazma sonucu çıkarılmaz. |
| S9 | Uygun değil 7: veri yapısının taşıdığı davranışlı nesneye uzanmak. |
| S10 | Uygun 7: dönen nesneyi çağrı yapmadan başka metoda iletmek. |
| S11 | README'deki işi `order.charge(...)` ile sahibine iletme örneği. |

Java `extends/super`, miras alınmış alan ve alt sınıf ezmesi Rust'a sözdizimi olarak taşınmaz. Uygun 13/14 için trait, bileşim, görünürlük ve bilinen helper hedefi ayrı değerlendirilmelidir. Java overload veya record/enum etiketleri de doğrudan Rust karşılığı değildir. Özellikle `Type::method(&value)` gerçek alıcılı olabilir; yalnız `::` yazımı statik çağrı değildir.

## Kenar kapsamı ve karar bağımlılıkları

Aşağıdakiler yeni kesin bulgu fixture'ı değil, genişleme envanteridir. Bağlı belgelerdeki dil deneyleri önceden derlenmiş/onaylanmıştır; bu yolculukta yeniden çalıştırılmadı. Hiçbirinin Rust analiz doğrulaması yoktur.

| Gerekli ürün senaryosu | Mevcut derlenen kanıt / sonraki beklenti | Açık bağımlılık |
|---|---|---|
| Trait, qualified çağrı, generic sınır | [selection.rs](../examples/rust-method-resolution/selection.rs): kapsam, farklı hedefler ve alıcılı yol çağrısı. Aynı hedef/kökene çözülen biçimler S2/S5 sonucunu korumalı. | Gerçek hedef/alıcı/dönüş aktarımı; [metot kabul önerisi](rust-method-resolution.md#sonraki-dar-kabul-testi) henüz çalışmadı. |
| Dinamik helper / public-private gövde | [helpers.rs](../examples/rust-visibility/helpers.rs), override edilmiş `selected` sonucunu dinamik çağrıda ayırır. | Görünürlük bilinen gövde demek değildir; çok hedefli/eksik gövde kökeni ve helper ikamesi. Java private/override varsayımı kopyalanmaz. |
| Gölgeleme ve yeniden atama | [bindings.rs](../examples/rust-origins/bindings.rs): iç bağlama dış kökeni ezmez; eski referans kopyası yeniden hedeflenmez. | Bağ kimlikleri/kapsam aktarımı; genel kontrol akışı birleşimleri sonraki iştir. |
| Enum ve açık alan | [rust-classification.rs](../examples/rust-classification.rs), payload, davranış ve ham yazmayı ayırır; [sınıflandırma kabul önerisi](rust-classification.md#sonraki-odaklı-kabul-testi) dış yazma+davranış için Hybrid önerir. | Yeni aggregate/erişim bilgisi uygulanmadı. Enum'u doğrudan Value sayma; alansız enum Value, payload çıkarımı ve türetilmiş ortak işlemler için eksik kararları saklama. Bu envanter kesin enum raporu üretmez. |
| Modül/crate görünürlüğü | [Görünürlük fixture'ı](rust-visibility.md), aynı modül/descendant erişimini ve adı yazılamayan dönen tipten açık alan yazmasını ayırır. | Temsil sınırı ve etkin erişim sözleşmesi. Genel modül içi yazma uyarısı kararlaştırılmadı. |
| Shared/local mutation ve Value | [Değişebilirlik](rust-mutability.md): Cell/RefCell/lock/guard, Rc/Arc, cache/eşitlik; [Copy alıcısı](../examples/rust-classification.rs) yerel sonucu değiştirebilir. | Yetki, gerçekleşen etki, mantıksal değer ve LoD ayrı. `&self`, Eq/hash veya bütün yazmalar için genel Value sonucu yok. |
| Move/clone/Copy ve yeni dış değer | [Köken](rust-origins.md), taşınan Box ve aliased alt nesneleri korur. | Oluşturulan nesnenin alanı ve hangi clone sonucunun yeni LoD nesnesi sayılacağı bekliyor. |
| Option/Result ve boşluk | [roles.rs](../examples/rust-containers/roles.rs): başarı/hata, as_ref/as_mut, callback'in atlanması; tip bilinebilirken öğe yoktur. | Rol başına köken, yokluğun statik callback analizine etkisi, kapsayıcı muafiyeti ve hata raporu. Key/value alanları başarı/hata diye yeniden adlandırılmaz. |
| Map ve ilişkili Item | [projections.rs](../examples/rust-containers/projections.rs), aynı Item tipinin farklı kökenini ve yeni dış öğedeki alias'ı ayırır. [Keys kabul önerisi](rust-containers.md#sonraki-dar-gerçek-adaptör-kabul-testi): `Keys<Key, Failure>` son argümanı Failure, Item'ı `&Key`'dir. | Açık çözülmüş içerik tipi/bileşen bağı; kabul testi uygulanmadı. Result::Iter<T> callback'i tek başına son-argüman kestirmesini ayırt etmez. |
| cfg/makro/generated çağrılar | [Expansion fixture'ı](rust-expansion.md): seçili cfg, iki gerçek açılmış çağrı, proc/build üretimi, token edition ve tanı konumları. | Etkin olmayan ile açılmamış etkin kod ayrımı; çağrı kimliği/kaynak bağı ve seçilecek altyapı API'si. |
| Bilinmeyen köken | [Köken](rust-origins.md) ve [metot hedefi](rust-method-resolution.md) sınırları: bilinen dönüş tipi, bilinen gövde/köken değildir. | Eksik aktif kod yeni/literal yapılmaz. Tamlık raporu ve çıkış politikası sonraki satır; kesin “uygun” beklentisi yazılmadı. |
| Factory/fluent/container politikası | Uygun 4/5/6/10 ve Uygun değil 4/6/10/12 ilgili Java temelleridir. | Mevcut açık factory/akıcılık/konteyner yazma soruları Rust için kapatılmaz; standart isim/alias kimliği çözülmeden muafiyet seçilmez. |
| Bağımsız fonksiyon, closure/async | [Kapsam ve port sınırı](rust-support.md), module ile gerçek self'i ayırır. | Bu katalog yalnız gerçek impl metotlarını kullanır. Modülü sahte self yapma; genel closure/async/iterator/makro implementasyonu hazır sayılmaz. |

Bu envanter ilk araştırma/uygulama sırasını görünür kılar; bütün kenarlar için son ürün beklentisi veya destek tamamlandı demek değildir.

## Sonraki gerçek kaynak→rapor testi

Öneri: **`a_returned_wallet_is_reported_while_a_parameter_wallet_is_allowed`**. Bu test **yalnız çağrı bulgularını** sınar. Girdi bu fixture'ın Cargo manifesti ve normal kütüphane hedefidir. Gerçek Rust kaynak/tip adaptörleri `CheckProject`'e bağlansın; yalnız S2 ve S5 metot hedeflerinin çağrı bulguları değerlendirilsin. Test elle Method/TypeShape kurmasın, beklenen sınıflandırmayı adaptöre enjekte etmesin.

Bağımsız beklenen sonuç:

- S5'te tek ihlal: `core.rs:29`, `PaymentService::on_returned_chain`, `debit`; çözülen alıcı Wallet, kök `return_of("wallet")`. Aynı satırdaki `order.wallet` parametrede çağrıldığı için izinlidir.
- S2'de ihlal yok: `core.rs:13` `debit` doğrudan Wallet parametresinde izinlidir.
- Bu iki hedefte çözülemeyen tip/köken veya okuma/tip okuma hatası bulunmamalı; bunlar beklenen ihlal yerine geçmez. Hibrit bulgusu hakkında assertion yoktur.

Mevcut [`Selection::selects_class`](../crates/core/src/application/selection.rs), metot hedefinde sınıf bulgularını seçmez. Bu yüzden S2/S5 metot seçiminde boş hibrit raporu, tiplerin hibrit olmadığını kanıtlamaz. Ayrı sınıflandırma doğrulaması ancak kaçan yetki politikası kararlaştırıldıktan sonra sınıf seçimiyle veya metot filtresi olmayan tam seçimle yapılmalıdır; böyle bir test burada eklenmedi veya çalıştırılmadı.

Hiç çağrı üretmeyen/etkin gövdeyi atlayan eşleme S5'te beklenen 1 yerine 0 verir; bütün noktaları ihlal sayan eşleme S2'yi bozar. Bilinen getter gövdesini Service'in kendi alanı gibi açmak da S5'i yanlış serbest bırakır. Bu karşılaştırma aynı `Wallet::debit(7)` davranışının yalnız alıcı ilişkisini değiştirir.

Adaptör arayüzü derlenebilir olduktan sonra test eksik davranış yüzünden kırmızı olmalı; eksik crate/API/compiler kurulumu kırmızı değildir. Sonra en az gerçek kaynak/alıcı/dönüş eşlemesi → yeşil → davranışı koruyan refactor → bağımsız inceleme izlenir. Daha sonra S6/S7 eklenir. Backend seçimi burada yapılmadı; bu test çalıştırılmadı. Derleme veya durum assertion'ları kaynak→rapor doğrulaması değildir.

## Gerçekte doğrulananlar

Bu inceleme düzeltmesi yalnız belgeyi değiştirir. Üç fixture girdisi byte olarak aynıdır; test, derleme, biçim, lint veya geçici bozulma deneyi yeniden çalıştırılmadı. Aşağıdaki sonuçlar önceki teslimin, incelemeden geçmiş kaynak doğrulama kanıtıdır.

Rust/Cargo 1.99.1, rustfmt 1.10.0-stable, clippy 0.1.99; yalnız edition 2024 ve mevcut macOS ARM64 host. Bir `.rs` kaynağı normal lib ve test yapılandırmasında derlendi. **9 gerçek durum testi geçti**, atlanan yok; bunlar 11 katalog hedefini çalıştırır. İki hedefte Clippy ve rustfmt geçti. Bağımsız, bağımlılıksız Cargo projesinin normal lib hedefi ayrıca `cargo check` ile doğrulandı; Cargo üzerinden runtime test tekrar edilmedi.

Repo kökünden komutlar:

```sh
mkdir -p target/rust-scenarios
rustc --edition=2024 --crate-name rust_scenario_core --crate-type lib -D warnings examples/rust-scenarios/core.rs -o target/rust-scenarios/librust_scenario_core.rlib
rustc --edition=2024 --test -D warnings examples/rust-scenarios/core.rs -o target/rust-scenarios/core-tests
target/rust-scenarios/core-tests
clippy-driver --edition=2024 --crate-name rust_scenario_core --crate-type lib -D warnings examples/rust-scenarios/core.rs -o target/rust-scenarios/clippy-core.rlib
clippy-driver --edition=2024 --test -D warnings examples/rust-scenarios/core.rs -o target/rust-scenarios/clippy-tests
rustfmt --edition 2024 --check examples/rust-scenarios/core.rs
CARGO_TARGET_DIR="$PWD/target/rust-scenarios/cargo" cargo generate-lockfile --offline --manifest-path examples/rust-scenarios/Cargo.toml
CARGO_TARGET_DIR="$PWD/target/rust-scenarios/cargo" RUSTFLAGS='-D warnings' cargo check --offline --locked --manifest-path examples/rust-scenarios/Cargo.toml --lib
```

Yalnız S7'de `clippy::borrow_deref_ref` izni vardır; sadeleştirme önerisi bu hedefin sınadığı `&*wallet` yazımını kaldırırdı. Genel uyarılar izinli değildir. Kaynak konumları biçimlenmiş dosyadan okundu; hiçbir analyzer çıktısından alınmadı. Yeni olumsuz compiler kaynağı yoktur: kesin senaryoların tamamı geçerli Rust'tır.

İlk yeşil durum testleri için dokuz geçici kopya `-D warnings` ile derlendi ve hedef runtime assertion ile kırıldı: self/alan/oluşturma çağrıları kaldırıldı; parametre, reborrow, veri sarmalayıcısı, iletme ve Order yolu yanlış cüzdana yöneltildi; skaler veri okuması bozuldu. Beklentiler değişmedi. Bunlar fixture'ın durum tanıklarını sınar; LoD beklentisini veya Rust çözümleyicisini doğrulamaz. Örneğin skaler sonuca 1 ekleme bozulması bir analiz karşı örneği değildir; katalogdaki yakın karşı örnekler nesne ilişkisini değiştirir.

Geçici kaynak/kayıtlar `target/rust-scenarios/faults` altında. Her `sample` ve ilgili `test_name` için kullanılan komutlar:

```sh
rustc --edition=2024 --test -D warnings "target/rust-scenarios/faults/$sample.rs" -o "target/rust-scenarios/faults/$sample"
"target/rust-scenarios/faults/$sample" --exact "tests::$test_name"
```

Önceki learning testleri ve geniş core/workspace/Maven paketleri yeniden çalıştırılmadı. Çıktılar yalnız bu worktree'nin `target/rust-scenarios` dizinindedir. Yeni dil kuralı ileri sürülmedi; kaynaklar onaylanmış araştırmanın sade bileşimidir. Açık politika bağımlılıkları uygulama testinde kesin sonuca çevrilmemelidir.

Okunabilirlik ayrı, salt okunur bir geçişte incelendi; bulgu yok. Hedef dosya/metot/satır kapsamı, bağımsız beklentiler ve karşı örneklerin değiştirdiği ilişki görünürdür. Dokuz durum testinde kur/işlet/denetle blokları ayrıdır; helper çağrıları katalog hedef sayımına eklenmez. İnceleme kaynak değiştirmedi.
