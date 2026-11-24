# Rust backend probe: kalıcı kaynak paketi

Bu belge probe'un güncel kullanım girişidir. `ra_ap_* =0.0.358` ailesi kullanılır; yalnız `ra_ap_hir_def` ve `ra_ap_hir_ty`, onaylı metot token edition düzeltmesiyle [third_party/rust-analyzer](../third_party/rust-analyzer/) altında tutulur. Normal çalıştırmada kaynak kopyalama, elle yama uygulama veya Cargo override ekleme gerekmez.

Taban `fbead18975444a40d23a3158b814f8fc6db23598`, doğrulama 24 Kasım 2026. Silinen eski çalışma ağacının exact snapshot'ı bulunamadı; paket Git'teki onaylı girdilerden yeniden üretildi.

Bu paketleme, önceki belgelerdeki izole deneyi kalıcı probe kullanımına bağlar. Eski belgeler kendi taban commit'lerini anlatır. Ürün Rust adaptörü, CLI hata politikası, genel traversal/tekrar önleme ve bütün Rust kapsamı bu değişikliğin sonucu değildir.

## Sabit kaynaklar

Dokuz doğrudan RA pini exact `=0.0.358`'dir. Lock toplam 179 paket ve aynı sürümde 27 RA paketi içerir. Salsa `0.28.5`, unicode-properties `0.1.4`, unicode-ident `1.0.24` ve diğer sürüm/dependency seçimleri korunur. [Probe manifestindeki](../examples/rust-backend-probe/Cargo.toml) iki repo-relative `[patch.crates-io]` yolu kullanılır; [lock](../examples/rust-backend-probe/Cargo.lock) farkı yalnız bu iki registry source/checksum çiftinin kaldırılmasıdır.

| Paket arşivi | SHA-256 |
|---|---|
| [ra_ap_hir_def 0.0.358](https://crates.io/api/v1/crates/ra_ap_hir_def/0.0.358/download) | `7ef811f35ed1c394d27a7e0d79869aa4891108496736597b87bbc632a4eee1ec` |
| [ra_ap_hir_ty 0.0.358](https://crates.io/api/v1/crates/ra_ap_hir_ty/0.0.358/download) | `725f913d562b8d794035bc3bcc3e62adf09b843e11b443750bf8a7027ec1d3bf` |

Arşivlerin VCS kaydı `2ad81e72b7e978cb800dffe1bcce5c9f98032acf`, `dirty=true` içerir. Yeniden üretim tabanı checksum'lı yayımlanmış arşivdir; bütün arşivin upstream commit'e eşit olduğu varsayılmaz.

[Onaylı patch](../examples/rust-backend-probe/experiments/edition-patch/method-token-edition.patch) SHA-256: `b6f555c62e922cc341e961ba64d410025cf3f3c23e949f6be4f03f1100b71a2e`. Arşivlerdeki 211 dosyanın tamamı korunur. Yalnız [upstream.json](../examples/rust-backend-probe/experiments/edition-patch/upstream.json) içindeki yedi kaynak dosyası değişmiştir; diğer 204 dosya arşivle byte eşittir. Yedi sonuç hash'i onaylı after_sha256 değerleridir.

[SOURCES.json](../third_party/rust-analyzer/SOURCES.json), yerel değişikliği, yedi dosyanın önce/sonra kimliklerini ve lisans kaynak/hash'lerini kaydeder. Bunlar pristine upstream kaynak değildir. [ARCHIVE-SHA256SUMS](../third_party/rust-analyzer/ARCHIVE-SHA256SUMS) indirilen arşiv kimlikleridir; arşiv binary'leri repo'ya alınmaz. Paket arşivlerinde bulunmayan [MIT](../third_party/rust-analyzer/LICENSE-MIT) ve [Apache](../third_party/rust-analyzer/LICENSE-APACHE) metinleri aynı resmî commit'ten değişmeden alınmıştır.

Yama metot adı token'ının edition bilgisini yalnız ilgili lookup context'ine taşır; genel inference edition'ını değiştirmez. [Önceki kaynak incelemesi](rust-macro-edition-patch.md) yedi dosyanın sorumluluklarını ve kapsam sınırını anlatır. Registry cache değiştirilmez; diğer 25 RA paketi normal registry kaynaklarıdır.

## Toolchain ve çalıştırma önkoşulları

Bu koşu Rust/Cargo **1.99.1**, rustc `58cf8f9c35b355199727627d07ae34a4e2846d67`, host `aarch64-apple-darwin` ile doğrulandı. RA paketlerinin MSRV'si 1.98 olması başka host/toolchain/proc ABI uyumunu kanıtlamaz. Bu teslim otomatik toolchain kurmaz veya global Cargo ayarı değiştirmez.

PATH'teki `rustc` ve `cargo` aynı seçili toolchain'e ait olmalıdır. `rustc --print sysroot` altındaki `lib/rustlib/src/rust/library` eşleşen rust-src kaynaklarını, `libexec/rust-analyzer-proc-macro-srv` ise kullanılabilir eşleşen proc server'ı içermelidir. Loader bu yolları açıkça kurar; yalnız server dosyasının varlığı başarılı açılım kanıtı sayılmaz. Testler gerçek proc açılımını ve retained client'ı denetler. Desteklenmeyen kurulum sessizce test atlamaz; hata verir.

Önce dependency cache'i `cargo fetch --locked` ile doldurulur. Sonraki offline komutlar bu cache'i ve kurulu std/proc önkoşullarını kullanır. Cargo target yolu mutlak olmalıdır; aşağıdaki `$PWD` bunu sağlar. Fixture build.rs ve yerel proc provider kodu çalışır; probe başka projeyi yüklemez.

Repo kökünden:

```sh
out="$PWD/target/backend-recovery/run"
manifest="$PWD/examples/rust-backend-probe/Cargo.toml"
mkdir -p "$out"
probe_cargo() {
  env CARGO_TARGET_DIR="$out" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 RUSTFLAGS='-D warnings' cargo "$@"
}
probe_cargo fetch --locked --manifest-path "$manifest"
probe_cargo test --locked --offline --manifest-path "$manifest" --tests -- --test-threads=1
probe_cargo test --locked --offline --manifest-path "$manifest" --all-targets -- --test-threads=1
probe_cargo run --locked --offline --manifest-path "$manifest" --example macro_edition_gate
probe_cargo check --locked --offline --manifest-path "$manifest" --all-targets
probe_cargo clippy --locked --offline --manifest-path "$manifest" --all-targets -- -D warnings
cargo fmt --manifest-path "$manifest" --all --check
```

Kök workspace dört ürün crate'ini, probe workspace yalnız kendisini içerir. Vendor paketleri workspace üyesi değildir; kök Cargo.toml/lock veya exclusion gerekmez. Normal komutlar `target/source-patched`, symlink, mutlak kullanıcı kaynak yolu, registry düzenlemesi veya global Cargo config gerektirmez.

## Normal öğrenme testleri

Toplam 21 test normal Cargo discovery içindedir: semantics 5, traits 3, expansion 7, macro_edition 3, unresolved_cfg 3. Önceki 18 davranış korunur; borrowed edition kabulü ve iki sıra kontrolü artık explicit example dışında integration testinden de koşar.

[Ortak macro implementation](../examples/rust-backend-probe/support/macro_edition.rs) küçük example ve üç integration testince kullanılır. Önceki bütün assertion/helper gövdeleri ve bağımsız borrowed/owned referans beklentileri korunur. [order_queries.rs](../examples/rust-backend-probe/experiments/edition-patch/order_queries.rs) değiştirilmeden paylaşılır; [iki compiler sıra tanığı](../examples/rust-backend-probe/experiments/edition-patch/order_cases.rs) mevcut consumer fixture'ına eklenmiştir.

Expansion testindeki legacy TypeMismatch muafiyeti kaldırılmıştır. Consumer'ın bütün modüllerinde InactiveCode dışındaki her diagnostic reddedilir. [Kaynak eşleme testleri](rust-source-mapping.md) ve [unresolved/cfg testleri](rust-unresolved-cfg.md) onaylı deney dosyalarını normal integration girişlerinden kullanır. Typed pointer, gerçek dosya/çağrı kimliği, unknown ve bağımsız kaynak beklentileri değiştirilmez. Bu testler genel diagnostic tamlığı veya ürün raporu iddiası değildir.

## Yeniden doğrulama ve bakım

Bir sürüm yükseltmesinde RA ailesi birlikte exact sürümlenir; farklı aileler karıştırılmaz. Kilidi baştan üretip Salsa/Unicode seçimlerini sessizce yenileme. Önce aynı borrowed kabulü, iki çağrı sırası ve compiler tanıklarıyla yeni kaynağı değerlendir. Düzeltmeyi gerçekten içeren tutarlı yayımlanmış aile doğrulanınca iki yerel patch kaldırılabilir; yerel kaynağa düşen testleri gevşetmek kabul sayılmaz.

Arşiv yenilenirse checksum ve VCS metadata'sını yeniden kaydet; temiz arşive onaylı patch'i sıfır fuzz ile uygula. Bütün arşiv girdilerini koru ve tüm dosyaları karşılaştır. Yedi değişiklik dışındaki farkı araştır; kaynak/patch/lisans kimliklerini ve SOURCES.json'u birlikte güncelle. Üçüncü taraf dosyalarını topluca formatlama veya kısaltma.

Portable kabul için başka konumda regular kaynak kopyası ve taze target kullan. Metadata ve compile log'larında iki vendor paketinin ve probe'un yeni kopya yollarını doğrula. Registry fallback kontrolünde yalnız iki patch kaydını kaldır ve aynı ailenin baseline registry lock'unu kullan; borrowed test aynı gerçek assertion'da kırılmalıdır. Patch bağlantısını geri getirince geçmelidir.

Strict diagnostic karşı örneğinde ayrı fixture kopyasında yalnız `Literal::i32_unsuffixed(13)` yerine `Literal::u32_suffixed(13)` kullan. Numeric değer ve beklenen hedefler aynı kalırken gerçek proc testi consumer diagnostic assertion'ında, compiler ise E0308 ile reddetmelidir. Kaynağı geri alıp taze proc/build çıktısıyla tekrar geçir. Beklenti değişikliği, ignore/should_panic, eski binary veya kurulum hatası başarı sayılmaz.

## Bu yeniden üretimdeki kanıtlar

Gerçek komutlar, çıkışlar ve log'lar çalışma ağacının `target/backend-recovery/` dizinindedir. Ana koşuların aynı adlı JSON kayıtları exact komutu ve mutlak target yolunu tutar; doğrudan format denetimleri ayrı log'lardadır. Önceki belgelerin eski yeşilleri yeni koşu sayılmamıştır.

| Kontrol / log adı | Gerçek sonuç |
|---|---|
| `baseline-red` | Yamasız borrowed integration testi gerçek sonuç/hedef/tanı assertion'ında exit 101. |
| `macro-green` | İlk paketleme parçasında eski 12 + macro 3, toplam 15/15; exit 0. |
| `strict-21` | Sıkı tanı ve normal discovery ile 21/21; exit 0. |
| `portable-fetch`, `portable-tests`, `portable-all-targets`, `portable-example` | Başka konumdaki regular kaynak kopyasında fetch --locked sonrası taze target; iki ayrı 21/21 koşu ve explicit example exit 0. Bunlar 21 farklı testin tekrarlarıdır. |
| `compiler-four` | Önceki iki ve eklenen iki sıra tanığı 4/4; exit 0. Fixture Clippy ve fmt de geçti. |
| `compiler-unresolved-clean` / `boolean` / `alternate` | Temiz default lib exit 0; bin E0425/101; açık feature lib E0425+E0599/101. |
| `fallback-red` → `fallback-restored-green` | İki override kaldırılmış registry metadata'sıyla aynı borrowed assertion exit 101; bağlantılar geri gelince aynı kaynak/beklenti 1/1, exit 0. |
| `proc-type-red` → `proc-type-restored-green` | 13u32, strict consumer diagnostic assertion'ında i32/u32 TypeMismatch ile exit 101; doğru kaynak ve taze proc çıktısıyla 1/1, exit 0. |
| `compiler-proc-type-red` → `compiler-proc-restored-green` | Aynı bozuk gerçek kaynak E0308/101; kaynak geri alınınca exit 0. |
| `portable-check`, `portable-clippy`, `portable-fmt`, `portable-metadata` | Aynı taşınabilir kopyada tüm hedefler check/Clippy ve fmt exit 0; metadata ile compile log'ları iki vendor paketi ve probe için yeni kopya yollarını doğruladı. |
| `check-strict`, `clippy-strict`, `fmt-strict`, `vendor-seven-fmt` | Probe tüm hedeflerde check/Clippy, probe formatı ve yalnız yedi onaylı üçüncü taraf kaynak için format denetimi exit 0. |

Karşı örnekler kendi regular `fault-layout` kopyasında koştu; asıl fixture değiştirilmedi. Fallback/proc koşuları yalnız bu çalışmanın dependency build çıktılarının APFS kopyasını kullanır; her kaynak bağlantısı değişiminde kendi probe paketi temizlenip yeniden derlendi. Doğru proc kaynağı geri alındığında analiz fixture target'ları silinip proc/build çıktıları yeniden üretildi. Bu önbellek kullanımı taşınabilir kabulün taze target iddiasından ayrıdır. Taşınabilir kopyanın kaynak kökü portable-layout.txt, 255 byte eş regular girdi karşılaştırması portable-source-identities.json içindedir; bu koşunun build target'ı başlangıçta boştur. Registry cache, kurulu std kaynakları ve proc server önkoşul olarak kullanılmıştır.

Ayrı bir ajan, test okunabilirliğini salt okunur inceledi; somut bulgu yok. Bu geçiş test/build çalıştırmadı ve bütün paketin nihai bağımsız incelemesi değildir. Kök workspace/Maven ve upstream RA test paketleri, değişmeyen girdiler için tekrar çalıştırılmadı.

Vendor toplamı **215 dosya / 5.027.904 byte**: 211 arşiv dosyası, SOURCES.json, ARCHIVE-SHA256SUMS ve iki lisans. Arşivdeki CRLF dahil dosya byte'ları Git blob'larında da korunmuştur. Teslim `SHA256SUMS` dosyası bu belge + 40 probe girdisi + 215 vendor girdisi, toplam **256 girdiyi** kapsar. Ayrı `REUSED-SHA256SUMS`, değişmeyen **20 dış fixture** girdisini içerir; kendi hash'i `57982e7460057385173291b93766c2a2a8ee7cd3ae3549f63bbad657f988fc3a`'dır. Yeni manifest hash'i teslim raporunda verilir; kendini içeren hash döngüsü kurulmaz.
