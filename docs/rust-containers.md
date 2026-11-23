# Rust kapsayıcılarının içerik tipi ve kökeni

**Öneri:** İçerik, son tip argümanından değil çözülmüş işlemden ve rolünden seçilsin. Dış kapsayıcı, içerik tipi, callback girdisi/sonucu ve nesne kökeni ayrı kalsın. Bilinen boşluk, bilinmeyen bilgi değildir. Bu çalışma adaptör veya genel akış çözümleyicisi uygulamaz; kapsayıcı muafiyetleri için yeni LoD kararı vermez.

İncelenen sürüm `46b0820`, tarih 23 Kasım 2026. [Karar listesindeki](../decision.md) kapsayıcı araştırmasıdır. Üretim, karar listesi, Cargo dosyaları ve önceki araştırmalar değişmedi. [Kapsam](rust-support.md), [sınıflandırma](rust-classification.md), [görünürlük](rust-visibility.md), [değişebilirlik](rust-mutability.md), [metot seçimi](rust-method-resolution.md) ve [köken](rust-origins.md) sınırları korunur.

## Mevcut köprülerin gerçek anlamı

| Yol | Bugünkü davranış ve aktarım sınırı |
|---|---|
| [`Types::element_type`](../crates/core/src/domain/classification.rs) | Dizinin elemanını veya platformun doğrudan `Container` saydığı sınıfın son tip argümanını alır. `element_type_of`, çözülmüş üst tipler arasında ilk dizi/platform kapsayıcısını bulup aynı seçimi yapar. İşlem, başarı/hata rolü veya ilişkili `Item` bilgisi kullanmaz. |
| [`Subject::element`](../crates/core/src/domain/method/subject.rs) | Yukarıdaki tip seçimini yapar; dış öznenin arkadaş/yabancı durumunu ve `keys`/`values` köken metadatasını korur. İç nesnenin bağımsız kökenini çözmez. Bu iki `Option<Standing>` alanında `None`, bilinen anahtar/değer kökeni yok demektir; boş harita veya boş içerik kanıtı değildir. |
| [Callback besleme](../crates/core/src/domain/method/expression.rs) | `fed_by_elements` dış eleman bağlamını, `fed_by_receiver` alıcıyı, `fed_by_argument_elements` kardeş argümanı seçer; ardından **seçilen kaynağın elemanı** alınır. `fed_by_receiver`, her durumda alıcının kendisini callback'e vermek değildir. Bağ yoksa alıcının kökeni ve bilinmeyen tip kullanılır. `declaring`/`resolving`, callback parametresinin tipini ayrıca tamamlar; köken üretmez. |
| [`Call::returning` / `returning_from`](../crates/core/src/domain/method/call.rs) | İlki sonuç tipidir. İkincisi, belirtilen işlevsel argümanların ürettiği ilk kullanılabilir `Standing` bilgisini alır; yalnız alıcı ve sonuç kapsayıcıysa çağrı sonucu kökeninde kullanır. Bütün alternatifleri birleştirmez; sıradan argümanı geri verme veya alanların alias özeti değildir. |
| [`keyed_by` / `valued_by`](../crates/core/src/domain/method/arguments.rs) | İşlevsel argümanın ürettiği kökeni, sıradan argümanda ise mevcut `keys`/`values` metadatasını alır. Düz argümanın kendi kökenini otomatik anahtar/değer yapmaz; tip seçmez. Birden fazla kökeni tam birleştiren model değildir. |
| [`yielding_keys` / `yielding_values` / `keeping_entries`](../crates/core/src/domain/method/call.rs) | İlk ikisi bilinen alıcı anahtar/değer kökenini sonuç kökeni yapabilir. Sonuncusu ikili metadatayı sonuca taşır. Tuple bileşeni tiplerini veya her tekil kayıt eşleşmesini çözmez. Diğer durumda mevcut çağrı politikası çalışır; kapsayıcı alıcının durumunu koruma kuralı ayrıca vardır. |
| [Java eşlemesi](../crates/java-source/src/mapping/links.rs), [Call kurulumu](../crates/java-source/src/mapping/expressions.rs) | `Declared`, imza/tip değişkenlerinden callback besleme ve sonuç bağlarını çıkarır. `sole_element` çok argümanlı sınıfı eleman beslemesi için reddeder; `map_variables` iki tip argümanlı kapsayıcıda anahtar/değer konumu arar. `yield_of` bildirimden hangi rolün döndüğünü çıkarır. Bu Java çıkarımı `Result<T,E>` veya üç argümanlı HashMap için kopyalanamaz. |

`TypeRef`, tuple bileşeni veya ilişkili tip projeksiyonu için tam Rust tipi değildir. Adaptör önce gerçek `Iterator::Item`, referans ve tuple tiplerini çözer; domain'in kullandığı nesne tipini ve gerekiyorsa bileşen bağlarını verir. Rust generic listesini keserek son argümanı yapay biçimde içerik yapmak önerilmez.

## Önceki kanıt

[`rust-semantics.rs`](../examples/rust-semantics.rs) içindeki `a_result_iterator_yields_the_success_type_and_not_the_last_type_parameter`, `Result<Wallet, String>::iter` öğesinin başarı rolünden geldiğini; `an_iterator_item_type_can_differ_from_its_generic_parameter`, map sonrasındaki ilişkili öğenin değişebildiğini gösterir. [Dizi edition deneyleri](rust-method-resolution.md) ödünç/sahip olunan `IntoIterator` farkını zaten ayırır.

[Köken deneyleri](rust-origins.md), kopyalanan referans ve yeni dış değerin mevcut alt nesneyi taşıyabilmesini; [Rc/Arc ve dönen tutamaç deneyleri](rust-mutability.md) ortak durumun korunmasını doğrular. Bunlar kopyalanmadı veya yeniden çalıştırılmadı. Yeni testler bu dil gerçeklerinden LoD sınıflandırma etiketi üretmez.

## Yeni dil ve kütüphane kanıtı

[Option](https://doc.rust-lang.org/std/option/enum.Option.html) ve [Result](https://doc.rust-lang.org/std/result/enum.Result.html) belgelerinde `as_ref`/`as_mut` dış değeri ödünç referans taşıyan yeni değere yansıtır. Result'ın başarı ve hata rolleri ayrıdır: `map` başarıyı, `map_err` hatayı dönüştürür; `and_then` başarı callback'inin verdiği Result'ı kullanır, mevcut hatada callback'i çalıştırmaz. Bunlar kütüphane sözleşmeleridir; arkadaşlık kararı değildir.

[`roles.rs`](../examples/rust-containers/roles.rs) altı test içerir:

| Ayrım | Bağımsız gözlem ve karşı örnek |
|---|---|
| Option ödünç / sahip olunan içerik | `option_projections_and_consumption_keep_the_boxed_wallet`: `Option<&Box<Wallet>>`, `Option<&mut Box<Wallet>>` ve tüketilerek alınan `Box<Wallet>` aynı canlı heap hedefini korur. Yazmadan önce 50, sonra 30 görülür. Dış değerin tüketilmesi yeni iç cüzdan oluşturmaz. |
| Bilinen None / bilinmeyen tip | `a_typed_none_skips_its_mapping_callback`: girdi `Option<&Wallet>`, sonuç `Option<i32>` olarak bilinir; sonuç None ve callback sayısı 0'dır. Yok öğe için hayalî köken veya callback sonucu yoktur. |
| Result'ın iki projeksiyonu | `result_projections_keep_success_and_error_targets_separate`: başarı ve hata aynı Box tipinde, başlangıç tutarları da 50'dir; adresleri farklıdır. `as_ref` doğru hedefleri verir. `as_mut` ile başarıya 20 yazıldıktan sonra hata hâlâ 50 **kaydedilir**, sonraki hata yazmasından sonra 30 olur. |
| map / map_err | `map_and_map_err_run_only_for_their_respective_result_roles`: callback'ler yeni `(referans, tutar)` tuple'ı üretir. Başarı işlendikten sonra çağrı sayıları `(1,0)`, hata da işlendikten sonra `(1,1)` olur. Çıktıların referansları doğru ayrı girdilere bağlıdır; aynı 50 tutarı yanlış kaynağı gizlemez. |
| and_then / önceki hata | `and_then_can_replace_success_and_keeps_an_existing_error`: başarı callback'i başka hata referansı verebilir; zaten hatalı girdi kendi hata referansını korur. İki yol sonrasında callback sayısı 1'dir; eşit tutarlı farklı referanslar ayırt edilir. `map` bu Result sonucunu aynı şekilde düzleştirmez. |
| Err / boş başarı iterasyonu | `an_error_has_no_success_items_and_keeps_its_known_error`: `std::result::Iter<'_, Wallet>` ve `Vec<&Wallet>` tipleri bilinir; başarı öğesi yoktur, hata referansı ve 50 tutarı korunur. Başarı içeriğinin yokluğu hata rolünün veya başarı tipinin bilinmediği anlamına gelmez. |

[HashMap](https://doc.rust-lang.org/std/collections/struct.HashMap.html) için üçüncü `S` hasher'dır, değer `V`'dir. Bu deneyin üç açık tip argümanında son argüman `S` olur. `keys`/`values` farklı roller verir; `iter` öğesi `(&K,&V)` tuple'ıdır ve hash sırası garantisi yoktur. [Iterator](https://doc.rust-lang.org/std/iter/trait.Iterator.html) öğesi ilişkili `Item` bilgisidir; `filter` öğe tipini korur, `map` callback çıktısını öğe yapar. Aynı öğe tipi aynı köken demek değildir.

[`projections.rs`](../examples/rust-containers/projections.rs) altı test içerir:

| Ayrım | Bağımsız gözlem ve karşı örnek |
|---|---|
| Anahtar / değer / hasher | `a_hashmap_has_distinct_key_value_and_hasher_roles`: `HashMap<String, &Wallet, BuildHasherDefault<DefaultHasher>>`. `get_key_value` sonucu `(&String, &&Wallet)`, hasher ayrı tiptedir. Anahtarın saklanan String buffer'ı ve değer referansı doğrulanır; `keys`/`values` bunları ayrı verir. Değerden yazılan 30 aynı cüzdanda görülür. |
| Tuple öğe / sıra | `hashmap_iteration_yields_key_value_pairs_without_relying_on_order`: iki öğe `Vec<(&String,&Box<Wallet>)>` olarak alınır. Her anahtarın başlangıçta kaydedilen doğru heap hedefi aranır; ilk/ikinci hash sırası varsayılmaz. İki cüzdanın tutarı da 50'dir. |
| Koruyan filter / collect | `filtering_keeps_selected_items_when_collecting_borrowed_content`: iki siparişten kodu 2 olan seçilir. `Vec<&Order>` içindeki referans doğru mevcut sipariştir; o cüzdana yazınca kaynak tutarlar `(50,30)` olur. Yeni Vec iç nesneyi yeni yapmaz. |
| Aynı Item, farklı köken | `mapping_can_keep_the_item_type_and_change_its_origin`: `Once<&Wallet>` girdisi başka cüzdan referansına map edilir; çıktı yine `Vec<&Wallet>`tir. Yanlışlıkla eski referansı korumak adres/durum assertion'larını bozar. |
| Yeni Item, eski alt nesne | `mapping_new_outer_items_can_keep_existing_aliased_descendants`: map yeni `Snapshot` değerleri, collect yeni Vec üretir. Her Snapshot mevcut doğru cüzdanı gösterir; birinden yazınca kaynaktaki ikinci cüzdan değişir. Dış öğe oluşturma, alt nesne bağımsızlığı değildir. |
| Çözülmüş kimlik / kısa ad | `an_alias_uses_standard_option_but_a_project_option_has_its_own_map`: `Maybe` standart Option alias'ıdır. Projedeki başka `Option` tipinin `map` metodu callback yerine sıradan argüman alır ve onu döndürür. Standart çağrı ilk cüzdanı, proje çağrısı aynı tutarlı ikinci cüzdanı verir; yalnız standart callback çalışır. |

[Tip alias'ı](https://doc.rust-lang.org/reference/items/type-aliases.html) mevcut tipe yeni ad verir; ayrı proje tipi aynı yazımla ayrı bildirimdir. Adaptör kısa `Option`, `Result`, `HashMap` veya `map` adına göre standart davranış seçmemeli; alias açılımı, crate/tip/metot kimliği ve seçilmiş trait uygulaması çözülmelidir. Açık `std` yazımı bulunmaması standart kimliği kaybettirmez. Standart kimlik de callback gövdesinin dönen nesnesini kendiliğinden açıklamaz.

Kaynaklar canlı resmî belgelerdir; deneyler aşağıdaki yerel sürüme aittir. Tür/adres/state tanıkları yalnız bu programları doğrular. Stack konumu değişikliği semantic nesne kimliği veya sınıflandırma kanıtı sayılmaz; canlı heap/ref hedefleri karşılaştırılır, ham adres dereference edilmez.

## En küçük aktarım önerisi

| Ayrı olgu | Mevcut köprü / gerekli en az ek bilgi |
|---|---|
| Dış tip ve seçilmiş işlem | Gerçek çözülmüş `Call` alıcısı/hedefi ve `returning` kullanılsın. Standart işlem kimliği adaptörde çözülür; bütün aynı adlı proje tiplerine platform istisnası uygulanmaz. |
| Kullanılan içerik tipi | Callback parametresi için mevcut `declaring`/`resolving` yeterlidir. Genel eleman tüketiminde son-argüman varsayımı yetmiyorsa, kullanılan projeksiyonun **açık çözülmüş içerik tipi** eklenmeli; verilmediğinde eski davranış korunmalıdır. `Item` tuple ise gereken bileşen tipi/rolü ayrıca bilinsin; sahte bir iki-parametreli harita tipi yapılmasın. |
| Rol ve köken | Haritanın gerçek anahtar/değer kökeninde mevcut köprüler anlamları uygunsa kullanılsın. Result başarı/hatası map anahtar/değeri diye yeniden adlandırılmasın. İşlemin seçtiği içerik rolüne ait köken ve projeksiyon bağı, gerektiğinde dil bağımsız ek bilgi olarak taşınmalı; Rust variant adları domain'e yerleştirilmemelidir. |
| Callback girdisi ve çıktısı | Beslenen içerik tipi, korunan/başka hedefe yansıtılan/yeni dış değer üreten callback sonucu ayrılsın. Uygun tek akışta `returning_from` kullanılabilir; bu köprü her role veya tuple bileşenine ait bütün alias'ları kanıtlamaz. `filter` ve bilinmeyen gövdeli `map` aynı özet sayılmasın. |
| Varlık ve eksiklik | İçerik için bilinen varlık, bilinen yokluk ve bilinmeyen durum ayrı olsun. Tipin veya köken özetinin eksikliği bilinen boşluk değildir. Callback'in bu çalıştırmada atlanmasıyla statik analizde hangi yolların inceleneceği ayrı ürün kararıdır. Bugünkü `keys/values: None` veya `Classification::Unknown` bu üç ayrımın yerine geçmez. |

Bu ek olgular sözleşme ihtiyacıdır; yeni API uygulanmadı. İlk callback tip testi mevcut köprüyle sınanabilir. Genel rol grafiği, kapsayıcı içindeki her nesnenin heap modeli veya bütün iterator adaptörlerinin özeti gerekmiyor. Tür, rol veya köken bilinmiyorsa eksiklik saklanmasın; `New`, `Literal` veya başarı rolü uydurulmasın. `None`/Err durumunda kapsayıcı ve içerik **tipleri** yine bilinebilir; bilinmeyen köken yeni nesne muafiyeti değildir.

Arkadaşlık/istisna kararını domain vermeye devam etsin. Kapsayıcıyı ödünç almak, tüketmek veya yeni dış kapsayıcı toplamak mevcut iç nesneyi oluşturulmuş yapmaz. Buna karşılık her map sonucunu yabancı veya bütün standart kapsayıcıları otomatik `Container` muafiyeti saymak da bu araştırmadan çıkmaz. Mevcut son-argüman ve çağrı istisnaları Java için değişmedi.

## Sonraki dar gerçek-adaptör kabul testi

`result: Result<Customer, Failure>` üzerinde `result.iter().for_each(...)` ön yüzün callback çözümünü sınayabilir. Ancak [Result::iter](https://doc.rust-lang.org/std/result/enum.Result.html), [result::Iter<'_, Customer>](https://doc.rust-lang.org/std/result/struct.Iter.html) üretir; bunun `Item` tipi `&Customer`, son tip argümanı zaten Customer'dır. Dolayısıyla bu örnek son-argüman kestirmesinin yanlışlığını ayırt etmez ve onun yüzünden kırmızı olacağı vaat edilemez.

Sonraki test önerisi: **`a_key_iterator_feeds_its_key_type_instead_of_its_value_type`**. Gerçek Rust kaynak okuyucusuna `entries: &HashMap<Key, Failure>` alan bir fonksiyon verilsin:

```rust
entries.keys().for_each(|key| key.activate());
```

`Key` ve `Failure` ayrı proje tipleri olsun; `activate` yalnız Key'de bulunsun. [HashMap::keys](https://doc.rust-lang.org/std/collections/struct.HashMap.html), `std::collections::hash_map::Keys<'_, Key, Failure>` döndürür. [Keys'in Iterator uygulamasında](https://doc.rust-lang.org/std/collections/hash_map/struct.Keys.html) `Item = &'a K` olduğundan `for_each` callback girdisi `&Key`'dir. Buna karşılık bu iteratorün son tip argümanı Failure'dır. Mevcut `a_hashmap_has_distinct_key_value_and_hasher_roles` testi `keys().next()` sonucunun değer referansından ayrı anahtar tipi/kimliği olduğunu zaten doğrular; yeni kaynak eklenmedi.

Aktarım zinciri açık olmalı: `Call::returning`, `keys` sonucunun doğru `Keys<Key, Failure>` tipini korur; adaptör ilişkili `Item = &Key` bilgisini çözer; `Lambda::resolving`/`declaring` domain'e çağrı hedefi nesne tipi Key'i verir. Generic argümanlar bu amaçla yer değiştirmemelidir. Bu iterator `Container` olarak tanındığında `Subject::element` → `Types::element_type_of` → `element_type` kestirmesi, açık callback tipi verilmezse son argüman Failure'ı seçer. `Subject::declared_as` ile verilen çözülmüş Key tipi bu varsayılanı düzeltebilir. `yielding_keys` yalnız köken metadatasını taşır; callback tipini düzeltmez. Genel iterator sınıflandırma politikası bu testle kararlaştırılmaz.

Bağımsız beklenti, gerçek Rust girdisinde `&Key`, domain callback modelinde Key ve çözülen `activate` hedefinde Key bildirimidir. Son argümanı callback tipi diye kullanan yanlış eşleme Failure veya çözülemeyen hedef üretir ve bu beklentiyi bozar; yalnız kaynağın derlenmesi yeterli değildir. Beklenen tip adaptöre test girdisi olarak enjekte edilmemelidir. Gerçek Rust adaptörü henüz yoktur; bu kabul testi çalıştırılmadı. İleride derlenebilir eşlemenin yanlış davranışı kırmızı kanıt olabilir; eksik crate/parser veya derleme hatası olamaz. Elle kurulmuş domain lambda'sı kaynak eşlemesini doğrulamaz. Genel `for` kökeni, hata kanalı, bütün callback akışı veya LoD kapsayıcı muafiyeti açık kalır.

Açık kararlar: rol başına kökenin en küçük temsili; tuple bileşenlerinin aktarımı; bilinen yokluğun statik callback analizine etkisi; eksik analiz raporu; standart kapsayıcı/platform politikası. Konteyner değişikliği, factory/akıcılık, oluşturulan nesnenin alanı ve Value soruları açık kalır. Genel closure/iterator/async/makro ve kontrol akışı analizleri sonraki satırlardır.

## Doğrulama

İnceleme düzeltmesinde yalnız bu belge değişti: Result örneğinin kapsamı daraltıldı, ayırt edici Keys tipi/Item ve domain aktarımı birincil kaynaklarla yeniden izlenerek kabul önerisi düzeltildi. Dört `.rs` kaynağı değişmedi; test, olumsuz derleme, biçim, lint veya bozulma deneyi yeniden çalıştırılmadı. Aşağıdaki sayılar önceki, bağımsız incelemeden geçmiş koşulardır.

macOS ARM64, Rust 1.99.1, rustfmt 1.10.0-stable, clippy 0.1.99. 2015/2018/2021/2024 edition'larının her birinde **roles 6 + projections 6 = 12 test geçti**, toplam 48 koşu, atlanan yok. Dört `.rs` dosyasının biçimi ve iki olumlu test hedefinin Clippy denetimi dört edition'da geçti; lint izni eklenmedi.

İlk kurulumda hata öğesi toplanmış referanslar yaşarken tüketildiği için ödünç hatası görüldü; hata projeksiyonu paylaşılan ödünçle okunarak düzeltildi. Bu kırmızı TDD kanıtı sayılmadı. İlk Clippy önerileri, gerçek tuple dönüşü ve harita değerinde referans kullanımıyla giderildi; araştırılan map/projeksiyon işlemleri korunup dört edition yeniden denetlendi.

Repo kökünden olumlu komutlar:

```sh
mkdir -p target/rust-containers
for edition in 2015 2018 2021 2024; do
    for sample in roles projections; do
        rustc --edition="$edition" --test -D warnings "examples/rust-containers/$sample.rs" -o "target/rust-containers/$sample-$edition" || exit 1
        "target/rust-containers/$sample-$edition" --quiet || exit 1
        clippy-driver --edition="$edition" --test -D warnings "examples/rust-containers/$sample.rs" -o "target/rust-containers/clippy-$sample-$edition" || exit 1
    done
    rustfmt --edition "$edition" --check examples/rust-containers/*.rs examples/rust-containers/denied/*.rs || exit 1
done
```

| Olumsuz kaynak | Dört edition'da amaçlanan tanı |
|---|---|
| [`result_error_as_success_item.rs`](../examples/rust-containers/denied/result_error_as_success_item.rs) | `E0308`: `Result<u32,String>::iter().next()` için `Option<&String>` beklentisi reddedilir; öğe başarı rolündeki `&u32`'dir. |
| [`hashmap_hasher_as_value.rs`](../examples/rust-containers/denied/hashmap_hasher_as_value.rs) | `E0308`: haritanın `get` sonucu hasher referansı olamaz; değer `u32`'dir. |

**2 ret/edition**, toplam 8; JSON hata kodu kümesi yalnız `E0308` idi. Hatalı dosya/araç veya başka kod kabul edilen kanıt değildir. `edition` dört kapsamdan biri, `sample` tablodaki uzantısız ad olacak şekilde:

```sh
rustc --edition="$edition" -D warnings --error-format=json "examples/rust-containers/denied/$sample.rs" -o "target/rust-containers/denied-$sample-$edition" 2> "target/rust-containers/denied-$sample-$edition.jsonl"
```

İlk koşuda geçen 12 test için 2024 edition'ında **12 ayrı geçici bozulma** `-D warnings` ile derlendi ve hedef assertion ile kırıldı. İç Box bağımsızlaştırıldı; None yerine fallback içerik verildi; başarı/hata projeksiyonu veya map girdisinin rolü değiştirildi. Mevcut hata and_then callback'ine sokuldu; hata başarı öğesi üretti. Harita değeri/anahtar eşleşmesi, filter seçimi, map kökeni ve yeni dış öğenin alt hedefi değiştirildi; proje `map` metoduna yanlış girdi döndürtüldü. Beklentiler değişmedi.

Geçici kaynak/kayıtlar `target/rust-containers/faults` altında. Her `sample` ve ilgili `test_name` için kullanılan komutlar:

```sh
rustc --edition=2024 --test -D warnings "target/rust-containers/faults/$sample.rs" -o "target/rust-containers/faults/$sample"
"target/rust-containers/faults/$sample" --exact "$test_name"
```

Bu kontroller yanlış hedef/rol/varlık kullanımını örneklerde ayırt eder; standart kütüphane semantiğinin değiştirildiği veya analiz çözümleyicisi uygulandığı iddia edilmez. Önceki araştırmalar ve geniş çekirdek/workspace/Maven koşuları yeniden çalıştırılmadı. Çıktılar bu worktree'nin `target/rust-containers` dizinindedir.

Test okunabilirliği ayrı, salt okunur bir geçişte incelendi; bulgu yok. On iki ad gözlenen rol/köken davranışlarıyla uyumlu; kur/işlet/denetle bölümleri boş satırla ayrılıyor ve beklentiler bağımsız. Sonraki çağrının/yazmanın gizleyebileceği sayaçlar ve ara durumlar önce kaydediliyor. Harita sırası varsayılmıyor. İnceleme kaynak değiştirmedi.
