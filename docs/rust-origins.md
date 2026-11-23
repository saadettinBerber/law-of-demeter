# Rust taşıma, ödünç ve bağlama kökenleri

**Öneri:** Sahipliğin, bağlamanın veya erişim yetkisinin değişmesi nesnenin kökenini yenilemesin. Yeni dış değer ile taşıdığı mevcut nesneler ayrı izlensin. Tip, seçilmiş çağrı hedefi ve bilinen/bilinmeyen köken ayrı bilgiler olsun. Bu çalışma Rust adaptörü veya genel alias çözümleyicisi uygulamaz.

İncelenen sürüm `1eef5a8`, tarih 23 Kasım 2026. [Karar listesindeki](../decision.md) taşıma/ödünç araştırmasıdır. Üretim, karar listesi, Cargo dosyaları ve önceki araştırmalar değişmedi. [Kapsam](rust-support.md), [sınıflandırma](rust-classification.md), [görünürlük](rust-visibility.md), [değişebilirlik](rust-mutability.md) ve [metot seçimi](rust-method-resolution.md) sınırları korunur. Rust kuralları derlenebilirliği belirler; LoD arkadaşlığını seçmez.

## Bugünkü köken yolu

| Tüketici / çağıran | Mevcut davranış ve Rust sınırı |
|---|---|
| [`Method::assign` / `assign_var`](../crates/core/src/domain/method.rs), [`Step::Assign`](../crates/core/src/domain/method/block.rs) | Sağ tarafın `Subject` bilgisini değerlendirir, yerel gerekçesiyle sarıp bağlar. Yeni yerel, kendiliğinden `Creation` değildir. Bildirilen tip kökeni değiştirmez. |
| [`Context::bind` / `local`](../crates/core/src/domain/method/subject.rs) | Yereller tek `HashMap<String, Subject>` içindedir; aynı anahtara atama önceki kaydı değiştirir. Lexical blok/kapsam kimliği yoktur. Bulunmayan yerel yabancı ve tipsizdir. Rust'ta her yeni bağlamanın kimliği ayrılmalı; aynı bağlamaya sonraki atama ise aynı kimliği kullanmalıdır. |
| [`Expression`](../crates/core/src/domain/method/expression.rs) | `Parameter`, `This`, doğrudan `Field` ve `New` arkadaş üretir. Örnek üzerindeki `FieldOf` yabancıdır; statik alan istisnadır. `Cast` tipi değiştirirken kökeni korur. `Literal` ve `Operation` arkadaş/literal sonucu üretir; bunlar bilinmeyen nesne kökeni için yer tutucu değildir. |
| [`Creation`](../crates/core/src/domain/method/expression.rs) | Argüman çağrılarını değerlendirir, dış oluşturulan nesneyi arkadaş yapar; alan başlatıcılarının nesne bağlarını saklamaz. Yeni dış struct'ın varlığı, iç referansın da yeni olduğunu anlatmaz. |
| [`Call`](../crates/core/src/domain/method/call.rs) | `returning` dönüş tipini taşır. Köken ayrıca yardımcı/konteyner bağları veya mevcut statik, akıcı, factory ve sorgu politikalarıyla belirlenir. `returning_from`, işlevsel argümanın sonucunu yalnız konteyner→konteyner akışında tüketir; sıradan argüman alias'ı değildir. |
| [Yardımcı dönüşü](../crates/core/src/domain/method.rs) | `handed_back`, bütün dönüşlerin arkadaş olmasını ve parametreye dayanmamasını ister. Çağıranın sıradan argümanını yardımcı parametresine bağlayıp geri taşıyan genel ikame yoktur. `forwarded_wallet` bu köprüyle çözülmüş sayılmaz. |
| [Java eşlemesi](../crates/java-source/src/mapping/statements.rs), [ifadeler](../crates/java-source/src/mapping/expressions.rs), [adımlar](../crates/java-source/src/mapping/steps.rs) | Yerel başlatıcı/atama `assign` adımına, yerel kullanım `Expression::local` içine gider. İç blokların adımları aynı sıraya açılır. Java [ad çözümü](../crates/java-source/src/scope.rs) kapsam çerçeveleri kullanır; buna rağmen bu domain sınırında bağ anahtarı addır. Rust gölgelemesi yalnız yazılı ada indirgenmemelidir. Alan ataması mevcut modelde alıcı/sağ tarafı değerlendirir; nesne başına alan kökenini güncelleyen bellek modeli değildir. |

[`splitting_the_chain_into_a_local_changes_nothing`](../crates/core/src/domain/method/scenarios.rs) mevcut elle kurulmuş atamanın yabancı kökenini korur. [Temel kural testleri](../crates/core/src/domain/method/rule.rs) parametre/alan/oluşturulan nesne ayrımını gösterir. Bunlar Rust sözdiziminin eşlendiği kanıtı değildir; bu çalışmada yeniden koşulmadı.

## Yeniden kullanılan kanıt

Önceki örnekler kopyalanmadı veya yeniden çalıştırılmadı:

| Ayrım | Onaylanmış örnek |
|---|---|
| Box taşıma / yeni tahsis | [`rust-semantics.rs`](../examples/rust-semantics.rs): `moving_a_handle_preserves_the_object_it_points_to`, taşıma sonrasında aynı heap adresini doğrular. Tutamacın taşınması iç nesne oluşturma değildir. |
| Sıradan değer clone / paylaşılan tutamaç clone | Aynı dosyada `cloning_an_owned_value_can_create_independent_state`, iki ayrı sayısal durumu `(50, 30)` gözler; `cloning_a_shared_handle_keeps_the_same_object`, Rc tahsisini ve değişikliğin ilk tutamaçtan görülmesini doğrular. İlk örnek bütün erişilebilir nesnelerin bağımsızlığını kanıtlamaz. |
| Rc/Arc ve dönen erişim | [`effects.rs`](../examples/rust-mutability/effects.rs): sahip sayısı/iç veri ayrımı, `Arc<Mutex<_>>`, dönen `&mut`/guard/tutamaç. Yeni erişim nesnesi mevcut korunan veriyi yeni yapmaz. |
| Sahip olunan Copy alıcısı | [`rust-classification.rs`](../examples/rust-classification.rs): `an_owned_copy_receiver_can_change_locally_and_preserve_the_original`. Yerel değişiklik veya atama, paylaşılan nesne kimliğiyle karıştırılmaz. |
| Çağrı hedefi / alıcı dönüşümü | [`selection.rs`](../examples/rust-method-resolution/selection.rs), sarmalayıcı ile gösterdiği nesnenin metodunu ayırır. Kullanıcı tanımlı `Deref` veya `clone` adlı çağrının hedefi yalnız sözcükten çıkarılmaz. |

## Yeni dil kanıtları

[Taşıma/kopyalama](https://doc.rust-lang.org/reference/expressions.html), [ödünç operatörleri](https://doc.rust-lang.org/reference/expressions/operator-expr.html) ve [alan seçimi](https://doc.rust-lang.org/reference/expressions/field-expr.html) yer, değer ve erişim kurallarını tanımlar. Bir yerden taşıma kaynağı kullanılamaz yapabilir; `&`/`&mut` o yere erişim üretir. Değer ifadesini ödünç almak ise geçici değer oluşturabilir. Bu nedenle **bilinen yere** `&mut *parent` uygulamakla `&Wallet { ... }` oluşturmak aynı işlem değildir. Keyfî `Deref` çağrısı bu basit yer kuralına indirgenmez.

[`bindings.rs`](../examples/rust-origins/bindings.rs) yalnız dil öğrenme testidir; hiçbir assertion bir analiz etiketini doğrulamaz. Kaynaklar canlı resmî belgelerdir; deney kanıtı yerel Rust 1.99.1 çalıştırmasına aittir.

| Ayrım | Bağımsız gözlem ve yakın karşı örnek |
|---|---|
| Paylaşılan ödünç → yeni yerel | `a_shared_borrow_assigned_to_another_local_keeps_its_referent`: yeni yerelin referansı aynı canlı cüzdanı gösterir, bakiye 50'dir. Aynı sayıyla ayrı bir cüzdan oluşturmak eşdeğer değildir. |
| Atama / eski referans kopyası | `reassigning_a_reference_keeps_an_earlier_copy_on_its_original_target`: `current` ikinci hücreye atanır; eski kopya ilk hücrede kalır. İkinciye yazınca `(50, 30)` görülür. Bağlama değişir; önceki referans yeniden hedeflenmez. |
| Yeniden ödünç | `a_mutable_reborrow_changes_the_wallet_and_then_releases_its_parent`: çocuk üzerinden yazılan 30, sonraki işlemden **önce** parent üzerinden de 30 kaydedilir; parent tekrar yazınca cüzdanda 20 kalır. Çocuk yaşarken parent'a yazma aşağıdaki olumsuz örnekte reddedilir. |
| Gölgeleme / kapsamdan çıkış | `leaving_a_shadowing_scope_restores_the_outer_binding`: iki cüzdanda da 50 vardır; iç `current` ikinciye, blok sonrasındaki dış `current` ilkine bağlıdır. Referans kimlikleri bu farkı sayısal eşitlikten bağımsız ayırır. |
| Destructuring / kısmi taşıma | `destructuring_moves_one_field_and_leaves_the_other_field_usable`: yalnız Box alanı taşınır; tahsis aynı kalır, yazmadan önce 50 ve sonra 30 görülür. Taşınmayan `order.label` kullanılabilir; bütün `order` değerinin kullanılması ayrı reddedilir. |
| Struct update | `struct_update_keeps_the_unspecified_reference_field_aliased`: yeni etiket `updated`, ilk etiket `original` kalır. `..original` ile taşınan/kopyalanan belirtilmemiş alan burada Copy referanstır; iki dış değer aynı hücreyi gösterir. Yazmadan önce 50, ilk değer üzerinden sonra 30 görülür. |
| Dış değeri clone etmek | `cloning_an_outer_value_can_copy_owned_data_and_share_a_descendant`: klonun String alanına ek yapmak ilk etiketi değiştirmez; referans alanından yazmak ilk değerin gösterdiği hücreyi 30 yapar. Bağımsız dış veriyle paylaşılan alt nesne birlikte bulunabilir. |
| Referans taşıyan Copy sarmalayıcı | `copying_a_reference_wrapper_keeps_its_shared_descendant`: ilk sarmalayıcı kullanılabilir kalır; kopyanın referansı aynı hücreye bağlıdır. 50'den 30'a yazma ilkinden de görülür. Copy, derin bağımsızlık değildir. |
| Girdiyi döndürme / alanı seçme | `the_same_reference_result_type_can_forward_an_input_or_project_a_field`: aynı `&Wallet` tipi ve aynı 50 değeri, ayrı cüzdan referansı ile siparişin iç cüzdanını ayırmaz. Bilinen iki gövdenin sonuçları doğru girdi/heap alanı referanslarıyla karşılaştırılır. |
| Ortak lifetime / farklı girdi | `matching_lifetime_signatures_can_return_different_input_references`: `first_wallet` ve `second_wallet` aynı `fn(&'a Wallet, &'a Wallet) -> &'a Wallet` biçiminde farklı girdileri verir. Bakiyeler eşittir; referanslar farklıdır. İmza tek başına hangi girdinin döndüğünü seçmez. |

[Pattern kuralları](https://doc.rust-lang.org/reference/patterns.html), bağlamanın varsayılan taşıma/kopyalama veya açık referans olmasını ve kısmi taşımayı açıklar. [Kapsam](https://doc.rust-lang.org/reference/names/scopes.html) yeni bağlamanın hangi kaynak bölgesinde geçerli olduğunu belirler. Örnekler düz `let` ve açık yer ifadeleriyle sınırlıdır; bütün pattern ergonomisi veya edition değişimleri sınanmadı.

[Struct update](https://doc.rust-lang.org/reference/expressions/struct-expr.html) yeni dış değeri kurarken belirtilmeyen alanları tabandan taşır/kopyalar. [Clone](https://doc.rust-lang.org/std/clone/trait.Clone.html) yeni değer üretir; referans veya akıllı işaretçinin gösterdiği veri yine paylaşılabilir. Türetilen Clone alanların Clone uygulamalarını çağırır; kullanıcı uygulaması ayrıca çözülmelidir. [Copy](https://doc.rust-lang.org/std/marker/trait.Copy.html) bit kopyasını ve ilk değerin kullanılabilirliğini anlatır, iç nesnelerin derin bağımsızlığını değil.

[Lifetime kuralları](https://doc.rust-lang.org/reference/lifetime-elision.html) referans geçerliliğini sınırlar; alias kökeni özeti değildir. Bilinen tek gövde girdiyi, bir alanı veya farklı bir geçerli referansı döndürebilir. Koşullu dönüşte hangi yolun seçileceği bilinmiyorsa tek girdi uydurulmaz; alternatif/eksik köken açık kalır. Kontrol akışı birleştirme bu çalışmada tasarlanmadı.

Adres gözlemleri anlamlarıyla sınırlıdır: taşınan Box için canlı heap tahsisi, ödünçler için aynı anda canlı ve taşınmayan referans hedefleri karşılaştırılır. Farklı stack adresleri “farklı domain nesnesi”, aynı sayısal değer de “aynı nesne” diye genellenmez. Ham adres yalnız karşılaştırılır; `unsafe` veya ham işaretçi dereference yoktur. Hücreye yazma alias'ı görünür kılar; tek başına Value/Hybrid veya LoD ihlali kararı değildir.

## En küçük aktarım ve önerilen politika

1. Doğrulanmış taşıma, yerel bağlama ve bilinen yere ödünç/yeniden ödünç için mevcut köken korunsun. Referans yetkisi veya sahipliğin değişmesi arkadaşlık kazandırmasın. Kaynağı yabancı olan nesne yeni yerelde de yabancıdır. Kullanıcı tanımlı alıcı dönüşümü için [hedef/köken bilgisi](rust-method-resolution.md) ayrıca gerekir.
2. Adaptör isim çözümlemesinde her lexical bağlamaya ayrı anahtar versin; sonraki kullanım o bağlamaya, yeniden atama aynı anahtara gitsin. Mevcut `assign`/`local` bu anahtarlarla yeterlidir; domain'e Rust kapsam ağacı eklemek gerekmez. Tanıda kaynak adı korunmalıdır. Dış bağlamanın değerine atama yapılması ile içte aynı adla yeni bağlama yapılması ayrıdır.
3. Gerçek struct oluşturma dış değer için ayrı kanıt olsun; taşınan, kopyalanan veya klonlanan alanların mevcut hedefleri kaybolmasın. Dış değerin oluşturulması, alanlarından erişilen her nesneyi oluşturulmuş yapmasın. Bunun mevcut oluşturulan-nesne-alanı politikasındaki nihai sonucu açık karardır; adaptör bu kararı kendisi seçmesin.
4. Clone/Copy adı veya işareti derin yeni nesne muafiyeti üretmesin. Bilinen standart işlem/gövde ve alan türüyle hangi hedefin korunduğu kaydedilsin. Bütün clone sonuçları yabancı veya bütün Copy tipleri Value sayılmasın. Fiziksel kopya, mantıksal değer ve LoD ilişkisi ayrı kalsın.
5. Tipin çözülmesi, hedefin çözülmesi, gövdenin bulunması ve kökenin bilinmesi ayrı durumlar olsun. Sonuç tipi bilinen fakat gövdesi eksik çağrıya `New`, `Literal` veya `Operation` verilmesin. Görünürlük/lifetime da kökeni tamamlamaz. Birden fazla olası köken varsa tek arkadaş seçilmesin; eksik analiz raporu sonraki karar adımıdır.

| Somut eksiklik | Gerekirse eklenecek en az dil bağımsız olgu |
|---|---|
| Sıradan parametreyi geri veren gövde için çağıranın girdisi bağlanmıyor. | Sonucun alıcıya veya belirli sıradan argümana alias olması ve varsa alan yolu; yalnız bilinen hedef/gövde veya standart sözleşmeyle. `Call::returning_from` başka anlamda kullanılmamalıdır. Dar bağlama kabul testi bu alanı gerektirmez. |
| `Creation` alan başlatıcı bağlarını saklamıyor. | Gerekli olduğunda belirli oluşturulan dış değerin alanına taşınan mevcut hedef bağı. Bu kanıt, alan çağrısına verilecek politika sonucundan ayrı olmalıdır; genel heap grafiği burada tasarlanmadı. |
| `Subject::standing` yalnız arkadaş/yabancı, `Unresolved` ise ad çözümüne ait. | Tip/hedef bilinse de kökenin eksik veya çok olasılıklı olduğunu açıkça taşıyabilme. `Unresolved(name)` kendi alanı/statik import'u çözebilir; genel typed-unknown köken yerine kullanılamaz. Eksiklik, başarılı oluşturma/literal kanıtına çevrilmemelidir. |

Yeni sözleşme uygulanmadı. İlk düz bağlama/ödünç dilimi mevcut `Expression`, `assign`/`assign_var`, `local` ve `Call::returning` ile sınanabilir. `resolving` tip bilgisini tamamlar; nesne kökeni vermez. Yukarıdaki eklemeler yalnız gerçek entegrasyon ihtiyacıyla yapılmalı; Rust borrow checker, bütün program heap/etki analizi veya büyük çağrı şeması domain'e taşınmamalıdır.

## Sonraki dar kabul testi

Öneri: **`a_temporary_and_reborrow_keep_the_chain_receivers_origin`**. Gerçek Rust adaptöründe aynı davranışlı `Customer` tipi için iki eşdeğer kaynak biçimi analiz edilsin:

```rust
order.customer().activate();
```

```rust
let customer = order.customer();
let temporary = &mut *customer;
temporary.activate();
```

`order: &mut Order`, benzersiz inherent `customer(&mut self) -> &mut Customer` ve sonucu kendini döndürmeyen davranış `activate(&mut self)` kullanılsın. Her iki biçimde siparişteki `customer` çağrısı serbest, dönen müşterideki `activate` aynı kök nedenle ihlal olsun; tanının yerel gerekçesi farklılaşabilir. Kontrol olarak doğrudan `customer: &mut Customer` parametresinin aynı yeniden ödünç almayla kullanılması serbest kalmalıdır. Factory/akıcılık/değer/konteyner istisnası kullanılmasın.

Bu henüz çalışan Rust analiz testi değildir. Kaynak eşleme arayüzü derlenebilir olunca, taşıma/ödüncü yanlışlıkla oluşturma/literal yapan eşleme nedeniyle kırmızı görülmeli; sonra en az eşleme ile yeşile geçilmelidir. Yok parser/crate veya derleme hatası kırmızı kanıt değildir. Elle `Method::assign` kurup geçirmek, Rust kaynağının doğru eşlendiğini göstermez.

Açık kararlar: oluşturulan nesnenin alanı ve alias kanıtının mevcut alan politikasına etkisi; sıradan çağrı argümanının köken ikamesi; bilinmeyen/çok olasılıklı kökenin raporu; clone sonucunun hangi koşulda yeni LoD nesnesi sayılacağı. Java getter/factory/akıcılık/konteyner ve proje Value kararları kapanmaz. Kontrol akışı birleşimleri, closure, iterator, async ve makro satırları sonraki işlerdir.

## Doğrulama

macOS ARM64, Rust 1.99.1, rustfmt 1.10.0-stable, clippy 0.1.99. 2015/2018/2021/2024 edition'larının her birinde **10 test geçti**, toplam 40 koşu, atlanan yok. Dört `.rs` dosyasının biçimi ve bir olumlu test hedefinin Clippy denetimi dört edition'da geçti; lint izni eklenmedi. İlk Clippy koşusunun gölgeleme deneyine verdiği `let_and_return` önerisi, iç bağlama ve gözlemi koruyan tuple dönüşüyle giderildi; ardından dört edition yeniden doğrulandı.

Repo kökünden olumlu komutlar:

```sh
mkdir -p target/rust-origins
for edition in 2015 2018 2021 2024; do
    rustc --edition="$edition" --test -D warnings examples/rust-origins/bindings.rs -o "target/rust-origins/bindings-$edition" || exit 1
    "target/rust-origins/bindings-$edition" --quiet || exit 1
    rustfmt --edition "$edition" --check examples/rust-origins/*.rs examples/rust-origins/denied/*.rs || exit 1
    clippy-driver --edition="$edition" --test -D warnings examples/rust-origins/bindings.rs -o "target/rust-origins/clippy-bindings-$edition" || exit 1
done
```

| Olumsuz kaynak | Dört edition'da doğrulanan amaçlanan tanı |
|---|---|
| [`use_after_move.rs`](../examples/rust-origins/denied/use_after_move.rs) | `E0382`: taşınmış `wallet` değerini okuma. |
| [`partially_moved_aggregate.rs`](../examples/rust-origins/denied/partially_moved_aggregate.rs) | `E0382`: alanı taşındıktan sonra bütün `order` değerini kullanma. |
| [`live_parent_reborrow.rs`](../examples/rust-origins/denied/live_parent_reborrow.rs) | `E0506`: çocuk yeniden ödüncü sonradan kullanılacakken `parent.balance` alanına atama. |

**3 ret/edition**, toplam 12; JSON hata kodu kümesi yalnız beklenen koddu. Bunlar dil öğrenme kanıtıdır, ürünün eksik davranışı için kırmızı TDD değildir. `edition` dört kapsamdan biri, `sample` tablodaki uzantısız ad olacak şekilde:

```sh
rustc --edition="$edition" -D warnings --error-format=json "examples/rust-origins/denied/$sample.rs" -o "target/rust-origins/denied-$sample-$edition" 2> "target/rust-origins/denied-$sample-$edition.jsonl"
```

Komut başarısız olmalı; amaçlanan hata kodu/konumu yerine dosya veya kurulum hatası ret sayılmaz.

İlk koşuda geçen on test için 2024 edition'ında **10 ayrı geçici bozulma** `-D warnings` ile derlendi ve hedef test assertion ile kırıldı. Paylaşılan ödünç, kısmi taşıma veya yeniden ödünç bağımsız değerle değiştirildi; yanlış referansa yazıldı; gölgelemede dış hedef seçildi. Struct update/clone/Copy alt referansı ayrıldı; iletilen girdi ve ikinci lifetime girdisi başka referansla değiştirildi. Beklentiler değiştirilmedi. Geçici kaynak/kayıtlar `target/rust-origins/faults` altında; her `sample` ve ilgili `test_name` için kullanılan komutlar:

```sh
rustc --edition=2024 --test -D warnings "target/rust-origins/faults/$sample.rs" -o "target/rust-origins/faults/$sample"
"target/rust-origins/faults/$sample" --exact "$test_name"
```

Bu bozulmalar örneğin yanlış hedefi veya bağımsızlaştırılmış alt veriyi yakaladığını gösterir; Rust derleyicisinin taşıma/ödünç semantiği değiştirildiği veya analiz çözümleyicisi yazıldığı iddia edilmez. Önceki araştırmalar ve geniş çekirdek/workspace/Maven koşuları yeniden çalıştırılmadı. Bütün çıktılar bu worktree'nin `target/rust-origins` dizinindedir.

Test okunabilirliği ayrı, salt okunur bir geçişte incelendi; bulgu yok. On testin adları/iddiaları uyumlu, kur/işlet/denetle bölümleri boş satırla ayrılmış ve beklentiler bağımsız. Sonraki yazmanın gizleyebileceği ara durumlar önce kaydediliyor. Olumsuz kaynakların derleme reddi, çalışan test başarısı diye sunulmuyor. İnceleme kaynak değiştirmedi.
