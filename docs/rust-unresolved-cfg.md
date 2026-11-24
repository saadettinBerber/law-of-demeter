# Rust etkin hata ve dışlanan cfg kodu

**Dar ayrım doğrulandı:** cfg! koşulu false iken içindeki çözülemeyen çağrı etkin kalır; cfg attribute'u kapalı item'ın ham AST'si kalır, etkin HIR bildirimi/gövdesi yoktur. Ayrıca receiver ve sonuç tipi bilinse bile method hedefi çözülemeyebilir. Bunlar dependency öğrenme testleridir; ürün hata/CLI politikası veya tamamlanmış Rust desteği değildir.

Taban `441891e7b6044833b47633d6f06658ae37e3ab02`. Yeni girdiler bu belge ve [unresolved-cfg](../examples/rust-backend-probe/experiments/unresolved-cfg/) altında **cases.rs, probe_tests.rs, fixture/Cargo.toml, fixture/Cargo.lock, fixture/lib.rs**. Önceki **32 probe + 20 dış fixture**, eski belgeler, kök Cargo/domain/production/decision.md değişmedi. Commit/push yoktur.

## Küçük fixture ve seçili bağlam

[Yeni manifest](../examples/rust-backend-probe/experiments/unresolved-cfg/fixture/Cargo.toml) bağımsız, dependency'siz **edition 2024** paketidir. Lib, Wallet::debit geçerli çağrısını ve `#[cfg(feature = "alternate")] conditional_call` item'ını içerir. Koşullu item'da hem missing_function hem Wallet üzerinde olmayan missing_debit vardır. Reserve üzerinde aynı adlı geçerli method bulunur; kısa ad eşleşmesi yanlış hedef seçtirmemelidir. missing_debit sonucunun `let balance: u32` bağlamı özellikle bilinen sonuç / bilinmeyen hedef karşı örneğidir; tipe adapter tarafından değer enjekte edilmez.

`cfg-boolean` bin hedefi, değişmeyen [onaylı cfg_boolean.rs](../examples/rust-expansion/consumer/negative/cfg_boolean.rs) dosyasını doğrudan source target yapar. RA kopyasında da kaynak byte olarak aynıdır. Wrapper'ın alternate özelliği bu küçük paket içindir; bütün expansion workspace'inin feature matrisini sınadığı iddia edilmez.

Yeni üç test normal lib/bin bağlamındadır: `set_test=false`, `rust_analyzer` override ile kapalı. cfg! testi mevcut normal yükleme/konum yardımcısını kullanır. Feature karşılaştırması için dar özel loader, `CargoConfig::features = CargoFeatures::Selected` üzerinden boş liste / `["alternate"]` seçer; yeni DB'ler birbirinin cfg'sini kullanmaz. Metadata/sysroot/load hataları Result ile taşınır; canonical source root ve gerçek VFS yolu denetlenir. AST item silinmez. Yeni fixture'da build.rs/proc yoktur; yeni loader bunları çalıştırmaz.

## Gerçek Semantics ve typed diagnostic çıktısı

| Seçili kaynak / feature | Etkinlik, hedef ve tip | Aynı kaynağa bağlı diagnostic |
|---|---|---|
| cfg_boolean.rs bin, alternate kapalı | main gerçek HIR Function'dır. Modelde feature atomu false; cfg! açılımı hatasız CfgPredExpr, tipi bool. missing_function callee için `resolve_expr_as_callable=None`; çağrı tipi Some fakat `contains_unknown=true` (`type error`). | **UnresolvedIdent**, gerçek dosyadaki callee **PATH_EXPR**, satır **3**. InactiveCode değildir. |
| lib, alternate kapalı | conditional_call ham AST'de gövdesiyle bulunur; `to_def=None`, etkin module bildirimi yoktur; ham çağrı için `type_of_expr=None`. Yakın Wallet::debit çağrısı doğru bağımsız bildirime çözülür. | **InactiveCode**, gerçek dosya ve koşullu **FN** pointer'ı. Etkin unresolved çağrı diagnostic'i yoktur. |
| Aynı lib, alternate açık | Aynı conditional_call AST'si gerçek HIR Function olur. missing_function callee hedefi None, çağrı tipi Some/unknown. missing_debit hedefi None iken receiver **&Wallet ADT**, sonuç **u32 / unknown=false**. Wallet::debit kontrolü doğru Function ve u32 tipini verir; Reserve::missing_debit yanlış aday olarak kullanılmaz. | **UnresolvedIdent** callee PATH_EXPR'a, **UnresolvedMethodCall** gerçek METHOD_CALL_EXPR'a bağlıdır. Method diagnostic'inin receiver tipi de bilinir ve ifade receiver tipiyle eşittir. InactiveCode yoktur. |

Diagnostic'ler yalnız sayılmaz: `UnresolvedIdent.node.file_id` ve gerçek ExprOrPatPtr; `UnresolvedMethodCall.expr.file_id` ve pointer; `InactiveCode.node` ayrı ayrı bağımsız kaynak AST düğümüyle karşılaştırılır. Gerçek API dosyası beklenen dosyayla değiştirilmez. Bu küçük snapshot'ta toplamlar sırasıyla **1 / 1 / 2**; beklenmeyen tür/scope testi kırar. Genel Rust hata sayısı/diagnostic tamlığı garantisi değildir.

None ve unknown burada farklı gözlemlerdir: dışlanan gövde için tip sorgusu yoktur; etkin serbest çağrıda hata tipi vardır; etkin method çağrısının sonuç tipi ise source bağlamından bilinir. Bilinen u32, çözülen hedef veya temiz analiz anlamına gelmez. Hiçbiri literal, yeni nesne veya izinli LoD sonucu olarak sınıflandırılmaz.

## Compiler karşı kontrolleri

Rust/Cargo **1.99.1**, rustc `58cf8f9c35b355199727627d07ae34a4e2846d67`, host aarch64-apple-darwin. Yeni fixture'da üç Cargo check koşusu yapıldı:

| Komut kapsamı | Gerçek sonuç |
|---|---|
| `--lib`, default feature | **exit 0**, temiz lib kontrolü. |
| `--bin cfg-boolean`, default feature | **exit 101 / E0425**; false cfg! dalındaki missing_function yine type-check edilir. |
| `--lib --features alternate` | **exit 101 / E0425 + E0599**; lib.rs:22 serbest çağrı, :23 &Wallet üzerinde eksik method. |

Compiler reddi beklenen kaynak kanıtıdır; kırmızı ürün testi değildir. Probe bu eksikliği doğru gözlediğinde yeşildir. Bilinçli bozuk bin ve açık-feature lib için başarılı Clippy iddiası yoktur; temiz default lib Clippy'si ayrıca geçti.

## API ve sürüm kanıtının sınırı

[Rust Reference](https://doc.rust-lang.org/reference/conditional-compilation.html), cfg attribute'u ile cfg! sonucunu ayırır. Sürümlü RA kaynağı ve gerçek sorgu bu örnekte birlikte kontrol edildi:

- [cfg_expand](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir-expand/src/builtin/fn_macro.rs) bir **CfgPredExpr** üretir; [lowering](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir-def/src/expr_store/lower.rs) predicate'i seçili cfg ile bool'a çevirir. Test bool literal AST'si beklemez. False seçimi gerçek crate cfg atomuyla denetlenir; false dalın yine hatalı olduğu compiler ile bağımsız doğrulanır. Public Semantics sorgusundan genel bir constant evaluator üretildiği iddia edilmez.
- [Semantics](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/semantics.rs) / [SourceAnalyzer](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/source_analyzer.rs) tip, callable ve method hedefini ayrı Optional sorgular olarak verir. Bu fixture'daki recovery tipi, çözülen hedef yerine kullanılmaz.
- [Typed diagnostic kaynakları](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/diagnostics.rs) UnresolvedIdent için source-map callee pointer'ını; UnresolvedMethodCall için expression pointer'ı ve receiver tipini; InactiveCode için item pointer/cfg/options bilgisini taşır. Testte mesaj/hover/debug metni semantic protokol olarak ayrıştırılmaz.

Bu **beş** primary dosya resmî `2ad81e72b7e978cb800dffe1bcce5c9f98032acf` içeriğiyle yayımlanmış 0.0.358 registry kaynağı arasında **5/5 byte eş** doğrulandı; `target/rust-unresolved-cfg/primary/verified.json` kayıtları içerir. Bunlar kararlı API garantisi değildir.

RA **=0.0.358**, dokuz doğrudan pin, **27 RA / 179 toplam lock paketi** korunur. [Onaylı edition yaması](rust-macro-edition-patch.md) ve strict probe patch'i yalnız izole kopyadadır; [source mapping testleri](rust-source-mapping.md) de bağlanarak önceki 15 test korunur. Yedi source hash'i upstream.json ile, gerçek path override'ları metadata ile denetlendi. Önceki `target/rust-source-mapping/source-patched` kopyaları **salt okunur** yeniden kullanıldı; bu dizine/registry'ye yazılmadı. Kalıcı probe override'ı veya yeni source patch yoktur.

## Gerçek doğrulama ve ilk yeşil fault'ları

İlk derlemede mevcut support modülünün kullanılmayan üyeleri `-D warnings` ile durdu; cfg! için literal AST varsayımı da doğru değildi. Mevcut normal loader/konum yardımı kullanıldı ve sürümlü API'ye uygun CfgPredExpr sorgusuna geçildi. Bu kurulum/API öğrenmesi hataları ürün TDD kırmızısı sayılmaz; false cfg beklentisi veya compiler sonucu değiştirilmedi.

| Bu dilimdeki koşu / log | Sonuç |
|---|---|
| `focused-final.log` | Yeni öğrenme testleri **3/3**, exit **0**. |
| `regression-18.log` | Önceki 15 + yeni 3 **18/18**, exit **0**. |
| `locked-offline-repeat.log` | Aynı doğru kaynakla **18/18**, exit **0**. |
| `check.log`, `clippy.log`, `probe-fmt.log`, `cases-fmt.log`, `entry-fmt.log` | Probe tüm hedeflerde check/Clippy ve ilgili format **exit 0**. Relocate edilen giriş dosyası artifact formatında `skip_children=true` kullanılır; gerçek tests yolundaki modül ağacı ayrıca cargo fmt/Clippy ile kontrol edilir. Yeni lint muafiyeti yoktur. |
| `fixture-clippy.log`, `fixture-fmt.log` | Default temiz lib Clippy ve fixture formatı **exit 0**. |
| `restored-3.log` | Fault kopyasında doğru kaynak geri alınıp paket yeniden derlendi: **3/3**, exit **0**. |

Dört yeni ilgili fault, fixture/beklentiler sabitken yalnız query/config glue'sunu değiştirir:

| Fault | Yakalanan ayrım |
|---|---|
| wrong-feature | Kapalı-item testinde gerçek CargoConfig'e alternate seçilir; bağımsız kapalı cfg beklentisi assertion'da kırılır. |
| wrong-method-query | Eksik method yerine yakın geçerli debit çağrısının gerçek hedefi sorgulanır; beklenen None karşısında Some(Function) kırılır. |
| wrong-diagnostic-node | API diagnostic'inin gerçek callee düğümü yerine gerçek üst CallExpr düğümü seçilir; PATH_EXPR/CallExpr provenance assertion'ı kırılır. Geçersiz ID kullanılmaz. |
| wrong-result-query | Hatalı çağrı yerine aynı main içindeki gerçek bool condition'ın tipi sorgulanır; unknown beklentisi bilinen bool ile kırılır. |

**4/4 fault derlendi, runtime assertion'da exit 101 verdi.** Komutlar, kaynak hash'leri ve sonuçlar `fault-results.json`; bozuk kaynaklar/loglar aynı target dizinindedir. Beklenen değer değiştirmek veya compiler/setup reddi fault başarısı değildir. Ayrı salt okunur Clean Code/test okunabilirliği geçişi yapıldı; bulgu yok, bu geçiş kaynak değiştirmedi.

Önceki 15 test bu dilimde regresyon olarak gerçekten tekrar koşuldu; bu, eski kapsamlı build/proc/ABI/fault kanıtlarının yeniden üretildiği veya yeni genel hata politikasının tamamlandığı anlamına gelmez. Geniş core/workspace/Maven, eski tüm fault'lar ve upstream paketleri koşulmadı. Yamasız edition kabulünün tarihsel başarısızlığı değiştirilmedi.

## Exact komutlar ve yeniden üretim

Repo kökünden, temiz yazar hedefinde. Yeni fixture Cargo.lock ilk kez `cargo generate-lockfile --offline --manifest-path examples/rust-backend-probe/experiments/unresolved-cfg/fixture/Cargo.toml` ile üretildi; teslim lock'u tek yerel paket içerir. Ana probe lock'u yeniden üretilmedi.

```sh
out="$PWD/target/rust-unresolved-cfg"
art="$PWD/examples/rust-backend-probe/experiments"
patched="$PWD/target/rust-source-mapping/source-patched"
mkdir -p "$out/layout/examples"
cp -R examples/rust-backend-probe examples/rust-scenarios examples/rust-method-resolution examples/rust-visibility examples/rust-expansion "$out/layout/examples/"
probe="$out/layout/examples/rust-backend-probe"
(cd "$probe" && patch --batch --fuzz=0 -p1 -i "$art/edition-patch/probe-experiment.patch")
(cd "$probe" && patch --batch --fuzz=0 -p1 -i "$art/source-mapping/include-tests.patch")
cat "$art/edition-patch/order_cases.rs" >> "$probe/fixtures/macro-edition/consumer/src/lib.rs"
cp "$art/unresolved-cfg/probe_tests.rs" "$probe/tests/unresolved_cfg.rs"
cat >> "$probe/Cargo.toml" <<EOF

[patch.crates-io]
ra_ap_hir_def = { version = "=0.0.358", path = "$patched/ra_ap_hir_def-0.0.358" }
ra_ap_hir_ty = { version = "=0.0.358", path = "$patched/ra_ap_hir_ty-0.0.358" }
EOF
CARGO_TARGET_DIR="$out/correct" cargo metadata --offline --format-version 1 --manifest-path "$probe/Cargo.toml" > "$out/metadata.json"
probe_cargo() {
  env CARGO_TARGET_DIR="$out/correct" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 RUSTFLAGS='-D warnings' cargo "$@"
}
probe_cargo clean --manifest-path "$probe/Cargo.toml" -p rust-backend-probe
probe_cargo test --locked --offline --manifest-path "$probe/Cargo.toml" --test unresolved_cfg -- --test-threads=1 --nocapture
probe_cargo test --locked --offline --manifest-path "$probe/Cargo.toml" --tests -- --test-threads=1 --nocapture
probe_cargo test --locked --offline --manifest-path "$probe/Cargo.toml" --tests -- --test-threads=1
probe_cargo check --locked --offline --manifest-path "$probe/Cargo.toml" --all-targets
probe_cargo clippy --locked --offline --manifest-path "$probe/Cargo.toml" --all-targets -- -D warnings
cargo fmt --manifest-path "$probe/Cargo.toml" --all --check
rustfmt --edition 2024 --check "$art/unresolved-cfg/cases.rs"
rustfmt --edition 2024 --config skip_children=true --check "$art/unresolved-cfg/probe_tests.rs"
```

Onaylı source kopyası yoksa [önceki arşiv/checksum/yama adımları](rust-macro-edition-patch.md#exact-yeniden-üretim) yazarın yeni target'ında uygulanabilir ve yalnız izole override yolu buna göre verilir. Bu koşuda eski source kopyası hash kontrolüyle kullanıldı; kaynak değiştirilmedi. Sysroot `/Users/deep/.rustup/toolchains/stable-aarch64-apple-darwin`, rust-src `lib/rustlib/src/rust/library` ve mevcut proc server önceki regresyon için korunur; yeni component/toolchain/global ayar yoktur.

Compiler kontrollerinin exact feature/target seçimleri:

```sh
fixture="$art/unresolved-cfg/fixture/Cargo.toml"
fixture_cargo() {
  env CARGO_TARGET_DIR="$out/compiler" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 RUSTFLAGS='-D warnings' cargo "$@"
}
fixture_cargo check --locked --offline --manifest-path "$fixture" --lib
fixture_cargo check --locked --offline --manifest-path "$fixture" --bin cfg-boolean
fixture_cargo check --locked --offline --manifest-path "$fixture" --lib --features alternate
fixture_cargo clippy --locked --offline --manifest-path "$fixture" --lib -- -D warnings
cargo fmt --manifest-path "$fixture" --all --check
```

Cache yalnız yazarın `target/rust-source-mapping/correct` çıktısından APFS `cp -cR` ile alındı; taze tüm dependency derlemesi iddiası yoktur. Yeni correct/fault hedefleri ayrı, debug=0/incremental=0 ve build jobs=1'dir. Fault kopyasında her source değişiminde yalnız kendi probe paketi temizlendi. Model/proc/build çıktıları `target/rust-unresolved-cfg` altındadır; eski/reviewer/canonical target'a yazılmadı. Önceki 15'in izinli yerel build/proc kodu ve normal dependency build dışında proje kodu çalıştırılmadı.

Fault komut biçimi (`test_name` fault-results.json'daki exact addır):

```sh
fault_manifest="$out/fault-layout/examples/rust-backend-probe/Cargo.toml"
CARGO_TARGET_DIR="$out/fault" cargo clean --manifest-path "$fault_manifest" -p rust-backend-probe
CARGO_TARGET_DIR="$out/fault" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 RUSTFLAGS='-D warnings' cargo test --locked --offline --manifest-path "$fault_manifest" --test unresolved_cfg "$test_name" -- --exact --nocapture
```

## Teslim ve açık kararlar

`target/rust-unresolved-cfg/SHA256SUMS` bu belgeyi ve mevcut 32 dahil bütün teslim probe/deney girdilerini, toplam **38 girdiyi**, kapsar. Dış 20 fixture ayrı REUSED-SHA256SUMS içindedir; hash **57982e7460057385173291b93766c2a2a8ee7cd3ae3549f63bbad657f988fc3a** ile değişmedi. Ana lock, Salsa/Unicode seçimleri, onaylı patch'ler ve önceki kaynaklar değişmedi.

Bu kontrollü ayrım doğrulandı; genel backend deneyi/nihai seçim tamamlanmadı. Kalıcı kaynak/paket tercihi, destek sınırları, eksik analiz raporu/CLI çıkışı, genel dedup ve ürün SourceReader/TypeSource/CheckProject/LoD uygulaması ayrı açık kararlardır. Parse/load başarısı, tanı yokluğu veya bilinen tip genel temiz analiz kanıtı değildir.
