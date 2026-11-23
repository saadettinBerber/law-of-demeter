# Rust görünürlüğü

**Öneri:** Tipin adlandırılabilmesi, değer üzerinden üye erişimi, davranışa katkı ve yardımcı çağrısının bilinen gövdesi ayrı bilgiler olsun. Rust görünürlüğü hiçbir nesneyi kendiliğinden Law of Demeter arkadaşı yapmaz. Çağrı ilişkilerinde alıcı tipi, ham durumun dışa açılmasında ise temsilin bulunduğu modül sınırı kullanılması önerilir. Bu bir ürün politikasıdır; Rust Reference Law of Demeter politikası seçmez.

İncelenen sürüm `e394142`, tarih 23 Kasım 2026. Bu çalışma yalnız [karar listesindeki](../decision.md) görünürlük araştırmasıdır. Üretim kodu, [önceki sınıflandırma araştırması](rust-classification.md) ve [hedef kapsam](rust-support.md) değişmedi; Rust adaptörü uygulanmadı.

## Bugünkü modelin sınırı

| Yol | Mevcut davranış ve Rust için sonuç |
|---|---|
| [`MethodShape`](../crates/core/src/domain/type_shape.rs) | `is_private`, `is_public`, `is_static` taşır. Bildirim kapsamı, etkin erişim veya çağrının hangi gövdeye bağlandığı yoktur. `not_public()` özel demek değildir; `as_private()` iki görünürlük bayrağını birlikte değiştirir. |
| [Davranış sınıflandırması](../crates/core/src/domain/classification.rs) | `is_exposed`, özel ve statik metotları ayıklar; genel `is_public` şartı aramaz. Paket içi/protected davranış sayılır. Değer nesnesinin eşitlik metotlarında ise ayrıca `is_public` gerekir. Rust'ta `pub` yazılmayan trait metodunu bu yüzden özel saymak yanlış olur. |
| [Java tip adaptörü](../crates/class-file/src/shape.rs) | Derlenmiş üyelerin bayraklarını `as_private`, `not_public`, `as_static` olarak aktarır. Bu Java sözleşmesi korunmalıdır; Rust anahtar sözcükleri bu bayraklara doğrudan çevrilmemelidir. |
| [Java kaynak adaptörü](../crates/java-source/src/members.rs) | Kaynak metoduna `is_private` koyar. [`own_helper`](../crates/java-source/src/mapping/expressions.rs), yalnız alıcısız/`this` çağrısında, ad ve argüman sayısıyla tek eşleşen özel metodu seçer. [`returned`](../crates/java-source/src/mapping/statements.rs), yalnız özel metodun dönüşünü `give_back` ile kaydeder; diğer dönüşleri değerlendirir. |
| [Çekirdekte yardımcı sonucu](../crates/core/src/domain/method/subject.rs) | `running_helper_at` verilmişse ad, satır ve mevcut alıcı tipiyle gövde aranır. `Context` yeniden görünürlük denetlemez. Aynı dosyadan verilen metotlarla çalışır; satır tabanlı döngü koruması vardır. [`handed_back`](../crates/core/src/domain/method.rs), bütün kaydedilmiş dönüşler parametreye dayanmayan arkadaşsa arkadaş sonucu verir. Bir yabancı/bilinmeyen dönüş veya döngü bu kanıtı engeller. |

Mevcut [sınıflandırma testleri](../crates/core/src/domain/classification/tests.rs) özel davranışın elenmesini; [Java yardımcı testleri](../crates/java-source/tests/verdicts_private_helpers.rs) özel olmayan yardımcı, parametre dönüşü, başka örnek, aşırı yükleme ve döngü sınırlarını korur. Bunlar bu araştırmada yeniden çalıştırılmadı. Yeni Rust yolu için Java'nın özel metodunu “gövdesi bilinen çağrı” yerine kullanan varsayımı genelleştirmek gerekmez.

`is_static` de görünürlük değildir. `<Wallet as Balance>::amount(&wallet)` örneği bir alıcı kullanır; tipe ait yoldan çağrılması onu alıcısız fonksiyon yapmaz. İmzayı ve hedefi adaptör çözmeli; bağımsız fonksiyon için modül sahte bir `self` olmamalıdır.

## Derlenen dil kanıtları

[Görünürlük kuralları](https://doc.rust-lang.org/reference/visibility-and-privacy.html): özel üyeye bildirildiği modül ve alt modülleri erişebilir. Bir öğeyi adıyla kullanmak için ad yolundaki üst öğeler de erişilebilir olmalıdır. `pub use` başka bir geçerli ad yolu açabilir. Bunlar ad erişimi kurallarıdır; nesne kökenini veya kural uygunluğunu belirlemez.

Tipin adını yazamamak, o tipteki değeri veya üyelerini kullanamamak değildir. Özel modüldeki bir `pub` tip, açık fonksiyonun dönüş imzasıyla dışarı ulaşabilir. Tüketici tipi adlandırmadan değerin açık metodunu çağırabilir ve açık alanını yazabilir. Rustc'nin [varsayılan olarak izin verilen `unnameable_types` lint'i](https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html#unnameable-types) bu ayrımı belgeler. Dolayısıyla kapalı ad yolu, kapalı durum sınırı kanıtı değildir.

| Ayrım | Kalıcı deney ve gözlenen sonuç |
|---|---|
| Aynı modüldeki ayrı tip | [`modules.rs`](../examples/rust-visibility/modules.rs) içindeki `Reader`, özel alanı okur/yazar ve özel metodu çağırır. Yazmadan sonraki 70 değeri hemen kaydedilir. Bu, ayrı tipin erişebilmesini gösterir; arkadaşlık veya hibrit etiketi sınanmaz. |
| Alt modül / kardeş modül | `descendant::read`, özel alanı ve metodu okuyup `(50, 50)` döndürür. Aynı erişimler kardeş modülde [`sibling_field.rs`](../examples/rust-visibility/denied/sibling_field.rs) ve [`sibling_method.rs`](../examples/rust-visibility/denied/sibling_method.rs) ile reddedilir. |
| `pub(super)` / `pub(in ...)` | Üst modüldeki çağrı ve iki üst düzeye kadar açılan adlandırılmış kapsam geçer. İlgili kapsamların dışından çağrı reddedilir. Kardeş modülü `pub(in ...)` hedefi yapmak da reddedilir. |
| `pub(crate)` / üst öğe sınırı | `reporting`, kardeş modüldeki `crate_balance` metodunu çağırır. Ancak `pub(crate)` bir fonksiyona giden özel alt modül yolu açılmaz: [`restricted_ancestor.rs`](../examples/rust-visibility/denied/restricted_ancestor.rs). |
| Tipin modülü / `impl` modülü | `Marker` başka modülde tanımlı olsa da özel `amount` metodu kendi `impl` modülünden çağrılır. Tipin tanımlandığı kardeş modülden çağrı [`impl_scope.rs`](../examples/rust-visibility/denied/impl_scope.rs) ile reddedilir. Üyelerin bildirim kapsamı yalnız tip adından çıkarılamaz. |
| Gerçek ikinci crate ve yeniden dışa aktarma | [`library.rs`](../examples/rust-visibility/library.rs) ayrı `rlib`, [`consumer.rs`](../examples/rust-visibility/consumer.rs) ayrı test crate'i olarak derlenir. `Wallet` yeniden dışa aktarılan yoldan kullanılır; özgün özel `hidden` modülü dışarıdan adlandırılamaz. Bu kanıt kardeş modül deneyinden bağımsızdır. |
| Yeniden dışa aktarılmamış tip / kullanılabilen değer | Aynı sağlayıcıdaki açık `returned_wallet`, özel `unnameable` modülünün `pub Wallet` değerini döndürür; bu tip için `pub use` yoktur. Tüketici dönüş tipini yazmadan metottan önce 50 okur, açık alana 70 yazar, hem alanı hem metodun sonucunu 70 olarak doğrular. [`external_returned_type.rs`](../examples/rust-visibility/denied/external_returned_type.rs) içindeki açık tip adı ise `E0603` ile reddedilir. |
| Trait / inherent metot | Tüketici, inherent `balance` metodunu ve `pub` yazılmamış trait uygulamasındaki `amount` metodunu çağırır. Trait yeniden dışa aktarılmıştır. Trait uygulamasındaki metoda ayrıca `pub` yazmak reddedilir. |
| Özel trait / açık dış arayüz | Tüketici, sağlayıcının özel trait'ini adlandıramaz. Sağlayıcının açık `private_traits::read` fonksiyonu bu trait'i içeride kullanabilir. Trait adının gizliliği, içerideki davranışın dış arayüzü etkileyemediği anlamına gelmez. |
| Açık/özel inherent yardımcı | [`helpers.rs`](../examples/rust-visibility/helpers.rs), iki yardımcının da saklanan cüzdana aynı referansı verdiğini gösterir. Açık olan metodun bildiriminde `pub` vardır; `Order` tipi `pub(super)` ile sınırlıdır. Bu dosya dış crate erişim deneyi değildir. |
| Özel yardımcı / değişen sonuç | Aynı dosyada özel `private_pick`, argümana göre iki farklı cüzdan referansı verir. İkisi de alıcının alanıdır; referansların farklı olması tek başına birinin yabancı olduğu anlamına gelmez. Özel olmak sonucu sabitlemez. |
| Varsayılan trait gövdesi / uygulama | Ortak `selected` gövdesi `self.source()` çağırır; seçilen uygulamaya göre farklı referanslar döner. Başka bir uygulama `selected` gövdesini de değiştirir. Bilinen varsayılan gövde, çağırdığı hedeflerin de bilindiğini göstermez. |
| Açık/özel trait ve dinamik çağrı | Hem `&dyn WalletSource` hem özel `&dyn LocalSource`, verilen alıcının uygulamasını kullanır. Varsayılan gövdeyi kullanan iki örnek korunur. Ek test `Replaced`'ı `from_public_source` üzerinden geçirir: değiştirilmiş `selected` diğer cüzdanı ve 90 bakiyesini verir; `source` ilk cüzdanı verirdi. Gizli trait tek hedef kanıtı değildir. Ortak köken özeti bulunabilmesi ayrı bir analiz sorusudur. |

[Trait gövdeleri](https://doc.rust-lang.org/reference/items/traits.html) uygulamada değiştirilebilir. [Trait nesnesi](https://doc.rust-lang.org/reference/types/trait-object.html) çağrısı çalışma zamanında tablodaki uygulamayı seçer. [Inherent uygulamalar](https://doc.rust-lang.org/reference/items/implementations.html) için de çözümlenmiş hedef gereklidir; görünürlük tek başına hedef çözümlemesi yerine geçmez. Bu deneyler genel metot seçimini veya tip çıkarımını çözmez.

## Önerilen sınırlar

Law of Demeter çağrı ilişkisi **alıcı tipi** üzerinde kalsın. Aynı modüldeki `Reader` ve `Wallet` ayrı tiplerdir. Kardeş/alt modül erişimi, görünür bir getter veya yeniden dışa aktarma dönen nesneyi arkadaş yapmaz. `Reader`'a verilen `wallet` parametresi arkadaş olabilir; ondan alınan davranışlı başka nesnenin kökeni ayrıca izlenir.

Hibritlikte ham durumun dışa açılması için varsayılan temsil sınırının **tipin tanımlandığı modül ve alt modülleri** olması önerilir. Böylece her özel alan, başka tipler aynı modülde ona erişebiliyor diye dışarıya açılmış sayılmaz. Bu, modül içindeki temsil paylaşımını kabul eden bir araç tercihidir; Rust'ın zorunlu kıldığı bir tasarım kuralı değildir.

| Durum | Her iki işte tip sınırı kullanılırsa | Önerilen ayrım |
|---|---|---|
| `Reader` aynı modülde özel `Wallet.balance` alanına yazar | Davranışlı alan taşıyan tip, yalnız dilin bu erişime izin vermesi nedeniyle hibrit sayılabilir. | Ayrı tipler LoD açısından birleşmez; modül içi yazma olanağı tek başına hibrit kanıtı sayılmaz. |
| Ham alan/modifiye edici erişimci başka modüle açılır | Davranışla birlikte hibrit adayıdır. | Etkin erişim gerçekten temsil sınırını geçiyorsa aynı sonuç önerilir; yalnız `pub` metni yeterli değildir. |
| Tipin ad yolu kapalı, değeri açık imzadan dönüyor | Yalnız ad yollarına bakılırsa dış üye erişimi yok sanılabilir. | Adlandırılabilirlik ve değer üzerinden üye erişimi ayrı incelenir. Açık dönüş imzasıyla ulaşan değerin açık alanı temsil sınırının dışında yazılabilir; `pub use` bulunmaması bunu engellemez. |

Etkin üye erişimi yalnız ad yolu veya yeniden dışa aktarma taramasıyla çıkarılmasın. Açık API imzalarından ulaşılabilen değerler de hesaba katılsın; bu değerlerin tipi için dışarıdan yazılabilir bir ad bulunması şart değildir. Ad yolunun kapalı olması veya erişim bilgisinin eksikliği, üyeler için “dışarıya kapalı” sonucu üretmesin.

Bu seçim modül içindeki kötü durum paylaşımını hibrit raporunda kaçırabilir; tip sınırını her ham erişime uygulamak ise özel alanlı davranış tiplerini gereksiz yere raporlayabilir. Modül içi tasarım uyarısı istenip istenmediği açık karardır. Kök modülün temsil alanı bütün crate'i kapsayabilir; `pub(crate)` bu durumda özel alanın erişimini genişletmeyebilir. Farklı modüldeki `impl` gövdesi, tipin özel alanlarına kendiliğinden erişim kazanmaz.

Davranış bilgisi Rust metodunda `pub` bulunmadığı için silinmesin. Ayrı tiplerden çağrılabilen alıcılı metotların erişimci/davranış bilgisi sınıflandırıcıya ulaşsın. Bu yöntem `private` bayrağının mevcut Java anlamını değiştirmez. Saf uygulama yardımcılarının ne zaman ayrı davranış sayılacağı daha dar bir politika gerektirirse ayrıca kararlaştırılmalıdır; görünürlük böyle bir kararın yerine geçmez.

Değer sınıflandırması bu araştırmanın sonucu değildir. Alan yazma, değer atama veya sahip olunan kopyanın yerel değişikliği tek başına `Value`'yu reddetmez. [Önceki araştırmadaki](rust-classification.md) otomatik proje değeri çıkarımını erteleme kararı ve değişmezlik soruları açık kalır.

## Yardımcı dönüşü ve gereken bilgi

Görünürlüğü açık olan, hedefi ve gövdesi bilinen inherent yardımcı da incelenebilir. Özel ama gövdesi eksik bir yardımcı, özel trait veya çözümlenmemiş dinamik çağrı için aynı sonuç çıkarılamaz. Hedefin kesin olması da tek başına arkadaş sonucu değildir: dönüşler, argümanlar ve alıcının çağıranla ilişkisi gerekir. Dinamik çağrının tüm olası hedefleri aynı köken özetini veriyorsa bu bilgi ileride kullanılabilir; burada böyle bir çözümleyici kurulmadı.

İlk uygulama kendi gerçek `self` alıcısındaki bilinen yardımcıyla sınırlanabilir. Yabancı nesnenin açık getter'ını kendi yardımcısı gibi değerlendirip alanını çağıranın alanı saymak uygun değildir. Mevcut çekirdeğin `running_helper_at`/`give_back` köprüsü dar, aynı dosyalı deney için zaten vardır; bu bilgileri elle vermekle geçen bir test yeni Rust davranışının kanıtı olmaz.

| Gerekli dil bağımsız bilgi | Somut neden ve en küçük kullanım |
|---|---|
| Üye erişiminin alıcı tipinin ve temsil kapsamının dışına taşıp taşımadığı; tip adlandırılabilirliğinden ayrı bilinen/kapalı/eksik ayrımı | `Reader` erişimi ile kardeş modülde reddedilen erişim aynı `is_private` değerine sığmaz. `returned_wallet` ise tip adı kapalıyken üyenin dışarıdan kullanılabildiğini gösterir. Adaptör kapsamları, ad yollarını ve açık API imzalarından gelen değerler üzerindeki üye erişimini ayrı çözer; domain iş kararını verir. Tip kimliği için dışarıdan yazılabilir ad şart koşulmaz. Tam modül/import ağacını domain'e taşımaya gerek yoktur. |
| Gerçek alıcının bulunması | Trait çağrısının `<Type as Trait>::method(&value)` yazımı alıcısını gizleyebilir. Alıcısız ilişkili fonksiyon da görünürlükten bağımsızdır. `is_static` bayrağına sözdiziminden karar verilmemelidir. |
| Çağrının çözümlenmiş hedefi ve mevcut gövde bağı; bilinmeyen/çok hedefli durum | Açık inherent yardımcı ile trait varsayılanı aynı görünürlükte farklı kanıt taşır. Bir hedef bilinse de gövdesi bulunmayabilir. Farklı dosya/`impl` bloklarına geçerken gövde kimliği satır/ad eşlemesinin yerine eklenmelidir. |
| Alıcı ve argümanların dönüş kökenine bağı | Özel `private_pick` farklı referanslar verir; başka örneğin alanı çağıranın alanı değildir. Mevcut köprü alanları ve dönüş modeli önce kullanılmalı; yalnız somut senaryonun eksikliği gösterdiği bilgi eklenmelidir. |

Bunlar sözleşme ihtiyaçlarıdır; yeni API veya domain kuralı uygulanmadı. Rust anahtar sözcükleri, trait/modül çözümlemesi adaptörde kalır. Adaptörün son `Object`/`Hybrid` etiketini seçmesi önerilmez. `is_public` üzerinden Java eşitlik sezgisini Rust'a aktarmak da bu çalışmanın kapsamına girmez.

## Sonraki odaklı kabul testi

Bu araştırmaya ait öneri: **`a_restricted_method_callable_from_another_type_can_contribute_behavior`**. Çağıran arayüzünden tek bir alıcılı, hesaplayan metot taşıyan tip kurulsun. Eski `is_private` bayrağı özel kalsın; ek erişim kanıtı metodun alıcı tipi dışından çağrılabildiğini göstersin. Setter ve dışarı açık ham alan olmasın. `Types::classify` için beklenen değer `Object` olsun. Böylece yalnız davranışın görünürlük yüzünden kaybolması sınanır; hibrit, değer veya yardımcı özeti kararı eklenmez.

Önce gerekli ek bilgi derlenebilir biçimde taşınsın. Davranış testi, derleme hatasıyla değil hâlâ elenen davranış nedeniyle kırılmalı; sonra en az sınıflandırıcı değişikliğiyle geçmelidir. Mevcut Java özel metot testi, ek bilgi verilmediğinde eski sonucunu korumalıdır. [Önceki açık alanlı hibrit kabul testi](rust-classification.md#sonraki-odaklı-kabul-testi) ayrı iştir. Rust kaynak adaptörü seçilince bu erişim bilgisinin gerçekten üretildiği ayrıca sınanmalıdır.

Açık kalan ürün kararları: modül içi ham erişim için ek uyarı, saf uygulama yardımcısının davranışa katkısı, çok hedefli çağrı özetlerinin birleştirilmesi ve eksik analiz raporu. Değişmezlik/değer, genel trait seçimi, sahiplik ve mevcut Java getter/akıcı çağrı/konteyner soruları bu araştırmayla kapanmaz.

## Yeniden çalıştırma

Taban: macOS ARM64, Rust 1.99.1, rustfmt 1.10.0-stable, clippy 0.1.99. İnceleme düzeltmesinden sonra 2015, 2018, 2021 ve 2024 edition'larının her birinde `modules` 9, `helpers` 8, gerçek ikinci crate `consumer` 5 test yeniden geçti: **22 test/edition**, atlanan yok. Sağlayıcı hem normal `rlib` hem lint hedefi olarak yeniden derlendi. Dört edition'da bütün 16 `.rs` dosyasının biçimi ve derlenebilir dört hedefin lint'i geçti. Olumsuz örnekler lint başarısı sayılmadı.

Repo kökünden olumlu koşular:

```sh
mkdir -p target/rust-visibility
for edition in 2015 2018 2021 2024; do
    rustc --edition="$edition" --crate-name visibility_library --crate-type rlib -D warnings examples/rust-visibility/library.rs -o "target/rust-visibility/libvisibility_library-$edition.rlib" || exit 1
    for sample in modules helpers; do
        rustc --edition="$edition" --test -D warnings "examples/rust-visibility/$sample.rs" -o "target/rust-visibility/$sample-$edition" || exit 1
        "target/rust-visibility/$sample-$edition" || exit 1
        clippy-driver --edition="$edition" --test -D warnings "examples/rust-visibility/$sample.rs" -o "target/rust-visibility/clippy-$sample-$edition" || exit 1
    done
    rustc --edition="$edition" --test -D warnings --extern "visibility_library=target/rust-visibility/libvisibility_library-$edition.rlib" examples/rust-visibility/consumer.rs -o "target/rust-visibility/consumer-$edition" || exit 1
    "target/rust-visibility/consumer-$edition" || exit 1
    clippy-driver --edition="$edition" --test -D warnings --extern "visibility_library=target/rust-visibility/libvisibility_library-$edition.rlib" examples/rust-visibility/consumer.rs -o "target/rust-visibility/clippy-consumer-$edition" || exit 1
    clippy-driver --edition="$edition" --crate-name visibility_library --crate-type rlib -D warnings examples/rust-visibility/library.rs -o "target/rust-visibility/libclippy_visibility-$edition.rlib" || exit 1
    rustfmt --edition "$edition" --check examples/rust-visibility/*.rs examples/rust-visibility/denied/*.rs || exit 1
done
```

Olumsuz örnekler düzeltmeden sonra her edition'da aşağıdaki tanıyla yeniden reddedildi: **12 ret/edition**, toplam 48. Tanıdaki yol/satır ve açıklama metni sürüme bağlıdır; deney hata kodunun tam eşleşmesini denetledi.

| `denied/` dosyası | Beklenen tanı |
|---|---|
| `sibling_field.rs` | `E0616`: `field balance of struct Wallet is private` |
| `sibling_method.rs` | `E0624`: `method balance is private` |
| `pub_super.rs`, `pub_in.rs` | `E0603`: `function amount is private` |
| `restricted_ancestor.rs`, `external_hidden_path.rs` | `E0603`: `module hidden is private` |
| `external_returned_type.rs` | `E0603`: `module unnameable is private` |
| `nonancestor_scope.rs` | `E0742`: `visibilities can only be restricted to ancestor modules` |
| `external_crate_method.rs` | `E0624`: `method crate_balance is private` |
| `external_private_trait.rs` | `E0603`: `trait LocalBalance is private` |
| `trait_method_visibility.rs` | `E0449`: `visibility qualifiers are not permitted here` |
| `impl_scope.rs` | `E0624`: `method amount is private` |

Yukarıdaki sağlayıcı derlendikten sonra, `edition` dört edition'dan biri ve `sample` tablodaki uzantısız dosya adı olacak şekilde kullanılan komut:

```sh
rustc --edition="$edition" -D warnings --error-format=json --extern "visibility_library=target/rust-visibility/libvisibility_library-$edition.rlib" "examples/rust-visibility/denied/$sample.rs" -o "target/rust-visibility/denied-$sample-$edition" 2> "target/rust-visibility/denied-$sample-$edition.jsonl"
```

Komut başarısız olmalıdır; JSON tanısındaki hata kodu tabloda verilendir. Eksik `--extern` veya bozuk derleyici kurulumu kabul edilen ret değildir. Kalıcı olumsuz dosyalar doğrudan tekrar derlenebilir. Bu retler dil öğrenme kanıtıdır; uygulanmış analiz davranışına ait kırmızı TDD testleri değildir.

İnceleme düzeltmesinde 2024 edition'ında şu geçici mutasyon deneyleri çalıştırıldı; beklentiler değiştirilmedi. Bütün değiştirilmiş kaynaklar `-D warnings` ile derlendi. Kırmızı sonuçlar derleme hatası değildir.

| Deney | Sonuç |
|---|---|
| Eski yardımcı testleri + `from_public_source` içinde yanlış `source.source()` çağrısı | 7 geçti; inceleyicinin bulduğu kaçan mutasyon yeniden üretildi. |
| Güçlendirilmiş yardımcı testleri + aynı yanlış çağrı | 7 geçti, 1 assertion ile kırıldı; `Replaced` için beklenen diğer cüzdan dönmedi. |
| Güçlendirilmiş yardımcı testleri + doğru `source.selected()` | 8 geçti. Asıl kaynak doğru tutuldu. |
| Yeni tüketici testi + yazmadan sonra eski 50 değerini veren değiştirilmiş açık metot | 1 assertion ile kırıldı; alan 70 iken metottan beklenen 70 yerine 50 alındı. Doğru sağlayıcıyla test dört edition'da geçti. |

Geçici kaynaklar ve kayıtlar `target/rust-visibility/review-fixes` altında tutuldu. Yardımcı deneylerinde `sample`, sırasıyla `original_helpers_wrong_call`, `strengthened_helpers_wrong_call`, `strengthened_helpers_correct_call` olacak şekilde kullanılan komutlar:

```sh
rustc --edition=2024 --test -D warnings "target/rust-visibility/review-fixes/$sample.rs" -o "target/rust-visibility/review-fixes/$sample"
"target/rust-visibility/review-fixes/$sample"
```

Yeni tüketici testinin geçici sağlayıcıyla kontrolü:

```sh
rustc --edition=2024 --crate-name visibility_library --crate-type rlib -D warnings target/rust-visibility/review-fixes/stale_unnameable_method.rs -o target/rust-visibility/review-fixes/libstale_unnameable_method.rlib
rustc --edition=2024 --test -D warnings --extern visibility_library=target/rust-visibility/review-fixes/libstale_unnameable_method.rlib examples/rust-visibility/consumer.rs -o target/rust-visibility/review-fixes/consumer_stale_unnameable_method
target/rust-visibility/review-fixes/consumer_stale_unnameable_method --exact a_returned_unnameable_value_allows_public_method_calls_and_field_writes
```

Bu mutasyon kontrolleri örneğin denetimlerini sınar; Rust derleyicisinin davranışını değiştirdiğimiz veya analiz aracını uyguladığımız anlamına gelmez. Önceki sürümdeki 20 genel mutasyon deneyi bu düzeltmede yeniden çalıştırılmadı. İlk sürümdeki edition biçim farkı erken dönüşle giderilmişti; bu düzeltmede ek kaynak düzenlemesi gerekmedi. Workspace/Maven ve mevcut çekirdek/Java testleri yeniden çalıştırılmadı. Kaynak incelemesi ve dil deneyleri Rust analiz raporu üretildiğini göstermez.
