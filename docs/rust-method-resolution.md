# Rust metot seçimi ve çağrı hedefi

**Öneri:** Adaptör çağrı yazımını gerçek alıcıdan, seçilen bildirimi somut gövdeden, dönüş tipini de dönen nesnenin kökeninden ayırsın. Ad ve argüman sayısı Rust hedefini seçmeye yetmez. Var olan tip/akış köprüleri kullanılsın; yalnız hedefi bilen tüketicinin ihtiyaç duyduğu dil bağımsız bilgi eklensin. Bu çalışma çözümleyici veya yeni LoD politikası uygulamaz.

İncelenen sürüm `fe9428a`, tarih 23 Kasım 2026. [Karar listesindeki](../decision.md) sıradaki araştırma budur. Üretim kodu, karar listesi, Cargo dosyaları ve önceki araştırmalar değiştirilmedi. [Kapsam](rust-support.md), [sınıflandırma](rust-classification.md), [görünürlük](rust-visibility.md) ve [değişebilirlik](rust-mutability.md) kararlarının açık sınırları korunur.

## Bugünkü model neyi tüketiyor?

| Yol | İncelenen davranış ve sınır |
|---|---|
| [Java çağrı araması](../crates/java-source/src/mapping/returns.rs) | `receiver_type`, `lineage`, ad ve argümanlarla adaylar kurulur. `Candidate`, sınıf/metot tip değişkenlerini yerine koyar. `Invocation` parametreleri, dönüşü, çıkarım bağlarını ve seçilebilmişse `Declared` bilgisini taşır. Rust alıcı sırası, trait kapsamı ve ilişkili tip çözümü bu Java yolunu kopyalayarak elde edilmez. |
| [Java aşırı yükleme](../crates/java-source/src/mapping/overloads.rs) | Kesin/genişletilmiş dönüşüm ve değişken argüman aşamalarıyla seçim yapılır. Belirsiz sonuçta `invoked` ortak dönüş tipini koruyabilir ama `declared: None` verir. Dolayısıyla dönüş tipinin bilinmesi hedefin seçildiği anlamına gelmez. |
| [Argüman/sonuç çıkarımı](../crates/java-source/src/mapping/arguments.rs) | Beklenen tip, iç çağrılar, lambda ve metot referanslarından parametre/dönüş bağları tamamlanır. Bu iş adaptördedir. [Akış eşlemesi](../crates/java-source/src/mapping/links.rs) seçilen bildirimin parametre/dönüş bilgisini tüketir. |
| [Kaynak → Call](../crates/java-source/src/mapping/expressions.rs) | Alıcı ifade eşlenir; tip üzerinden çağrı `StaticType` olabilir. Çözülmüş dönüş `Call::returning` ile verilir; yardımcı ve argüman akışı ayrıca bağlanır. Rust'ta `::` yazımı gerçek alıcı olmadığını göstermez. |
| [Call ve köken](../crates/core/src/domain/method/call.rs) | `Call` gerçek alıcı ifadesi, ad, argümanlar, isteğe bağlı dönüş tipi ve akış bağlarını taşır. Seçilmiş bildirim/uygulama kimliği taşımaz. `standing_after_call`, `StaticType` alıcısındaki sonucu arkadaş sayar; bunu gerçek alıcılı Rust yol çağrısına uygulamak ilişkiyi kaybettirir. |
| [Şekil ve yeniden arama](../crates/core/src/domain/type_shape.rs) | `MethodShape` parametre/dönüş, statik ve görünürlük bilgilerini taşır. Trait uygulama kimliği, alıcı dönüşümü veya ilişkili tip projeksiyonu yoktur. [`Types::is_fluent`/`declares_static`](../crates/core/src/domain/classification.rs) katalogda ad/argüman sayısı gibi bilgilerle yeniden arar; Rust'ta çözülmüş hedef yerine bu aramaya güvenilemez. |

`TypeRef`, sınıf/tip değişkeni ve eleman tipi bulunan dizi taşır; dizi uzunluğu, const generic, referans yetkisi ve ilişkili tip projeksiyonu için Rust modeli değildir. Adaptör seçim sırasında tam dil bilgisini kullanmalı, domain'e gereken sonuçları çözülmüş vermelidir. Yeni const-generic veya trait çözümleyicisini domain'e koymak önerilmez.

Mevcut [`Lambda`/`MethodReference`](../crates/core/src/domain/method/expression.rs) köprüleri `resolving`, `declaring`, `taking`, `returning` ve besleme bağlarını zaten alır. Bunlar önce kullanılmalı. Genel çıkarım sistemi veya kapsayıcıların bütün köken kuralları bu araştırmanın işi değildir. Mevcut Java ve çekirdek testleri yeniden çalıştırılmadı.

## Yeniden kullanılacak kanıt

[`rust-semantics.rs`](../examples/rust-semantics.rs) içindeki `a_shared_trait_receiver_can_be_selected_before_a_mutable_inherent_receiver` ve `a_fully_qualified_call_can_choose_a_different_method_with_the_same_name`, otomatik ödünç alma sırasının “her zaman inherent önce” olmadığını zaten sınar. Bu deneyler kopyalanmadı veya yeniden çalıştırılmadı.

[`rust-visibility/helpers.rs`](../examples/rust-visibility/helpers.rs) içindeki `a_public_trait_object_uses_an_overridden_selected_method`, `&dyn WalletSource` üzerinden değiştirilmiş `selected` gövdesini ayırt eder. Önceki inceleme, yalnız varsayılan gövdeyi kullanan iki örneğin yanlış `source()` çağrısını yakalamadığını göstermişti; onaylanmış karşı örnek korunur. Trait bildiriminin seçilmesi, `dyn` alıcının somut uygulamasının veya gövdesinin bilindiğini göstermez.

## Doğrulanmış yeni dil gerçekleri

[Rust Reference](https://doc.rust-lang.org/reference/expressions/method-call-expr.html) alıcı adaylarını tekrarlı dereference, ardından mümkünse dizi→dilim dönüşümüyle kurar; her `T` adayından sonra `&T`, `&mut T` eklenir. Her adayda inherent ve kapsamda görülen trait metotları aranır; tip parametresinin trait sınırları ayrıca önemlidir. Bu sıra bütün adaylardan bağımsız bir “inherent kazanır” kuralı değildir. Seçimden sonra ödünç, yaşam süresi ve güvenlik koşulları çağrıyı yine reddettirebilir.

[`selection.rs`](../examples/rust-method-resolution/selection.rs) dokuz test içerir. Etiketler seçilen örnek gövdesinin tanıklarıdır; analiz aracının ürettiği sınıflandırmalar değildir. Noktalı çağrı gereken yerlerde gerçekten kullanılır; açık trait çağrıları ise yalnız istenen bildirimin seçilmesini sınar.

| Ayrım | Deney ve bağımsız gözlem |
|---|---|
| Sarmalayıcı / gösterdiği nesne | `WalletWrapper.selected()` kendi metodundan `wrapper` etiketi verir. Kendi metodu olmayan `WalletRef.selected()` otomatik dereference ile `Wallet` metodundan `wallet` verir. Dönen referanslar da beklenen saklanan/ödünç alınan cüzdanı gösterir. Aynı dönüş kökeni, aynı seçilmiş metot demek değildir. |
| Dizi / dilim alıcısı | Yalnız `[i32]` için `SliceTotal` uygulanmıştır; `[20, 30].slice_total()` 50 verir. Bu örnekte seçilen imza dilim alıcısı gerektirir. Dizi üzerinden her metodun dilime ait olduğu sonucu çıkmaz. |
| Kapsamdaki trait | Aynı `Wallet` ve `quote` adı, yalnız `BalanceQuote` alınan modülde `balance`, yalnız `CreditQuote` alınan modülde `credit` verir. Başka modülde bir trait'in uygulanmış olması onun her çağrı kapsamında bulunduğunu göstermez. |
| Açık trait yolu | `<Wallet as BalanceQuote>::quote` kapsamındaki noktalı çağrıyla aynı `balance` sonucunu verir. `<Wallet as CreditQuote>::quote` bilerek `credit` seçer. Bu, belirsiz noktalı çağrının kendiliğinden çözüldüğü iddiası değildir; karşı derleme örneği aşağıdadır. |
| Generic sınır ve ilişkili dönüş tipi | `quote_from_bound<T: Quote>` içinde `source.quote()` trait bildirimine bağlanır. `QuoteSource` için `Quote::Output = u32`; sonuç açıkça `u32` olarak 50 alınır. Aynı somut değerde doğrudan `.quote()` inherent metodundan metin `inherent` verir. Generic gövde seçimi, somut tip gelince doğrudan inherent arama olarak yeniden yapılmaz. |
| Nokta, tip yolu, trait yolu | `source.quote()` ve `QuoteSource::quote(&source)` aynı inherent metni verir. `<QuoteSource as Quote>::quote(&source)` farklı trait metodundan 50 verir. Üçünde de gerçek alıcı vardır; yol yazımı tek başına alıcısız çağrı değildir. |
| Gerçekten alıcısız ilişkili fonksiyon | `QuoteSource::forwarded_wallet(&wallet)` imzasında `self` yoktur; cüzdan sıradan argümandır. Dönen referans aynı cüzdanı gösterir ve bakiyesi 50'dir. Alıcısız olmak, yeni nesne oluşturma kanıtı değildir. |
| Edition'dan bağımsız açık dizi çağrısı | `IntoIterator::into_iter(wallets)` dört edition'da `std::array::IntoIter<Wallet, 2>` verir. Öğeler açıkça sahip olunan `Wallet` tipindedir; bakiyeler `[50, 90]` olur. Taşınan öğenin LoD kökeni burada kararlaştırılmaz. |

[İlişkili öğeler](https://doc.rust-lang.org/reference/items/associated-items.html) metot alıcısıyla alıcısız fonksiyonu ayırır ve `T::Output` anlamını tanımlar. [Açık çağrı yolları](https://doc.rust-lang.org/reference/expressions/call-expr.html#disambiguating-function-calls) ad çakışmasını çözebilir. Generic örneğin sonucu bu küçük program için derleyici kanıtıdır; bütün trait/tip çıkarımı davranışının sınandığı iddia edilmez.

### Edition'a bağlı gerçek öğe tipi

| Kaynak | Olumlu kapsam ve gözlenen tip | Karşı kapsam |
|---|---|---|
| [`array_before_2021.rs`](../examples/rust-method-resolution/array_before_2021.rs) | 2015/2018: `wallets.into_iter()` sonucu `std::slice::Iter<'_, Wallet>`, öğe `&Wallet`. Dönen iki referans başlangıç dizisinin iki öğesiyle aynı adresi gösterir. | 2021/2024: aynı beklenti `E0308`; gerçek sonuç `std::array::IntoIter<Wallet, 2>`. |
| [`array_from_2021.rs`](../examples/rust-method-resolution/array_from_2021.rs) | 2021/2024: aynı yöntem yazımı `std::array::IntoIter<Wallet, 2>`, öğe `Wallet`. Bakiyeler `[50, 90]` olur. | 2015/2018: aynı beklenti `E0308`; gerçek sonuç `std::slice::Iter<'_, Wallet>`. |

Bu fark [Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2021/IntoIterator-for-arrays.html) tarafından belgelenir: eski edition'ların noktalı dizi çağrısı ödünç alınmış dizi yolunu korur; açık `IntoIterator::into_iter` çağrısı bu istisnaya girmez. Testlerin tip beklentileri ayrıdır; aynı beklenti bütün edition'lara zorlanmadı.

Yalnız bu bilinçli eski yazımı korumak için dizi örneklerinde `array_into_iter`, eski örnekte ayrıca `clippy::into_iter_on_ref` lint'i fonksiyon düzeyinde izinlidir. Clippy ilk koşuda eski örneğe `.iter()` önerdi; bunu uygulamak araştırılan çağrıyı değiştirirdi. Genel uyarılar izinli değildir; diğer kontroller `-D warnings` ile çalıştı.

[Reference](https://doc.rust-lang.org/reference/expressions/method-call-expr.html#r-expr.method.edition2021) bu istisna için metot adını temsil eden token'ın edition'ını belirtir. Buradaki makrosuz dosyalarda `--edition` yeterlidir. Makrodan gelen token bağlamı ve açılım sınırları sonraki makro araştırmasının konusudur; yalnız paket edition'ını her koşulda yeterli saymak önerilmez.

## Adaptörün vereceği en az bilgi

| Ayrı olgu | Aktarım ve sınır |
|---|---|
| Kaynaktaki yazım ve gerçek alıcı | Nokta veya `::` ayrımı yerine seçilen imzadaki alıcı kullanılsın. `<T as Trait>::m(&value, arg)` modelde `value` alıcısıyla kurulur; kaynak argümanı `&value`, sıradan ilk argüman gibi iki kez eklenmez. Gerçekten alıcısız fonksiyonda bütün argümanlar korunur. `Call::new` mevcut alıcı ifadesini zaten kabul eder. |
| Alıcıya uygulanan dönüşüm | Hangi borrow/dereference/dilim dönüşümünün uygulandığı adaptörde çözülsün. Etkin alıcının nesne bağı korunsun. Kullanıcı tanımlı `Deref` gövdesi başka nesne verebilir; her sarmalayıcıya otomatik “şeffaf ve arkadaş” etiketi konmasın. Köken çözülmemişse eksiklik saklanmasın. |
| Seçilmiş bildirim ve dispatch | Trait/metot bildirimi, biliniyorsa somut uygulama/gövde kimliği ayrı tutulsun. Kesin hedef, bilinen trait bildirimi ama dinamik uygulama ve belirsiz/çözülememiş çağrı karıştırılmasın. Ad ve argüman sayısı kalıcı kimlik değildir. |
| Çözülmüş dönüş ve gerekli yerine koymalar | `Call::returning` somut domain tipini alabilir. `QuoteSource` örneğinde `T = QuoteSource`, `<T as Quote>::Output = u32` çözümü adaptöre aittir. Argüman/lambda yorumuna gereken somut tipler mevcut `resolving`/`declaring`/`taking` köprülerine verilsin. Çözülememiş projeksiyon uydurma proje tipi adına dönüştürülmesin. |
| Sonucun nesne kökeni ve akışı | Dönüş tipi ve seçilmiş metot kimliği, “yeni nesne” kanıtı değildir. Mevcut `returning_from`, işlevsel argümanın ürettiği kökeni yalnız konteyner→konteyner sonucuna taşır; `forwarded_wallet` gibi sıradan argüman alias'ını genel olarak modellemez. Alıcı kaynak argümanlarından ayrıldıysa akış indisleri de gerçek sıradan argümanlara göre kurulmalıdır. Genel alias/taşıma/iterator köken kuralları sonraki adımlardır. |

İlk dar yol için her çağrıya büyük bir kayıt şeması eklemek gerekmez. Gerçek alıcıyı eşlemek ve çözülmüş dönüşü vermek mevcut `Call` ile mümkündür. Çekirdeğin akıcılık/statiklik/yardımcı aramasının seçilen bildirimle karar vermesi gerektiğinde, isteğe bağlı dil bağımsız hedef/gövde kimliği ve çözüm durumu eklemek gerekebilir. `MethodShape` adını trait adlarıyla birleştirip sahte metot adları üretmek veya Rust tip kurallarını domain'e taşımak önerilmez.

`dyn Trait` için seçilmiş bildirim bilinse de somut gövde ayrıca kanıt gerektirir; varsayılan trait gövdesi otomatik seçilmez. Birden çok hedefin aynı köken özetini vermesi ileride değerlendirilebilir. [Görünürlük](rust-visibility.md) gövdenin sabit olduğunu, [değişebilirlik](rust-mutability.md) de çağrının etkisini tek başına kanıtlamaz. Hedef/etki bilinmezliği, alıcının erişim kapsamı ve değer sınıflandırması ayrı kalır.

Belirsiz veya çözülemeyen Rust çağrısında ad/argüman sayısıyla bir aday seçilmesin. Bilinen alıcı/tip bilgisi korunabilir; fakat hedefin çözülmediği açık kalmalıdır. Ortak dönüş tipi bulmak, çağrı uygunluğunu veya gövde bilgisini kanıtlamaz. Eksik analiz raporu ve çıkış kodu ayrı karar adımıdır.

## Sonraki dar kabul testi

Öneri: **`a_type_qualified_receiver_call_keeps_the_dot_calls_receiver_origin`**. Rust adaptör altyapısı seçilip kaynak eşlemesi kurulurken, `Reporter` metoduna verilen `order: &Order` için şu üç biçim aynı benzersiz trait metoduna çözülsün: `order.customer().activate()`, `Order::customer(order).activate()`, `<Order as CustomerSource>::customer(order).activate()`. Kapsamda yalnız ilgili trait olsun; müşteri davranışlı bir proje nesnesi olsun.

Beklenti üçünde de aynıdır: `customer` parametre olan siparişte çağrılır; dönen müşteri arkadaş yapılmaz ve `activate` aynı nedenle raporlanır. Adlar factory/akıcı çağrı istisnasına girmez. Tip yolunu `StaticType` alıcısına çeviren yanlış eşleme bu testi bozar. Gerçek alıcıyı ayrı verip `returning` köprüsünü kullanmak dar test için yeterlidir; yeni dispatch/kimlik şemasını aynı teste yüklemek gerekmez.

Bu henüz çalıştırılmış Rust analiz testi değildir. İlgili eşleme arayüzü derlenebilir olduktan sonra eksik davranış nedeniyle kırmızı görülmeli; yok crate veya ayrıştırıcı kurulum hatası kırmızı kanıt sayılmamalıdır. Dönüş tipini ve etiketi elle sabitleyip geçen bir domain testi Rust çözümleyicisinin kanıtı değildir. Mevcut Java sonuçları korunmalı; alıcısız factory, akıcılık, konteyner, sahiplik, makro ve değer kararları bu kabul testiyle kapatılmamalıdır.

## Doğrulama

Taban macOS ARM64, Rust 1.99.1, rustfmt 1.10.0-stable, clippy 0.1.99. Ortak kaynak dört edition'da 9'ar test; ilgili dizi kaynağı kendi iki edition'ında 1'er test geçti: **10 test/edition**, toplam 40 koşu, atlanan yok. Toplam 11 farklı test gözlemi vardır. Yedi `.rs` dosyasının biçimi dört edition'da denetlendi; olumlu iki test hedefinin uygulanabilir lint kontrolleri geçti. Eski dizi yazımındaki iki yerel lint izni yukarıda açıklanmıştır.

Repo kökünden olumlu koşular:

```sh
mkdir -p target/rust-method-resolution
for edition in 2015 2018 2021 2024; do
    case "$edition" in
        2015|2018) era=array_before_2021 ;;
        2021|2024) era=array_from_2021 ;;
    esac
    for sample in selection "$era"; do
        rustc --edition="$edition" --test -D warnings "examples/rust-method-resolution/$sample.rs" -o "target/rust-method-resolution/$sample-$edition" || exit 1
        "target/rust-method-resolution/$sample-$edition" || exit 1
        clippy-driver --edition="$edition" --test -D warnings "examples/rust-method-resolution/$sample.rs" -o "target/rust-method-resolution/clippy-$sample-$edition" || exit 1
    done
    rustfmt --edition "$edition" --check examples/rust-method-resolution/*.rs examples/rust-method-resolution/denied/*.rs || exit 1
done
```

| Olumsuz kaynak | Kapsam ve beklenen tanı |
|---|---|
| [`ambiguous_traits.rs`](../examples/rust-method-resolution/denied/ambiguous_traits.rs) | Dört edition: `E0034`, `multiple applicable items in scope`; aynı ad ve argüman sayısı seçim yapmaz. |
| [`trait_not_in_scope.rs`](../examples/rust-method-resolution/denied/trait_not_in_scope.rs) | Dört edition: `E0599`, `Wallet` üzerinde `quote` bulunamaz; trait kapsamda değildir. |
| [`missing_generic_bound.rs`](../examples/rust-method-resolution/denied/missing_generic_bound.rs) | Dört edition: `E0599`, `&T` üzerinde `quote` bulunamaz; somut çağıranın inherent metodu generic gövdeyi kurtarmaz. |
| [`receiverless_as_method.rs`](../examples/rust-method-resolution/denied/receiverless_as_method.rs) | Dört edition: `E0599`, `forwarded_amount` metot olarak bulunamaz; imzada alıcı yoktur. |
| Karşı edition'daki dizi kaynağı | 2015/2018'de `array_from_2021`, 2021/2024'te `array_before_2021`: `E0308`, `Iter`/`IntoIter` uyuşmazlığı. |

Dört ortak ret ve bir edition karşılaştırması her edition'da doğrulandı: **5 ret/edition**, toplam 20. JSON hata kodu kümesi yalnız beklenen kodu içerdi. Bunlar dil deneyidir; uygulanmış analiz davranışının kırmızı TDD testi değildir.

`edition` dört edition'dan biri, `sample` tablodaki ortak uzantısız ad olacak şekilde:

```sh
rustc --edition="$edition" -D warnings --error-format=json "examples/rust-method-resolution/denied/$sample.rs" -o "target/rust-method-resolution/denied-$sample-$edition" 2> "target/rust-method-resolution/denied-$sample-$edition.jsonl"
```

Karşı edition için `sample`, ilgili dizi dosyasının uzantısız adıdır; test kaynağı şu komutla derlenir:

```sh
rustc --edition="$edition" --test -D warnings --error-format=json "examples/rust-method-resolution/$sample.rs" -o "target/rust-method-resolution/denied-$sample-$edition" 2> "target/rust-method-resolution/denied-$sample-$edition.jsonl"
```

Her iki komut reddedilmelidir; eksik dosya/araç veya başka hata kodu kabul edilen kanıt değildir. Edition uyuşmazlığı henüz olmayan analiz aracının hatası değildir.

İlk koşuda geçen 11 testin her biri için 11 ayrı geçici örnek bozuldu. Sarmalayıcı adayı kaldırıldı/eklendi; dilimden önce dizi adayı eklendi; trait import'u, açık trait yolu ve generic sınır değiştirildi. Inherent tanık, dönen argüman, dizi sırası veya sonucu da değiştirildi. Beklentiler değişmedi. Ortak dokuz ve modern dizi deneyi 2024, eski dizi deneyi 2018 edition'ında çalıştı; bütün son geçici kaynaklar `-D warnings` ile derlendi ve hedef test assertion ile kırıldı. İlk ek-sarmalayıcı denemesi kullanılmayan metot lint'ine takıldı; bu kırmızı kanıt sayılmadı. Geçici örnek, gövde kullanımını koruyarak düzeltildi ve assertion hatası ayrıca gözlendi.

Geçici kaynak/kayıtlar `target/rust-method-resolution/faults` altında tutuldu. `sample`, geçici kaynak adı; `test_name`, ilgili test adı olacak şekilde kullanılan komutlar:

```sh
rustc --edition="$edition" --test -D warnings "target/rust-method-resolution/faults/$sample.rs" -o "target/rust-method-resolution/faults/$sample"
"target/rust-method-resolution/faults/$sample" --exact "$test_name"
```

Bu kontroller örneklerin ayırt edici sonuçlarını korur; Rust derleyicisinin yöntemini değiştirdiğimizi veya bir analiz çözümleyicisi yazdığımızı göstermez. Önceki araştırma örnekleri ve geniş çekirdek/workspace/Maven koşuları yeniden çalıştırılmadı. Bütün çıktılar bu worktree'nin `target/rust-method-resolution` dizinindedir.

Test okunabilirliği ayrı, salt okunur bir geçişte incelendi: 11 ad gözlenen davranışı anlatıyor; kur/işlet/denetle bölümleri boş satırla ayrılıyor. Beklenen etiketler, tipler, tutarlar ve referans kimlikleri bağımsız; sonraki işlemle gizlenen ara durum yok. Bu incelemede kaynak değişmedi.
