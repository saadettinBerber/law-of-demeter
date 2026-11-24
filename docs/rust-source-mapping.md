# Rust açılım çağrılarının kaynak ve kimlik eşlemesi

**Üç dar öğrenme testi geçti:** tek declarative invocation'ın iki charge çağrısı ayrı anahtarlara sahiptir; aynı snapshot'ta tekrar sorgu aynı anahtarları verir. Proc ve include kaynakları gerçek Cargo/RA modelinde ayrı doğrulandı. Kaynak konumu, çağrı kimliği ve makro bildirimi aynı bilgi değildir. Genel traversal/dedup veya ürün raporu uygulanmadı.

Taban `ed01ed2383844ece8fe2884f52c40f14cd86f164`. Yeni girdiler yalnız bu belge, [source_mapping_cases.rs](../examples/rust-backend-probe/experiments/source-mapping/source_mapping_cases.rs) ve [include-tests.patch](../examples/rust-backend-probe/experiments/source-mapping/include-tests.patch). Mevcut **30 probe**, **20 dış fixture** ve önceki belgeler değişmedi. Domain, kök Cargo, production ve decision.md değişmedi; commit/push yoktur.

## Seçili deney bağlamı

RA ailesi **0.0.358**, 27 RA / 179 toplam kilitli paket, dokuz doğrudan exact pin korunur. Yalnız izole probe manifestinde hir_def/hir_ty path override'ı vardır. [Onaylı edition yaması](rust-macro-edition-patch.md) yeniden icat edilmedi: önceki yazar kopyasından alınan yedi kaynak dosyasının hash'leri [upstream.json](../examples/rust-backend-probe/experiments/edition-patch/upstream.json) ile doğrulandı. Yama hash'i `b6f555c62e922cc341e961ba64d410025cf3f3c23e949f6be4f03f1100b71a2e`; yeni source patch yoktur. Registry kaynaklarına yazılmadı.

Rust/Cargo **1.99.1**, rustc `58cf8f9c35b355199727627d07ae34a4e2846d67`, host aarch64-apple-darwin. Explicit sysroot `/Users/deep/.rustup/toolchains/stable-aarch64-apple-darwin`, eşleşen rust-src `lib/rustlib/src/rust/library`, proc server `libexec/rust-analyzer-proc-macro-srv`. Toolchain/component/global config değişmedi.

[Mevcut expansion workspace'i](../examples/rust-expansion/Cargo.toml) üç gerçek yerel crate olarak byte eş **gerçek dizin kopyalarıyla** yüklendi; symlink kullanılmadı. Yeni iki makro testi `LOD_FIXTURE_MODE=base`, default feature, `set_test=true` ve gerçek proc sunucusunu kullanır. Include testi base/default normal lib (`set_test=false`, proc kapalı) kullanır. `rust_analyzer` cfg override'ı, explicit std yolları, locked/offline metadata, Cargo build hata kontrolü ve proc istemcisinin ömrü önceki loader'dan aynen korunur.

[Önceki probe-experiment.patch](../examples/rust-backend-probe/experiments/edition-patch/probe-experiment.patch) izole kopyada uygulanır: eski legacy tanı istisnası kaldırılır. Yeni testler dahil consumer'ın bütün modüllerinde **InactiveCode dışındaki her tanı reddedilir**. Yeni dosya expansion testinin çocuk modülüdür; mevcut özel loader/çapa yardımcılarını kullanır. Yeni parser, dependency, framework veya kalıcı helper API yoktur.

## Gözlenen kaynak ve çağrı kimliği

Çağrı anahtarı `InFile<SyntaxNodePtr>`: `Semantics::hir_file_for(node)` ile gerçek `HirFileId`, aynı node'dan pointer. MacroFile içindeki kimlik gerçek `MacroCallId`'yi içerir. Hedef Function ayrıca çözülür; anahtar Function'ın kendisi veya `(dosya, satır, ad)` değildir. **Yalnız aynı DB/snapshot içindeki** tekrar sorgu kanıtlanır; süreçler/snapshot'lar arasında kalıcılık iddiası yoktur.

| Kaynak / hedef kapsam | Bağımsız beklenti ve gerçek sonuç |
|---|---|
| consumer `one_declarative_invocation_generates_two_distinct_charges`, `charge_twice!(&order)` | Tek MacroCallId; charge düğümleri expanded aralık **28..43 / 103..120**, farklı pointer'lar. İkisi de aynı consumer **Wallet::charge Function**'ına, **&Wallet ADT**'sine çözülür; argümanlar **7 / 11**. Tek invocation'ı yeniden expand etmek iki anahtarı da korur. |
| Aynı declarative çağrıların konumu | `original_range_opt` ikisinde **None**. `original_range` ikisinde **consumer/src/lib.rs:107**, `expansion_legacy::charge_twice!` aralığı **2285..2316**. Bu gerçek konum çakışması iki çağrıyı birleştirme gerekçesi değildir. |
| Declarative tanım/token kaynağı | Macro kimliği, aynı DB'de bağımsız parse edilen legacy bildirimiyle eşittir. `Semantics::source(Macro)` gerçek legacy dosyasındaki makro AST'sidir. Metot adı span'ları ayrı gerçek `charge` token'larına gider. Nonroot tüm düğüm eşlemesi **legacy/src/lib.rs:5 / :8**, `first.charge(7)` **113..128**, `second.charge(11)` **219..236** verir; context root değildir. Bu rooted rapor aralığından farklı bilgidir. |
| consumer `a_function_like_proc_macro_generates_a_charge_in_the_consumer` | Gerçek function-like proc açılımı: charge **1..18**, balance **19..35**, ayrı düğüm anahtarları; doğru consumer Function/Wallet ADT kimlikleri, unknown olmayan dönüşler ve charge miktarı **13**. Proc Macro bildiriminin kaynağı generator crate'indeki gerçek `generated_charge` fonksiyonudur. |
| Proc konumları | İki çağrıda `original_range_opt=None`; rooted `original_range` **consumer/src/lib.rs:118** içindeki **wallet** girdisine, **2652..2658**'e gider. Bu tam çağrı veya generator gövdesi aralığı değildir. Nonroot tüm-düğüm yolu ortak anchor/context bulamayınca consumer'daki macro path+! **2613..2651**'e döner. Tek metot token'ı span sorgusu ise bu fixture/protokolde **generator/src/lib.rs:4**, bildirim adı `generated_charge` **120..136**, nonroot context verir. Bu anchor, `Ident::new("charge", ...)` kaynak ifadesinin kesin konumu diye sunulmaz. |
| build-generated `generated_charge`, normal lib | Gerçek OUT_DIR/VFS generated.rs dosyası consumer crate'ine aittir ve yazarın target dizini altındadır. Aktif include çağrısı **MacroFile + 36..52**, fiziksel parse çağrısı **FileId + 39..55**: anahtarlar farklı. Aktif çağrı doğru consumer Wallet::charge ve &Wallet'a çözülür; miktar **7**. `original_range_opt` ve `original_range` fiziksel generated.rs **39..55** aralığına eşittir. |
| Include ham/aktif ilişkisi | `original_file` consumer köküne, `original_file_respecting_includes` gerçek generated.rs dosyasına gider. `include_id.call_node` consumer'daki bağımsız ham `include!` AST'sidir. Ham charge adı token'ının gerçek `span_map` kaydı `map_range_down_exact` ile **tek aktif çağrı anahtarına** geri eşlenir. API'nin döndürdüğü `file_id`, beklenen include ID'siyle ve eşlenen düğümün `Semantics::hir_file_for` sonucu ile ayrı doğrulanır; anahtar API'den gelen dosya kimliğiyle kurulur. Aynı Function'ın kaynağını yeniden istemek de aynı aktif anahtarı verir. Ham generated Fn/charge doğrudan `to_def`/`resolve_method_call` sorgusunda **None** verir; bu, ham parse'ın aktif HIR gövdesi yerine geçmediğinin gözlemidir, uygun/temiz analiz sonucu değildir. |

Tablodaki sayısal dosya aralıkları bu değişmeyen fixture snapshot'ına aittir; anahtarlar bu sayılardan elle üretilmez. FileId/MacroCallId debug sayıları kalıcı protokol değildir. Beklenen Function/ADT/Macro kimlikleri bağımsız kaynak bildirimlerinden aynı DB'de alınır. Parser/setup başarısı veya eşit metot adları semantik başarı diye kullanılmaz; expansion error, parse, cfg, unknown ve diagnostics kontrolleri korunur.

## Sürümlü API'nin sınırı

Altı birincil kaynak dosyası resmî commit `2ad81e72b7e978cb800dffe1bcce5c9f98032acf` içeriğiyle yayımlanmış 0.0.358 registry kaynağı arasında **6/6 byte eş** doğrulandı; kayıt `target/rust-source-mapping/primary/verified.json` içindedir. API'ler kararlılık garantisi olarak sunulmaz.

- [Semantics::original_range / original_range_opt](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/semantics.rs): opt, bütün node için uyumlu eşlemenin root context olmasını ister. None, tip/hedef bilinmiyor demek değildir. Rooted yol yalnız root span'ları kullanabilir; böylece proc örneğinde yalnız receiver girdisi kalır. Eşleme yoksa invocation'a döner; çağırana ayrıca “fallback oldu” biti vermez.
- [InFile eşleme yolları](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir-expand/src/files.rs) ve [map_node_range_up / rooted, MacroCallKind::original_call_range](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir-expand/src/lib.rs): nonroot yol span anchor/context uyuşmazlığında eşleyemez; rooted yol root span'ları süzer. Fn-like fallback genel olarak path ve `!` aralığıdır; girdi dahil tam invocation aralığı ayrı API'dir. Her dönen aralık kesin tanım kökeni değildir.
- [SpanMap::span_for_range](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir-expand/src/span_map.rs) expanded aralıkta başlangıç offset'inin span'ını alır; bütün expression'ın tüm token kökenlerini garanti etmez. Testte metot adı token'ı ve gerçek fiziksel token span'ı ayrı sorgulanır. `ExpansionInfo::map_range_down_exact` aynı span için bir iterator verir; **genel bire bir eşleme** sözü vermez. Tek sonuç bu tek include örneğinde assertion'dır.
- [Macro::source](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/has_source.rs) macro_rules bildirimi veya proc sağlayıcı Fn AST'sini verir. [SyntaxNodePtr](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/syntax/src/ptr.rs) syntax konumu pointer'ıdır; tek başına hangi expansion dosyasında olduğunu içermez. Bu yüzden HirFileId ile tutulur.

**Önerilen raporlama sınırı, uygulanmış politika değil:** seçili etkin gövdelerden gelen çağrılar yetkili semantik girdidir; fiziksel dosyaları ayrıca taramak yeni çağrı üretmek için yeterli değildir. Aynı expanded anahtarın tekrar ziyareti ayrılabilir. Ham/expanded anahtarlar otomatik eşit değildir; bu include'da token eşlemesi ilişkiyi kanıtladı. Çoklu include, iç içe makro, attribute/derive, tekrar üretilmiş token ve başka cfg/snapshot'lar için aynı eşdeğerlik kanıtlanmadı. Rapor yeri olarak açık invocation veya generated dosya bağı gösterilebilir; `original_range` tek başına kesin definition-origin diye etiketlenmemeli. İki gerçek çağrı aynı gösterim konumuna sahip olabilir. Genel traversal/dedup algoritması, kaynak şeması veya domain alanı eklenmedi.

## Gerçek koşular ve karşı kontroller

Yeni üç test doğru dependency davranışında yeşil başladı. İlk kurulumda MacroRules yerine `ast::Macro` ToDef girişinin gerektiği ve eksik yerel değişkenler nedeniyle derleme hataları düzeltildi; bunlar kırmızı davranış kanıtı değildir. Aşağıdaki tablo **ilk inceleme öncesindeki tarihsel koşulardır**; P2 düzeltmesinin yeni koşuları sonraki bölümde ayrıdır:

| Koşu / log | Sonuç |
|---|---|
| `regression-15.log` | **15/15**, exit **0**: expansion eski 4 + yeni 3, semantics 5, traits 3. |
| `locked-offline-repeat.log` | Aynı doğru kaynakla locked/offline tekrar **15/15**, exit **0**. |
| `check.log`, `clippy.log`, `fmt.log`, `artifact-fmt.log` | İlgili bütün probe hedeflerinde check/Clippy, probe fmt ve yeni Rust dosyası rustfmt **exit 0**; yeni lint izni yoktur. |
| `collapsed-key.log` | İki node için aynı ilk expanded node pointer'ı seçildi; `assert_ne!` aynı anahtarı yakaladı, **exit 101**. |
| `swapped-amount.log` | Gerçek çağrı sorgusu ters sıraya alındı; bağımsız beklenen 7 karşısında gerçek 11 assertion'ı, **exit 101**. |
| `wrong-proc-definition.log` | Proc kaynağı yerine consumer Wallet::charge kaynağı sorgulandı; consumer/provider dosya kimliği assertion'ı, **exit 101**. |
| `wrong-include-file.log` | Elle kurulan down-map anahtarında MacroFile yerine fiziksel FileId kullanıldı; aktif anahtar eşitliği assertion'ı, **exit 101**. |
| `restored-3.log` | Fault kopyasında doğru kaynak geri alınıp probe paketi yeniden derlendi; odaklı **3/3**, exit **0**. |

Dört fault da derlendi ve runtime assertion'da kırıldı; beklentiler değiştirilmedi. Kaynak hash'leri/komut/exit kayıtları `fault-results.json`, izole bozuk kaynaklar target altındadır. Bunlar ürün TDD'si veya Rust analiz raporu değildir. Kod/test okunabilirliği ayrı **salt okunur** Clean Code geçişinde incelendi; bulgu yok, bu geçiş kaynak değiştirmedi. Geniş core/workspace/Maven, önceki bütün runtime/fault paketleri ve upstream RA test/lint paketleri tekrar koşulmadı. Onaylı yamanın önceki kırmızı→yeşil zinciri tarihsel kanıttır; burada yeniden yapılmış sayılmaz.

## P2: API dosya kimliğinin doğrulanması

İlk test `mapped_tokens.file_id` alanını atıp anahtarı beklenen `include_id` ile kuruyordu. Bu nedenle önceki yanlış-file fault'u yalnız elle kurulan anahtarın yanlışlığını yakalıyordu; **API'nin yanlış dosya kimliğini fark ettiğinin kanıtı değildi**. Bağımsız incelemenin bu bulgusu dar biçimde düzeltildi.

Artık gerçek `mapped_tokens.file_id` önce `include_id` ile karşılaştırılır. Her eşlenen çağrı düğümünde `Semantics::hir_file_for(node)` bu gerçek dönüş kimliğiyle karşılaştırılır; `InFile<SyntaxNodePtr>` anahtarı da **API'den gelen** kimlikle kurulur. Aktif/ham anahtar farkı, tekrar sorgu, source/target/type ve strict diagnostic assertion'ları aynıdır. Yeni helper veya mapping/dedup framework eklenmedi.

Yeni karşı örnek yalnız yazarın fault kopyasında, aynı tokenlar/fixture/beklentilerle üretildi. `include_id.eager_arg(db)` gerçek modelden farklı ve geçerli bir MacroCallId verdi. Farklılığı ve bu ID'nin `Semantics::expand` sonucunun hatasızlığı denetlendi; sonra yalnız API sonucunun dosya alanı değiştirildi:

```rust
let alternate_id = include_id.eager_arg(&fixture.db).ok_or("real eager argument is missing")?;
assert_ne!(alternate_id, include_id);
let alternate_expansion = semantics.expand(alternate_id);
assert!(alternate_expansion.err.is_none(), "real eager argument expansion failed");
mapped_tokens.file_id = alternate_id;
```

Fault kopyasında `mapped_tokens` binding'i bu atama için `mut` yapılır; iterator/tokenlar değiştirilmez. Bu rastgele/geçersiz ID veya beklenti değişimi değildir. Yayımlanmış RA API'sinin bozuk davrandığı iddiası da değildir; probe'un dönen provenance bilgisini denetlemesi sınanır.

| Bu düzeltmede gerçekten koşulan / `target/rust-source-mapping/p2/` | Sonuç |
|---|---|
| `old-eager-id-green.log` | Eski test aynı yanlış API dosyasıyla **1/1, exit 0**: incelemenin kaçan karşı örneği yeniden üretildi. |
| `strengthened-eager-id-red.log` | Güçlendirilmiş test aynı fault ile **0/1, exit 101**; gerçek dosya kimliği assertion'ı farklı geçerli eager/include ID'lerini reddetti. Derleme/setup hatası yoktur. |
| `regression-15.log` | Doğru kaynakla eski 12 + mapping 3 **15/15, exit 0**; include düğümünün Semantics dosya kimliği de yeni denetimden geçti. |
| `check.log`, `clippy.log`, `fmt.log`, `artifact-fmt.log` | Probe tüm hedeflerde check/Clippy, probe fmt ve değişen Rust girdisinin rustfmt denetimi **exit 0**. |

Ayrı doğru/fault hedefleri korundu. Her karşı örnek sürümü ve doğru koşu öncesinde yalnız ilgili yazar hedefindeki probe paketi temizlenip yeniden derlendi. Debug=0/incremental=0, locked/offline ve mevcut source override'ları aynıdır. Fault kaynakları ve hash'leri `p2/old-with-eager-id.rs`, `p2/strengthened-with-eager-id.rs`, `p2/counterexample-results.json` içinde kayıtlıdır. Fault kaynağı doğru sürüme geri alındı. Yeni dosya-kimliği akışı için ayrı salt okunur test/kod okunabilirliği geçişi yapıldı; kaynak değiştirilmedi, bulgu yoktur.

Repo kökünden aynı exact test komutu önce eski, sonra güçlendirilmiş fault kaynağında kullanıldı; çıktılar yukarıdaki iki ayrı log'a alındı:

```sh
out="$PWD/target/rust-source-mapping"
fault_manifest="$out/fault-layout/examples/rust-backend-probe/Cargo.toml"
CARGO_TARGET_DIR="$out/fault" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RUSTFLAGS='-D warnings' cargo clean --manifest-path "$fault_manifest" -p rust-backend-probe
CARGO_TARGET_DIR="$out/fault" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RUSTFLAGS='-D warnings' cargo test --locked --offline --manifest-path "$fault_manifest" --test expansion source_mapping_cases::an_included_generated_call_has_an_identity_separate_from_its_physical_parse -- --exact --nocapture
```

Doğru kaynakla kullanılan exact komutlar:

```sh
probe="$out/layout/examples/rust-backend-probe"
probe_cargo() {
  env CARGO_TARGET_DIR="$out/correct" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RUSTFLAGS='-D warnings' cargo "$@"
}
probe_cargo clean --manifest-path "$probe/Cargo.toml" -p rust-backend-probe
probe_cargo test --locked --offline --manifest-path "$probe/Cargo.toml" --tests -- --test-threads=1 --nocapture
probe_cargo check --locked --offline --manifest-path "$probe/Cargo.toml" --all-targets
probe_cargo clippy --locked --offline --manifest-path "$probe/Cargo.toml" --all-targets -- -D warnings
cargo fmt --manifest-path "$probe/Cargo.toml" --all --check
rustfmt --edition 2024 --check examples/rust-backend-probe/experiments/source-mapping/source_mapping_cases.rs
```

Önceki dört fault, eski locked/offline tekrar, 3/3 restore, altı primary kaynak ve sayısal aralık gözlemleri bu düzeltmede yeniden koşulmuş gibi sunulmaz. Geniş veya upstream paketler koşulmadı.

Önceki incelemeden yalnız **source_mapping_cases.rs ve bu belge** değişti; include-tests.patch ve onaylı edition yaması değişmedi. Kanıt hâlâ tek include ve aynı DB/snapshot ile sınırlıdır; genel bire bir eşleme/dedup veya ürün desteği sonucu çıkarılmaz.

## Exact yeniden üretim komutları

Repo kökünden, temiz yazar hedefiyle. Aşağıdaki hazırlık onaylı kaynak kopyası varsa onu doğrular; yoksa [önceki belgedeki arşiv/checksum/yama adımları](rust-macro-edition-patch.md#exact-yeniden-üretim) aynı iki crate için uygulanır. Registry değiştirilmez.

```sh
out="$PWD/target/rust-source-mapping"
art="$PWD/examples/rust-backend-probe/experiments"
mkdir -p "$out/layout/examples"
cp -R "$PWD/target/rust-edition-patch/source-patched" "$out/source-patched"
cp -R examples/rust-backend-probe examples/rust-scenarios examples/rust-method-resolution examples/rust-visibility examples/rust-expansion "$out/layout/examples/"
probe="$out/layout/examples/rust-backend-probe"
(cd "$probe" && patch --batch --fuzz=0 -p1 -i "$art/edition-patch/probe-experiment.patch")
cat "$art/edition-patch/order_cases.rs" >> "$probe/fixtures/macro-edition/consumer/src/lib.rs"
(cd "$probe" && patch --batch --fuzz=0 -p1 -i "$art/source-mapping/include-tests.patch")
cat >> "$probe/Cargo.toml" <<EOF

[patch.crates-io]
ra_ap_hir_def = { version = "=0.0.358", path = "$out/source-patched/ra_ap_hir_def-0.0.358" }
ra_ap_hir_ty = { version = "=0.0.358", path = "$out/source-patched/ra_ap_hir_ty-0.0.358" }
EOF
CARGO_TARGET_DIR="$out/correct" cargo metadata --offline --format-version 1 --manifest-path "$probe/Cargo.toml" > "$out/metadata.json"
probe_cargo() {
  env CARGO_TARGET_DIR="$out/correct" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RUSTFLAGS='-D warnings' cargo "$@"
}
probe_cargo test --locked --offline --manifest-path "$probe/Cargo.toml" --test expansion source_mapping_cases -- --test-threads=1 --nocapture
probe_cargo test --locked --offline --manifest-path "$probe/Cargo.toml" --tests -- --test-threads=1 --nocapture
probe_cargo test --locked --offline --manifest-path "$probe/Cargo.toml" --tests -- --test-threads=1
probe_cargo check --locked --offline --manifest-path "$probe/Cargo.toml" --all-targets
probe_cargo clippy --locked --offline --manifest-path "$probe/Cargo.toml" --all-targets -- -D warnings
cargo fmt --manifest-path "$probe/Cargo.toml" --all --check
rustfmt --edition 2024 --check "$art/source-mapping/source_mapping_cases.rs"
```

Hash kontrolü: `upstream.json` içindeki yedi `file/after_sha256` alanı gerçek source-patched dosyalarıyla karşılaştırıldı; edition patch ve baseline 30 girdisi HEAD byte'larıyla eşleşti. Lock karşılaştırması **179 paket sürüm/dependency listesi aynı**, yalnız iki override'ın registry source/checksum alanları kalkmış gösterir. Metadata iki manifestin gerçekten bu yazar source-patched dizininde olduğunu, 27 RA paketinin 0.0.358 kaldığını doğrular; check/build log'ları bu iki paketin derlendiğini gösterir. Salsa/Unicode pinleri değişmez.

Cache yalnız yazarın `target/rust-edition-patch/patched` çıktısından APFS `cp -cR` ile alındı; taze tüm dependency derlemesi iddiası yoktur. Yeni `correct` ve ayrı `fault` target'larında debug=0/incremental=0 kullanıldı. Kopyalama sonrası yalnız bu hedeflerde `analyzed*` dizinleri kaldırıldı, başka target temizlenmedi. Build/proc çıktıları bu hedeflerin altındadır; aynı kaynakta Cargo cache kullanımı her komutun build.rs'yi yeniden çalıştırdığı anlamına gelmez.

Fault'lar ayrı `fault-layout` gerçek kopyalarında, aynı override ve beklentilerle yapıldı. Her kaynak değişiminde yalnız kendi probe paketi `cargo clean -p rust-backend-probe` ile temizlendi; stale test binary kabul edilmedi. Exact fault komutu aşağıda ilk test içindir; diğer iki test adı tablodaki kaynak dosyada bulunur:

```sh
fault_manifest="$out/fault-layout/examples/rust-backend-probe/Cargo.toml"
CARGO_TARGET_DIR="$out/fault" cargo clean --manifest-path "$fault_manifest" -p rust-backend-probe
CARGO_TARGET_DIR="$out/fault" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RUSTFLAGS='-D warnings' cargo test --locked --offline --manifest-path "$fault_manifest" --test expansion source_mapping_cases::one_declarative_invocation_keeps_two_distinct_semantic_charge_sites -- --exact --nocapture
```

## Teslim sınırı

`target/rust-source-mapping/SHA256SUMS` bu belgeyi, iki yeni deney girdisini ve önceki 30 probe girdisini kapsar. Dış 20 fixture `REUSED-SHA256SUMS` içindedir; hash'i **57982e7460057385173291b93766c2a2a8ee7cd3ae3549f63bbad657f988fc3a** ile değişmedi. Vendor/build ağaçları teslim girdisi değildir. Yeni test bağlama patch'i onaylı strict probe kopyasına uygulanır; dağıtılan probe'a kalıcı override veya otomatik test bağlantısı eklenmedi.

Genel kaynak/rapor politikası ve dedup algoritması, active unresolved/inactive cfg negatifi, nihai backend seçimi ve ürün SourceReader/TypeSource/CheckProject/LoD uygulaması açık kalır. Yamasız mevcut probe'un bilinen edition kabulü tarihsel olarak hâlâ başarısızdır; bu dilimde yeniden koşulmadı veya başarıya çevrilmedi.
