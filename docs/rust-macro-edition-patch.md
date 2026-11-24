# Rust metot token edition: izole kaynak yaması

**Dar hipotez doğrulandı:** RA 0.0.358'in gerçek metot adı token edition'ını aday aramasına taşıyan kaynak kopyası, aynı borrowed kabulünü geçti. Yamasız kaynak assertion'da kaldı; override kaldırılınca aynı assertion yeniden kaldı. Bu yayımlanmış bir düzeltme veya ürün backend seçimi değildir. Repodaki probe, pinleri ve başarısız explicit diagnostic aynen durur.

Taban `3ab37d4c6edd26427995adeda1736e868f53443b`. Önceki [teşhis](rust-macro-edition.md) ve [sürüm araştırması](rust-backend-version.md) tarihsel kanıttır. Bu dilimde yalnız bu belge ve [edition-patch deney girdileri](../examples/rust-backend-probe/experiments/edition-patch/) eklendi; mevcut 25 probe ve 20 dış fixture girdisi değişmedi. Kök Cargo, domain, production ve decision.md değişmedi.

## Kaynak ve yama sınırı

[Rust Reference](https://doc.rust-lang.org/reference/expressions/method-call-expr.html#r-expr.method.edition2021) array IntoIterator adayının edition filtresini **metot adı token'ına** bağlar. Resmî **ra_ap_hir_def/ra_ap_hir_ty 0.0.358** arşivleri indirildi; SHA-256'ları crates.io sürüm metadata'sıyla eşleşti. Arşivlerin `.cargo_vcs_info.json` kaydı `2ad81e72b7e978cb800dffe1bcce5c9f98032acf`, `dirty=true` der; yeniden üretim tabanı commit varsayımı değil, checksum'lı yayımlanmış arşivdir.

| Resmî arşiv | SHA-256 |
|---|---|
| [ra_ap_hir_def 0.0.358](https://crates.io/api/v1/crates/ra_ap_hir_def/0.0.358/download) | `7ef811f35ed1c394d27a7e0d79869aa4891108496736597b87bbc632a4eee1ec` |
| [ra_ap_hir_ty 0.0.358](https://crates.io/api/v1/crates/ra_ap_hir_ty/0.0.358/download) | `725f913d562b8d794035bc3bcc3e62adf09b843e11b443750bf8a7027ec1d3bf` |

[method-token-edition.patch](../examples/rust-backend-probe/experiments/edition-patch/method-token-edition.patch) yalnız yazarın `target/rust-edition-patch/source-patched` kopyalarına uygulandı:

- `hir_def::Expr::MethodCall` bir `method_edition: syntax::Edition` taşır. `expr_store/lower.rs`, gerçek name_ref aralığının `SpanMap::span_for_range(...).ctx.edition(db)` bilgisini alır. Eksik metot token'ında mevcut dosya edition'ı yedektir; bu bozuk kaynak desteği kanıtı değildir.
- `InferenceContext::lookup_probe`, bu alanla yerel bir `MethodResolutionContext` kurar. Genel `InferenceContext.edition` yazılmaz. Başka expression türleri mevcut davranışı korur.
- Diğer beş dosya ek alanı tanımlayan veya mevcut match'leri alanı yok sayacak biçimde uyarlayan küçük değişikliklerdir. Toplam **7 dosya**, **2 yerel RA crate'i** değişir. Lisanslar ve değişmeyen üçüncü taraf yorumları korunur; yeni kod yorumu yoktur.

İlgili sürümlü akış: [lowering](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir-def/src/expr_store/lower.rs), [lookup_probe / with_method_resolution](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir-ty/src/method_resolution.rs), [aday filtresi](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir-ty/src/method_resolution/probe.rs). Son dosyada array filtresi `ctx.edition.at_least_2021()` kullanır. Aynı context boxed-slice 2024 filtresini ve görünür trait aramasındaki başka edition dalını da besler; **bunlara yeni kapsama iddiası yoktur**. Completion, `Expr::Field` üzerinden eksik metot araması, path fallback ve diğer resolver yolları bütünüyle yeniden tasarlanmadı.

Arşiv kimlikleri, yama hash'leri, yedi dosyanın önce/sonra hash'leri [upstream.json](../examples/rust-backend-probe/experiments/edition-patch/upstream.json) içindedir. Temiz arşiv kopyasına `patch --batch --fuzz=0 -p1` yeniden uygulandı; yedi sonuç hash'i eşleşti. Registry kaynakları değiştirilmedi; ilgili yedi registry dosyası temiz arşiv hash'leriyle de eşleşti.

## Gerçek karşılaştırma

Rust/Cargo **1.99.1**, rustc `58cf8f9c35b355199727627d07ae34a4e2846d67`, host `aarch64-apple-darwin`. Explicit sysroot `/Users/deep/.rustup/toolchains/stable-aarch64-apple-darwin`; rust-src bunun `lib/rustlib/src/rust/library`, proc server `libexec/rust-analyzer-proc-macro-srv` yolundadır. Compiler/component/global config kurulmadı veya değiştirilmedi.

İlk üç koşuda kaynak, borrowed beklentisi ve loader cfg **aynı** kaldı; yalnız izole Cargo kaynak override'ı değişti. Test cfg seçimi, rust_analyzer override'ı, crate/macro/Function kimlikleri, unknown ve tanı denetimleri önceki diagnostic'ten korunur. Debug çıktıları tanı içindir; assertion'lar display metni ayrıştırmaz.

| Bu dilimdeki koşu | Gözlem / exit |
|---|---|
| Yamasız 0.0.358, `baseline-red.log` | Doğrudan 2018 ve forwarded 2024 kontrolü geçti; doğru borrowed sonuç/hedef/tanı assertion'ı **101**. |
| İki kaynak override'ı, `patched-first.log` | Aynı kaynak/beklenti **0**; legacy sonuç slice Iter, shared borrow adjustment ve borrowed referans çağrısıyla aynı Function kimliği; test tanıları boş. |
| Override kaldırılmış kaynak, `reverted-red.log` | Registry kimlikleri metadata ile doğrulandı; aynı borrowed assertion yeniden **101**. Bu setup/derleme hatası değildir. |
| Sıra karşı kontrolleri eklenmiş yamalı diagnostic, `patched-orders.log` ve locked/offline `patched-orders-repeat.log` | İkisi de **0**; legacy→modern ve modern→legacy aynı gövdede doğru tip/hedefleri korudu. |
| Değişmeyen compiler tanıkları, `compiler-original.log` | **2/2**, exit **0**; provider'da 0 test. |
| İki sıra karşı kontrolü eklenen compiler fixture, `compiler-orders.log` | **4/4**, exit **0**; önceki iki + yeni iki test. Bu 6 farklı test değil, 4 farklı testin iki koşusudur. |

[order_cases.rs](../examples/rust-backend-probe/experiments/edition-patch/order_cases.rs) yalnız izole consumer kaynağına eklenir. Legacy sonuç orijinal String referansına `ptr::eq` ile bağlı kalır; modern String `into_bytes` ile tüketilir. Sonuç iterator'larına annotation eklenmez. [order_queries.rs](../examples/rust-backend-probe/experiments/edition-patch/order_queries.rs) her iki sırada gerçek MacroCall kimliğini, expanded metot token edition'ını, unknown olmayan sonuç tipini ve Function hedefini sorgular. Beklenti aynı DB'nin bağımsız açık borrowed/owned referans çağrılarındandır; iki kontrol tipi/hedefi ayrıca birbirinden farklıdır. Yalnız makro gövdesini doğrudan çağırmak veya sayısal payload eşitliğiyle yetinmek söz konusu değildir.

## Eski expansion ve hata sınırı

[probe-experiment.patch](../examples/rust-backend-probe/experiments/edition-patch/probe-experiment.patch) yalnız izole probe'da uygulanır. Diagnostic'e sıra sorgusunu bağlar; eski proc testindeki dar legacy tanı muafiyetini **tamamen kaldırıp** mevcut `assert_local_diagnostics` çağrısını kullanır. Bu yardımcı consumer'ın tüm modüllerindeki **InactiveCode dışındaki tüm tanıları reddeder**. Repodaki asıl diagnostic/test değiştirilmez.

Yamalı izole probe'da eski **12 test** bu sınırla çalıştırıldı: expansion **4**, semantics **5**, traits **3**. Kimlik, cfg, kaynak ve unknown kontrolleri korunur. Base/bonus build.rs ve mevcut yerel function-like proc kodu gerçek olarak çalıştı; başka kullanıcı projesi yüklenmedi. Eski legacy TypeMismatch, muafiyet olmadan artık gözlenmedi.

Ayrı gerçek fixture kopyasında `Literal::i32_unsuffixed(13)` → `Literal::u32_suffixed(13)` yapıldı. Aynı numeric 13 ve hedef beklentileri değişmedi. Yamalı RA ile exact proc testi **runtime assertion'da exit 101** verdi: `unexpected consumer diagnostics`, expanded literal için beklenen i32 / gerçek u32 TypeMismatch. Sayısal 13 denetimi bu assertion'dan önce geçti. Aynı bozuk kaynak `cargo check --workspace --tests` ile **E0308 / exit 101** verdi. Compiler reddi ürün TDD kırmızısı sayılmaz; diagnostic kabul sınırını gösteren kanıt probe assertion'ıdır. Kopyanın proc/build çıktıları ayrı hedefte yeniden üretildi.

Kurulumda iki başarısız deneme ayrıca ayrıldı: başlangıçtaki `cargo generate-lockfile --offline` transitive Unicode seçimini yenileyip E0080 verdi; bu lock reddedildi. Orijinal lock'tan `cargo metadata --offline` ile yalnız iki path kaydı çözüldü. **179 paket / 27 RA 0.0.358** korunur; tüm sürüm ve dependency listeleri aynıdır, yalnız hir_def/hir_ty registry source/checksum alanları path override nedeniyle yoktur. Salsa/Unicode pinleri değişmedi. İlk regresyonda üç trait sorgusu, symlink üzerinden Cargo kökü ile canonical VFS yolu eşleşmediği için `fixture crate was not loaded` dedi; dış girdilerin byte eş gerçek kopyalarıyla düzeltildi. Bunlar davranış kırmızısı veya yamanın başarısı değildir.

## Exact yeniden üretim

Repo kökünden; `out` yalnız bu çalışma ağacının hedefidir. Aşağıdaki kurulum temiz kopyalar içindir; orijinal registry/probe/fixture'lara yazmaz. Arşivleri `upstream.json` içindeki SHA-256'larla doğrula.

```sh
out="$PWD/target/rust-edition-patch"
art="$PWD/examples/rust-backend-probe/experiments/edition-patch"
mkdir -p "$out/source-clean" "$out/source-patched" "$out/layout/examples"
curl -fL https://crates.io/api/v1/crates/ra_ap_hir_def/0.0.358/download -o "$out/ra_ap_hir_def-0.0.358.crate"
curl -fL https://crates.io/api/v1/crates/ra_ap_hir_ty/0.0.358/download -o "$out/ra_ap_hir_ty-0.0.358.crate"
shasum -a 256 "$out/ra_ap_hir_def-0.0.358.crate" "$out/ra_ap_hir_ty-0.0.358.crate"
tar -xzf "$out/ra_ap_hir_def-0.0.358.crate" -C "$out/source-clean"
tar -xzf "$out/ra_ap_hir_ty-0.0.358.crate" -C "$out/source-clean"
cp -R "$out/source-clean/." "$out/source-patched/"
(cd "$out/source-patched" && patch --batch --fuzz=0 -p1 -i "$art/method-token-edition.patch")
cp -R examples/rust-backend-probe "$out/layout/examples/"
cp -R examples/rust-scenarios examples/rust-method-resolution examples/rust-visibility examples/rust-expansion "$out/layout/examples/"
probe="$out/layout/examples/rust-backend-probe"
cp "$probe/Cargo.toml" "$out/baseline-Cargo.toml"
cp "$probe/Cargo.lock" "$out/baseline-Cargo.lock"
cat >> "$probe/Cargo.toml" <<EOF

[patch.crates-io]
ra_ap_hir_def = { version = "=0.0.358", path = "$out/source-patched/ra_ap_hir_def-0.0.358" }
ra_ap_hir_ty = { version = "=0.0.358", path = "$out/source-patched/ra_ap_hir_ty-0.0.358" }
EOF
CARGO_TARGET_DIR="$out/patched" cargo metadata --offline --format-version 1 --manifest-path "$probe/Cargo.toml" > "$out/patched-metadata.json"
cp "$probe/Cargo.toml" "$out/patched-Cargo.toml"
cp "$probe/Cargo.lock" "$out/patched-Cargo.lock"
```

Baseline koşusu override eklemeden, ilk yamalı koşu ekledikten sonra yapılır. Geri alma koşusunda yalnız izole manifest/lock baseline kopyalarından geri kondu; sonra patched kopyalar geri yüklendi. Her bağlam ayrı `baseline`, `patched`, `fault/reverted` target kullandı. Metadata iki patched paketin gerçek manifest yollarını; build log'ları bunların derlendiğini doğrular. Cache APFS `cp -cR` ile yalnız yazarın önceki `target/rust-macro-edition/cargo` çıktılarından ve kendi yeni hedeflerinden kopyalandı; taze tüm dependency derlemesi iddiası yoktur. Paket konumu değiştiğinde yalnız kendi probe paketi `cargo clean -p rust-backend-probe` ile temizlendi. Fault kopyasında `analyzed-*` dizinleri kaldırılarak eski proc DLL/OUT_DIR kullanımı engellendi.

```sh
probe_cargo() {
  env CARGO_TARGET_DIR="$out/patched" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RUSTFLAGS='-D warnings' cargo "$@"
}
probe_cargo run --locked --offline --manifest-path "$probe/Cargo.toml" --example macro_edition_gate
cp "$out/baseline-Cargo.toml" "$probe/Cargo.toml"
cp "$out/baseline-Cargo.lock" "$probe/Cargo.lock"
CARGO_TARGET_DIR="$out/fault/reverted" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RUSTFLAGS='-D warnings' cargo run --locked --offline --manifest-path "$probe/Cargo.toml" --example macro_edition_gate
cp "$out/patched-Cargo.toml" "$probe/Cargo.toml"
cp "$out/patched-Cargo.lock" "$probe/Cargo.lock"
(cd "$probe" && patch --batch --fuzz=0 -p1 -i "$art/probe-experiment.patch")
cat "$art/order_cases.rs" >> "$probe/fixtures/macro-edition/consumer/src/lib.rs"
probe_cargo run --locked --offline --manifest-path "$probe/Cargo.toml" --example macro_edition_gate
probe_cargo test --locked --offline --manifest-path "$probe/Cargo.toml" --tests -- --test-threads=1
probe_cargo check --locked --offline --manifest-path "$probe/Cargo.toml" --all-targets
probe_cargo clippy --locked --offline --manifest-path "$probe/Cargo.toml" --all-targets -- -D warnings
cargo fmt --manifest-path "$probe/Cargo.toml" --all --check
cargo fmt --manifest-path "$probe/fixtures/macro-edition/Cargo.toml" --all --check
rustfmt --edition 2024 --check "$art/order_cases.rs" "$art/order_queries.rs"
```

Compiler tanıkları için aynı debug/incremental/warnings ayarlarıyla ayrı hedefler kullanıldı:

```sh
CARGO_TARGET_DIR="$out/compiler" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RUSTFLAGS='-D warnings' cargo test --locked --offline --manifest-path examples/rust-backend-probe/fixtures/macro-edition/Cargo.toml --workspace --lib
CARGO_TARGET_DIR="$out/compiler-orders" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RUSTFLAGS='-D warnings' cargo test --locked --offline --manifest-path "$probe/fixtures/macro-edition/Cargo.toml" --workspace --lib
CARGO_TARGET_DIR="$out/compiler-orders" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RUSTFLAGS='-D warnings' cargo clippy --locked --offline --manifest-path "$probe/fixtures/macro-edition/Cargo.toml" --workspace --all-targets -- -D warnings
```

13u32 koşusunda `fault-layout/examples/rust-backend-probe` patched probe kopyası, kardeş `rust-expansion` ise değiştirilen gerçek fixture kopyasıdır. Exact komutlar:

```sh
mkdir -p "$out/fault-layout/examples"
cp -R "$probe" "$out/fault-layout/examples/"
cp -R "$out/layout/examples/rust-expansion" "$out/fault-layout/examples/"
python3 - "$out/fault-layout/examples/rust-expansion/generator/src/lib.rs" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
source = path.read_text()
assert source.count('Literal::i32_unsuffixed(13)') == 1
path.write_text(source.replace('Literal::i32_unsuffixed(13)', 'Literal::u32_suffixed(13)'))
PY
CARGO_TARGET_DIR="$out/fault/proc-type" cargo clean --manifest-path "$out/fault-layout/examples/rust-backend-probe/Cargo.toml" -p rust-backend-probe
CARGO_TARGET_DIR="$out/fault/proc-type" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RUSTFLAGS='-D warnings' cargo test --locked --offline --manifest-path "$out/fault-layout/examples/rust-backend-probe/Cargo.toml" --test expansion the_real_proc_server_expands_calls_resolving_to_the_consumers_wallet -- --exact --nocapture
LOD_FIXTURE_MODE=base CARGO_TARGET_DIR="$out/fault/compiler-type" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RUSTFLAGS='-D warnings' cargo check --locked --offline --manifest-path "$out/fault-layout/examples/rust-expansion/Cargo.toml" --workspace --tests
```

Probe check/Clippy tüm hedefler, probe ve sıra fixture'ının formatı, yeni Rust artifact formatı ve sıra fixture Clippy'si **exit 0** verdi. Değişen yedi üçüncü taraf dosya resmî commit'in rustfmt ayarıyla (`edition=2024`, `reorder_modules=true`, `use_small_heuristics=Max`, `skip_children=true`) format denetiminden geçti. Ek olarak probe manifestinden `cargo clippy --locked --offline --manifest-path ... -p ra_ap_hir_def -p ra_ap_hir_ty --lib -- -D warnings` denemesi Cargo feature resolver panikiyle exit 101 verdi; upstream crate'lerin bağımsız Clippy başarısı iddia edilmez. Derlenen dependency kaynakları probe check/regresyonunda doğrulanmıştır; tüm upstream RA test paketi çalıştırılmadı.

## Teslim ve açık kapılar

Yeni kod ve testler için ayrı **salt okunur okunabilirlik geçişi** yapıldı: iki çağrı sırası, belirleyici referans/sahiplik tanıkları, gerçek kimlik beklentileri ve hata kabul sınırı incelendi; bu geçiş dosya değiştirmedi. İlk yeşil yama sonucunu geri alınmış override gerçek assertion ile ayırt etti. Beklentiler backend'in yanlış sonucuna çevrilmedi; ignore/should_panic veya başarısızlığı success'e çeviren akış yoktur.

Manifest `target/rust-edition-patch/SHA256SUMS`, bu belgeyi ve mevcut 25 probe girdisi dahil bütün teslim probe girdilerini kapsar. `REUSED-SHA256SUMS` dış 20 girdiyi kapsar; hash'i `57982e7460057385173291b93766c2a2a8ee7cd3ae3549f63bbad657f988fc3a` ile değişmedi. Derleme/vendor ağaçları teslim edilmez. `upstream.json`, küçük source patch ve probe patch temiz kaynaklara yeniden uygulanabilir; onaylı fixture'lar değiştirilmez.

**Sınır:** sonuç, bu token-edition nedenini ve kontrollü çağrı yollarını doğrular. Yama upstream kabulü, bakım/dağıtım maliyeti veya bütün Rust çözümlemesi kanıtı değildir. Kalıcı patched dependency/fork/vendor alınmadı; yayımlanmış 0.0.358'in doğru borrowed kabulünü geçtiği söylenemez. Source mapping/dedup, active unresolved/inactive cfg kapıları, genel backend deneyi ve nihai seçim açık kalır. SourceReader/TypeSource/CheckProject, LoD/Value/Hybrid/CLI politikası uygulanmadı. Geniş core/workspace/Maven ve önceki tüm fault paketleri koşulmadı.
