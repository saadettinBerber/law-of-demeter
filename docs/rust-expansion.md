# Rust yapılandırması, makro açılımı ve üretilen kaynak

**Öneri:** Analiz bir seçili derleme bağlamına ait etkin ve açılmış kodu kullansın. Etkin olmayan öğe, çözülemeyen etkin öğeden ayrılsın. Her açılmış çağrı bir kez değerlendirilsin; çağrı kimliği ile kullanıcıya gösterilen kaynak konumu aynı şey sayılmasın. Açılım/girdi eksikse sessiz uygunluk sonucu verilmesin. Bu çalışma makro açıcı veya Rust adaptörü uygulamaz; altyapı seçmez.

İncelenen sürüm `cce2aa4`, tarih 23 Kasım 2026. [Karar listesindeki](../decision.md) cfg/makro/build araştırmasıdır. Önceki [kapsam](rust-support.md), [sınıflandırma](rust-classification.md), [görünürlük](rust-visibility.md), [değişebilirlik](rust-mutability.md), [metot seçimi](rust-method-resolution.md), [köken](rust-origins.md) ve [kapsayıcı](rust-containers.md) sınırları korunur. Üretim, karar listesi ve kök Cargo dosyaları değişmedi.

## Mevcut sınır

[`SourceReader`](../crates/core/src/application/ports.rs), dosya yolu, sınıflar ve `Method` nesneleri veya yol/satır/mesaj taşıyan `ReadError` verir. Seçili Cargo birimi, açılım durumu veya üretilmiş dosya bağı için özel sözleşmesi yoktur. [`Call`](../crates/core/src/domain/method/call.rs) çağrı adı/satırı/alıcı/akış taşır; açılım kimliği taşımaz. [`CheckProject`](../crates/core/src/application/check_project.rs) her verdict'i dosyanın yoluyla [`Finding`](../crates/core/src/application/report.rs) yapar; burada dosya/satır/ad üzerinden dedup yoktur. Aynı satırda iki gerçek çağrının korunması mümkündür, fakat farklı kaynak kökenlerini bu bilgiler tek başına açıklamaz.

Java [okuyucusu](../crates/java-source/src/reader.rs) kaynak metninden sözdizimi ağacı okur; [eşleme](../crates/java-source/src/mapping/expressions.rs) çözülmüş alıcı/dönüş/akışla domain çağrısı kurar. Rust'ta bu sınıra gelmeden yapılandırma ve açılım bağlamı gerekir. Domain'e cfg, Cargo veya Rust Span yerleştirilmemeli; açılmış ifadenin nesne ilişkisi mevcut köprülerle aktarılmalıdır.

[`rust-semantics.rs`](../examples/rust-semantics.rs) içindeki `an_inactive_cfg_item_does_not_need_to_resolve_its_body` ve `a_macro_can_generate_a_method_call` temel kanıt olarak korundu; kopyalanmadı veya yeniden çalıştırılmadı. [Dizi metot seçimi araştırması](rust-method-resolution.md) da burada yalnız karışık-edition karşı örneğiyle tamamlandı.

## Küçük yerel fixture

[`examples/rust-expansion`](../examples/rust-expansion/Cargo.toml) kendi workspace/lockfile'ına sahiptir; dış bağımlılık yoktur:

| Paket | Edition ve işi |
|---|---|
| `expansion-legacy` | 2018; dışa aktarılan `macro_rules!` tanımları. |
| `expansion-generator` | 2021; gerçek `proc-macro` crate'i. Yalnız **function-like** `generated_charge!` çalıştırıldı; özel attribute/derive proc-macro denenmedi. |
| `expansion-consumer` | 2024; makroları kullanan ayrı crate, `build.rs`, `OUT_DIR` include'u ve seçilen test/örnek hedefleri. |

Taban Rust/Cargo 1.99.1, macOS ARM64; host ve açık hedef `aarch64-apple-darwin`. Bu karışık workspace dört edition matrisi değildir; başka platform/özellik birleşimleri doğrulanmadı. `cfg(test)` test hedefiyle sınırlıdır. `cfg_attr` deneyindeki Clone, standart derive'dır; özel derive proc-macro kanıtı değildir.

## Doğrulanmış ayrımlar

[Koşullu derleme](https://doc.rust-lang.org/reference/conditional-compilation.html) etkin olmayan öğeyi kaldırır; `cfg!` ise boolean ifade üretir. `cfg_attr` yalnız koşul sağlanınca attribute ekler. Etkin olmayan gövdede ad çözümü gerekmemesi, dengesiz token veya her türlü bozuk sözdiziminin kabul edildiği anlamına gelmez.

| Ayrım | Fixture gözlemi |
|---|---|
| `#[cfg]` / `cfg!` | `selected_value` için alternate kapalıyken 17, açıkken 23 veren farklı öğe seçilir. Buna karşılık alternate kapalıyken bile `negative/cfg_boolean.rs` içindeki false `cfg!` dalının `missing_function` adı `E0425` verir. False boolean dalı kaynak öğesini kaldırmaz. |
| `cfg_attr` | `negative/cfg_attr.rs`, `alternate,attr-probe` ile Clone üretip `(7,7)` doğrular; yalnız `attr-probe` ile `.clone()` `E0599` olur. Seçilen attribute tip şekline katılır. |
| Hedef cfg / build cfg | Testte seçilen macOS/aarch64 gövdesi ve 64 bit beklentisi doğrulanır. Build script `build_bonus` için check-cfg bildirir, bonus modunda cfg üretir. Base/bonus testleri ayrıca istenen ortam modunu sabit `base`/`bonus` beklentisiyle karşılaştırır; yanlış cfg seçimi kendi yanlış beklentisini üretmez. |
| Makrodan gelen alıcı ve çoklu çağrı | Tek `charge_twice!(&order)` çağrısı iki ayrı `order.wallet()` alıcısı ve iki `charge` çağrısı üretir. Ara bakiyeler `(43,32)`, alıcı okuma ve ödeme sayıları 2'dir. Raw çağrı yerinde bu metot zincirleri yazılı değildir. Tek çağrıya dedup etmek bir etkiyi kaybettirir; tanım şablonunu ayrıca çalışmış kod gibi saymak da yanlıştır. |
| Yerel provider / consumer proc-macro | `generated_charge!(wallet)` derleme sırasında sağlayıcı fonksiyonunu çalıştırarak `charge(13)` ve `balance()` token'ları üretir. Test çalışırken bu üretilmiş çağrılar bakiyeyi 37, ödeme sayısını 1 yapar. Makronun derleme zamanı çalışmasıyla ürettiği çağrının çalışma zamanı etkisi ayrıdır. |
| Build üretimi / include | `build-input.txt` değeri 7'dir. Base kaynak `charge(7)`, bonus kaynak `charge(11)` üretir; `generated_balance` sonuçları 43/39'dur. Kod `include!(concat!(env!("OUT_DIR"), "/generated.rs"))` ile tüketicinin bağlamına girer. Dış hedef dizininde bulunması onu bağımsız kütüphane kaynağı yapmaz. |
| Tanım token'ı / çağıranın token'ı | 2018 sağlayıcısındaki `definition_items!` metodun `into_iter` token'ını tanımdan verir: 2024 tüketicide sonuç `slice::Iter<'_,String>`, öğeler ilk dizinin aynı referanslarıdır. `callsite_items!(consumed, into_iter)` metot token'ını tüketiciden alır: sonuç `array::IntoIter<String,2>`, sahip olunan değerler `first,second` olur. İki tip beklentisi gerçekten derlenir; yalnız paket edition'ı yeterli değildir. |

[macro_rules](https://doc.rust-lang.org/reference/macros-by-example.html) token şablonu açar. [Prosedürel makrolar](https://doc.rust-lang.org/reference/procedural-macros.html) başka crate'te kullanılabilen derleme zamanı fonksiyonlarıdır; ürettikleri token'lara Span atanabilir. Bu fixture yöntem token'larında `Span::call_site()` kullanır. [Metot adı token'ının edition kuralı](https://doc.rust-lang.org/reference/expressions/method-call-expr.html#r-expr.method.edition2021), karışık-edition sonucunu açıklar. Bu deney bütün hijyen, nested macro veya edition davranışını kapsamaz.

## Hangi aşama kod çalıştırdı?

[Cargo build script yaşam döngüsü](https://doc.rust-lang.org/cargo/reference/build-scripts.html), script'in paket derlenmeden önce derlenip çalıştırılmasını tanımlar. [cargo check](https://doc.rust-lang.org/cargo/commands/cargo-check.html) ana hedefte son kod üretimi/link adımını atlayabilir; bu, build script veya kullanılan proc-macro'nun çalışmadığı anlamına gelmez. Kaynak okumakla makro açmak aynı işlem değildir; dosya taraması seçili bağlamı sağlayamaz.

| Gerçek koşu | Gözlenen etki |
|---|---|
| `cargo metadata --no-deps` | Üç yerel paket, edition ve hedefler okundu; özel metadata hedef dizini oluşmadı, build marker/derleme çıktısı yoktu. Bu fixture'da build/proc çalıştırma kanıtı değildir. |
| Taze base test koşusu | Build script kendi `OUT_DIR/build-runs.log` dosyasına `base` yazdı, `generated.rs` üretti; altı runtime test geçti. |
| Aynı base komutunun `-vv` tekrarı | Altı test yine çalıştı. Cargo consumer için `Fresh` yazdı; script'in kayıt dosyası byte olarak değişmedi. Cargo olayının/yazılmış çıktının görünmesi tek başına yeni script çalışması sayılmadı. |
| Aynı feature setinde base → bonus | Aynı OUT_DIR kaydına `bonus` eklendi; üretilen kod 11 kullanmaya başladı; altı test geçti. `rerun-if-env-changed` girdisi değişti. |
| Bonus + alternate | Ayrı feature OUT_DIR'ında `bonus` kaydı oluştu, feature gövdesi değişti; altı test geçti. |
| Taze `cargo check --tests` | Build kaydı oluştu; proc-macro kullanan test gövdesi tip denetiminden geçti. Runtime test çalışmadı. Yalnız sağlayıcı bağımlılığının varlığı her makro fonksiyonunun çalıştığı anlamına gelmez; etkin invocation gerekir. |

Script yalnız kendi `build-input.txt` ve `invalid-generated.rs` girdilerini okur; çıktı ve marker'ı yalnız OUT_DIR'a yazar. İki dosya için `rerun-if-changed`, `LOD_FIXTURE_MODE` ve `LOD_FIXTURE_BROKEN` için `rerun-if-env-changed` bildirir. Dosya değişimiyle cache yenileme ayrıca denenmedi; geçici bozuk kopyadaki 8 girdisi yalnız taze derlemede sınandı. OUT_DIR kalıcı olabilir; marker yalnız bu kontrollü script'in gerçekten çalıştığını ayırt eder. Yeniden üretimde ilk tazelik gözlemi için daha önce kullanılmamış özel hedef dizini seçilmelidir.

Host/target burada aynıdır. Cross derlemede build script içindeki `cfg!` host'u anlatır; hedef bilgisi için Cargo'nun `CARGO_CFG_*` girdileri gerekir. Bu, belgelenmiş sınırdır; başka hedefte çalıştırılmış deney değildir. Proje build/proc kodunun çalıştırılacağı aşama ileride seçilecek analiz altyapısıyla açıkça belirtilmelidir. Bu çalışmada yalnız yazdığımız yerel fixture kodu çalıştırıldı; üçüncü taraf build/proc kodu yoktur.

## Kaynak konumu kanıtı

Beş amaçlı compiler reddi alındı; kod kümeleri yalnız aşağıdaki beklenen hatalardı. Bunlar ürünün eksik davranışı için kırmızı TDD değildir.

| Kaynak / yapılandırma | Rust 1.99.1 JSON tanısı |
|---|---|
| `negative/cfg_boolean.rs`, `cfg-probe` | `E0425`, satır 3: false dalındaki `missing_function`. |
| `negative/cfg_attr.rs`, `attr-probe` | `E0599`, satır 7: Clone üretilmediği için `clone`. |
| `negative/proc_location.rs`, `proc-probe` | `E0599`, primary ve expansion invocation satır 4; `def_site_span`, `generator/src/lib.rs:4`. Üretilen `charge` token'ının call-site Span'i kaynak dosyadaki literal bir `.charge` çağrısı demek değildir. |
| `LOD_FIXTURE_BROKEN=1`, consumer lib | `E0599`, primary `OUT_DIR/generated.rs:3`; bu kaynak `invalid-generated.rs` girdisinden kopyalandı. Burada primary span'da expansion zinciri yoktu. Include/üretici bağı yalnız bu span'dan çıkarılamaz. |
| `denied/declarative_location.rs` | `E0599`, primary satır 3'teki makro gövdesi; expansion invocation satır 10, definition satır 1. |

Tanım, invocation ve üretilmiş dosya ayrı kökenlerdir. Bu gözlem yalnız rustc tanı formatına aittir; gelecekte seçilecek analiz kütüphanesinin kaynak eşleme API'si doğrulanmış değildir. [include!](https://doc.rust-lang.org/std/macro.include.html) dosyayı çevreleyen bağlama yerleştirir; dosya yolu tek başına bağlam/üretici kimliği değildir.

## En az bağlam ve önerilen raporlama

| Gereken olgu | En küçük kullanım ve sınır |
|---|---|
| Seçili birim | Paket/bağımlılık kimliği, hedef türü, feature/cfg kümesi, target ve edition adaptör/application girdisi olsun. Token'ın edition bağlamı paket edition'ına indirgenmesin. Bütün feature matrisi tasarlanmadı. |
| Etkinlik / tamamlanma | Etkin olmadığı bilinen, etkin ve çözülmüş, etkin fakat açılımı/üretimi eksik ve yapılandırma yokluğundan etkinliği bilinmeyen durumlar ayrı kalsın. Eksik bağlam görünür okuma/analiz sorunu olsun; ham token'lar başarılı domain modeli yerine konmasın. Kesin çıkış kodu ve kısmi rapor politikası sonraki karar satırıdır. |
| Açılmış çağrı ve tanı kökeni | Her semantik çağrının seçili birim içinde ayırt edilebilir kimliği; gösterilecek invocation konumu ve mevcutsa tanım/üretilen dosya bağı application tarafında korunsun. Aynı `(yol,satır,ad)` iki gerçek çağrı olabilir. Yalnız aynı açılmış çağrı iki kez eşlenmişse tekrar önlensin. |
| Nesne bilgisi | Açılmış gerçek alıcı, seçilmiş hedef ve dönüş/akış bilgisi mevcut domain köprülerine verilsin. Makro adı veya metin içindeki noktalar LoD ilişkisinin yerine geçmesin. cfg/Span/Cargo kuralları domain'e girmesin. |

İlk dar test için yeni domain şeması zorunlu değildir: iki açılmış çağrı birer kez eşlenebilir. Daha zengin tanıya ihtiyaç olduğunda isteğe bağlı dil bağımsız çağrı kimliği/kaynak bağı yalnız eklemeyle taşınabilir; bir evrensel source-map grafiği tasarlanmadı. Çözümleyici desteği, generated dosyanın proje aidiyeti ve açılım eksikliği rapordan saklanmamalıdır. Dizin altındaki bütün `target` dosyaları taranmasın; yalnız seçili birimin gerçek girdileri alınsın.

## Sonraki dar gerçek-adaptör kabul testi

Öneri: **`an_active_macro_keeps_two_distinct_expanded_calls`**. Gerçek Cargo bağlamında `audit` özelliğiyle etkinleşen fonksiyondaki tek invocation, iki `order.customer().activate()` zinciri üretsin. `order` parametredir; Customer davranışlı, Value/konteyner/factory/akıcılık istisnasına girmeyen proje nesnesidir.

Beklenti `audit` açıkken iki ayrı `activate` ihlali, kapalıyken kaldırılmış fonksiyon için sıfırdır. Açılmamış etkin invocation'ı atlamak açık koşuda 0, aynı invocation konumuyla dedup etmek 1, ham şablonla açılmış çağrıları birlikte saymak fazladan sonuç üretir. Böylece eksik açılım/yanlış bağlam davranışı ayırt edilir. Her açılmış çağrı bir kez eşlenmeli; ortak invocation konumu iki çağrıyı birleştirmemelidir.

Bu Rust analiz testi henüz çalıştırılmadı. Gelecekte derlenebilir adaptörde bu yanlış davranışlardan biri nedeniyle kırmızı görülmeli; eksik parser/crate veya compiler hatası kırmızı kanıt değildir. Elle Call kurup iki sonuç saymak gerçek açılımı doğrulamaz. Test bütün proc-macro, build kökeni veya kaynak eşleme desteğini tamamlanmış saymaz. Container/Value, oluşturulan nesne alanı ve mevcut Java politika soruları açık kalır.

Açık kararlar: seçilecek altyapının açılım/kaynak API'si; iç içe açılımlarda gösterilecek ana konum; eksik bağlamda kısmi rapor/çıkış politikası; analiz cache'inin hangi girdilerle doğrulanacağı. Özel attribute/derive proc-macro, başka hedefler ve bütün feature birleşimleri için destek kanıtı yoktur.

## Doğrulama ve yeniden üretme

Üç yapılandırmada altışar test, ayrıca base cache tekrarında altı test geçti: **24 runtime test koşusu**, sekiz farklı cfg-seçimli test gözlemi. `cfg_attr` örnek programı bir kez başarıyla çalıştı. Taze `check --tests` geçti; beş amaçlı ret doğrulandı. Geniş workspace/Maven veya önceki Rust örnekleri çalıştırılmadı. Aşağıdaki workspace yalnız bu fixture'dır.

Repo kökünden kullanılan temel komutlar; `LOD_FIXTURE_BROKEN` normal koşularda ayarlı değildi:

```sh
fixture_manifest=examples/rust-expansion/Cargo.toml
fixture_target="$PWD/target/rust-expansion/fixture"
CARGO_TARGET_DIR="$fixture_target" cargo generate-lockfile --offline --manifest-path "$fixture_manifest"
CARGO_TARGET_DIR="$fixture_target" RUSTFLAGS='-D warnings' LOD_FIXTURE_MODE=base cargo test --offline --locked --manifest-path "$fixture_manifest" -p expansion-consumer --lib --target aarch64-apple-darwin
CARGO_TARGET_DIR="$fixture_target" RUSTFLAGS='-D warnings' LOD_FIXTURE_MODE=base cargo test --offline --locked --manifest-path "$fixture_manifest" -p expansion-consumer --lib --target aarch64-apple-darwin -vv
CARGO_TARGET_DIR="$fixture_target" RUSTFLAGS='-D warnings' LOD_FIXTURE_MODE=bonus cargo test --offline --locked --manifest-path "$fixture_manifest" -p expansion-consumer --lib --target aarch64-apple-darwin
CARGO_TARGET_DIR="$fixture_target" RUSTFLAGS='-D warnings' LOD_FIXTURE_MODE=bonus cargo test --offline --locked --manifest-path "$fixture_manifest" -p expansion-consumer --lib --target aarch64-apple-darwin --features alternate
CARGO_TARGET_DIR="$PWD/target/rust-expansion/attr" RUSTFLAGS='-D warnings' LOD_FIXTURE_MODE=bonus cargo run --offline --locked --manifest-path "$fixture_manifest" -p expansion-consumer --example cfg_attr --features alternate,attr-probe --target aarch64-apple-darwin
CARGO_TARGET_DIR="$PWD/target/rust-expansion/metadata" RUSTFLAGS='-D warnings' LOD_FIXTURE_MODE=base cargo metadata --offline --locked --no-deps --format-version 1 --manifest-path "$fixture_manifest"
CARGO_TARGET_DIR="$PWD/target/rust-expansion/check" RUSTFLAGS='-D warnings' LOD_FIXTURE_MODE=base cargo check --offline --locked --manifest-path "$fixture_manifest" -p expansion-consumer --tests --target aarch64-apple-darwin
rustc --print cfg --target aarch64-apple-darwin
```

Cache gözleminde `fixture/**/build-runs.log` dosyaları karşılaştırıldı: ilk base kaydı `base\n`, tekrar sonrası aynı byte'lar, bonus sonrası aynı dosyada `base\nbonus\n`; alternate için ayrı dosyada `bonus\n`. Ayrı lint/check/negative/fault hedefleri bu dosyaları değiştirmedi. Kayıtlar `target/rust-expansion/configuration-results.txt` ve ilgili log'lardadır. Build-script-executed gibi tekrar gösterilebilen olaylar çalıştırma sayacı olarak kullanılmadı.

Olumsuz komutların her biri başarısız olmalıdır; `sample/features/code` tablosu sırasıyla `cfg_boolean/cfg-probe/E0425`, `cfg_attr/attr-probe/E0599`, `proc_location/proc-probe/E0599` şeklindedir. `case_name` sırasıyla `cfg-boolean`, `cfg-attr`, `proc-location` idi:

```sh
CARGO_TARGET_DIR="$PWD/target/rust-expansion/negative-$case_name" RUSTFLAGS='-D warnings' LOD_FIXTURE_MODE=base cargo check --offline --locked --manifest-path "$fixture_manifest" -p expansion-consumer --target aarch64-apple-darwin --message-format=json --example "$sample" --features "$features"
CARGO_TARGET_DIR="$PWD/target/rust-expansion/negative-generated-location" RUSTFLAGS='-D warnings' LOD_FIXTURE_MODE=base LOD_FIXTURE_BROKEN=1 cargo check --offline --locked --manifest-path "$fixture_manifest" -p expansion-consumer --target aarch64-apple-darwin --message-format=json --lib
rustc --edition=2024 -D warnings --error-format=json examples/rust-expansion/denied/declarative_location.rs -o target/rust-expansion/declarative-denied
```

Cargo JSON compiler-message içindeki hata kodları ve primary/expansion span'leri denetlendi. Ayrıntılar `target/rust-expansion/diagnostic-observations.json` içindedir; mevcut target çıktısı taze derleme yerine gösterilmedi.

Rustfmt 1.10.0-stable ve clippy 0.1.99 ile biçim/lint geçti. Cargo fmt üyelerin kendi edition'larını kullandı; hedefe bağlı olmayan iki olumsuz kaynak ayrıca biçimlendi. Clippy, normal/base ve alternate/bonus için üç paketin derlenebilir tüm hedeflerinde; cfg_attr için yalnız olumlu özellik birleşiminde çalıştı. Tek yerel lint izni karışık-edition testindeki `array_into_iter` ve `clippy::into_iter_on_ref` içindir: eski yazımı `.iter()` ile değiştirmek araştırılan token-edition ayrımını kaldırır. Genel uyarılar izinli değildir.

```sh
CARGO_TARGET_DIR="$PWD/target/rust-expansion/format" cargo fmt --manifest-path "$fixture_manifest" --all --check
rustfmt --edition 2024 --check examples/rust-expansion/consumer/invalid-generated.rs examples/rust-expansion/denied/declarative_location.rs
CARGO_TARGET_DIR="$PWD/target/rust-expansion/lint" RUSTFLAGS='-D warnings' LOD_FIXTURE_MODE=base cargo clippy --offline --locked --manifest-path "$fixture_manifest" --workspace --all-targets --target aarch64-apple-darwin -- -D warnings
CARGO_TARGET_DIR="$PWD/target/rust-expansion/lint" RUSTFLAGS='-D warnings' LOD_FIXTURE_MODE=bonus cargo clippy --offline --locked --manifest-path "$fixture_manifest" --workspace --all-targets --features alternate --target aarch64-apple-darwin -- -D warnings
CARGO_TARGET_DIR="$PWD/target/rust-expansion/attr" RUSTFLAGS='-D warnings' LOD_FIXTURE_MODE=bonus cargo clippy --offline --locked --manifest-path "$fixture_manifest" -p expansion-consumer --example cfg_attr --features alternate,attr-probe --target aarch64-apple-darwin -- -D warnings
```

İlk yeşil gözlemler için ayrı geçici workspace kopyalarında **dokuz bozulma** derlendi ve runtime assertion ile kırıldı: build girdisi base/bonus için 8 yapıldı; iki feature gövdesinin sabitleri bozuldu; hedef cfg koşulları yer değiştirdi; ikinci makro ödemesi kaldırıldı; proc-macro tutarı 12 yapıldı; eski makro ilk öğeyi atladı; Clone örneği 8 döndüren yanlış uygulamayla değiştirildi. Beklentiler değişmedi. Son makro bozulması öğe koruma assertion'ını sınar; compiler edition kuralını değiştirdiğimiz iddia edilmez. Compiler tip uyuşmazlığı kırmızı ürün testi sayılmadı.

Kopyalar `target/rust-expansion/faults/<case>/input`, çıktılar aynı case'in `output` dizinindedir. Orijinal girdiler değişmedi. Her case'in mode/features seçimiyle şu komut, `tests::<test_name>` için yalnız bir testi çalıştırdı; Clone case'inde `cargo run ... --example cfg_attr --features alternate,attr-probe` kullanıldı:

```sh
CARGO_TARGET_DIR="$PWD/target/rust-expansion/faults/$case_name/output" RUSTFLAGS='-D warnings' LOD_FIXTURE_MODE="$mode" cargo test --offline --locked --manifest-path "target/rust-expansion/faults/$case_name/input/Cargo.toml" -p expansion-consumer --target aarch64-apple-darwin --lib --features "$features" -- --exact "tests::$test_name"
```

Boş feature seçiminde `--features` verilmedi. Bu kontroller fixture'ın kaybolan/yanlış üretilen etkiyi yakaladığını gösterir; uygulanmış analiz davranışının TDD kanıtı değildir.

Test okunabilirliği ayrı, salt okunur bir geçişte incelendi; bulgu yok. Sekiz cfg-seçimli test ve cfg_attr programında adlar gözlenen davranışla uyumlu, kur/işlet/denetle blokları boş satırla ayrılmış, beklentiler bağımsızdır. İlk makro etkisi sonraki yazmadan önce kaydedilir; kaynak/tip kimliği ve sayaçlar yanlış sonucu ayırır. Etkinleştirilmeyen testler çalışmış sayılmadı. İnceleme kaynak değiştirmedi.
