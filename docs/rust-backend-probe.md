# Rust backend deneyi: temel çağrılar

**RA 0.0.358, aşağıdaki uyumlu kilitle Rust 1.99.1 üzerinde derlendi; S2/S5 tip ve metot kimlikleri gerçek Cargo snapshot'ından okundu.** Üç öğrenme testi geçti. Bu yalnız [altyapı karşılaştırmasının](rust-backends.md) ilk dilimidir: nihai backend/sürüm seçimi, genel deney satırı ve Rust ürün desteği tamamlanmadı.

Taban `17a1ec9`, tarih 23 Kasım 2026. Yeni girdiler yalnız bu belge ve [`rust-backend-probe`](../examples/rust-backend-probe/Cargo.toml) projesidir. Üretim adaptörü, `TypeSource`, `SourceReader`, `CheckProject`, LoD raporu veya sınıflandırma etiketi uygulanmadı. Beklentiler [onaylanmış kataloğun](rust-scenarios.md) S2/S5 kaynaklarıdır; eski fixture değiştirilmedi veya kopyalanarak yeni ürün testi yapılmadı.

## Kurulum ve gerçek sürüm engelleri

Host `aarch64-apple-darwin`; Rust/Cargo **1.99.1**, rustc commit'i `58cf8f9c35b355199727627d07ae34a4e2846d67`. Başlangıçta std kaynakları yoktu. Yetkilendirilen tek kurulum `rustup component add rust-src --toolchain stable-aarch64-apple-darwin` oldu. Önce/sonra `rustc --version --verbose` çıktıları `cmp` ile aynı bulundu. Compiler, varsayılan toolchain veya global config değiştirilmedi; nightly/rustc-dev/RA bileşeni kurulmadı.

Sekiz doğrudan test bağımlılığı **`=0.0.358`**: `ra_ap_cfg`, `ra_ap_hir`, `ra_ap_ide_db`, `ra_ap_intern`, `ra_ap_load-cargo`, `ra_ap_project_model`, `ra_ap_syntax`, `ra_ap_vfs`. Gerçek locked/offline Cargo metadata'sında **179 paket** (probe dahil), **27 `ra_ap_*` paketi** vardır; 27'sinin sürümü de 0.0.358'dir. Ayrı `ra-ap-rustc_*` ayna paketleri 0.166.0 ailesindedir; başka bir `ra_ap` kohortu karıştırılmadı. Tam geçişli sürümler/checksum'lar [Cargo.lock](../examples/rust-backend-probe/Cargo.lock)'tadır.

Metadata/MSRV yeterli olmadı; ilk build'lerde görülen somut sorunlar:

| Gözlem | Uygulanan dar düzeltme / sonuç |
|---|---|
| Lexer `E0080`: unicode-ident 1.0.24 **Unicode 17**, unicode-properties 0.1.5 **Unicode 18**; lexer eşitlik kontrolü derlemeyi durdurdu. | Kilitte unicode-properties **0.1.4** seçildi; bu sürüm Unicode 17 kullanır ve lexer'ın `^0.1.4` koşulunu karşılar. Kaynaklar [0.1.5](https://docs.rs/crate/unicode-properties/0.1.5/source/src/tables.rs), [0.1.4](https://docs.rs/crate/unicode-properties/0.1.4/source/src/tables.rs), [unicode-ident 1.0.24](https://docs.rs/crate/unicode-ident/1.0.24/source/src/tables.rs). |
| Salsa 0.28.6 ile RA variance sorgusunun cycle-initial makrosu `E0061` verdi. Yalnız salsa'yı 0.28.5 yapmak, kalan 0.28.6 makrolarıyla `E0326` verdi. | `salsa`, `salsa-macros`, `salsa-macro-rules` birlikte **0.28.5** kilitlendi. Yayımlanmış RA kaynakları değiştirilmedi; bundan sonra native bağımlılık build'i geçti. |
| Konum sorgusunun `EditionedFileId` değeri doğrudan VFS `FileId` yerine verilince `E0308`. | Aynı DB üzerinden `.file_id(&db)` kullanıldı. Bu compiler hatası öğrenme assertion'ı değildir. |
| İlk tanı sorgusu `Tried to use attached db, but no db is attached` paniği verdi; S2/S5 sorguları zaten geçmişti. Backtrace kayıtlıdır. | Tanı toplama, HIR'ın public re-export ettiği `attach_db(&db, ...)` içinde çalıştırıldı; üç test sonra geçti. Bu sürümdeki API kullanım koşuludur, yakalanıp gizlenmiş hata değildir. |

Bunlar kurulum/API edinimi sırasında görülen hatalardır; eksik ürün davranışının kırmızı TDD kanıtı sayılmaz. Ayrıca tam offline metadata sorgusu ilk seferde henüz alınmamış başka-platform `inotify` paketini istedi. `cargo fetch --locked` kalan kaynakları aldı; sonraki tam metadata ve taze native build offline geçti. Lock'u silerek yeniden çözmek bu uyumlu geçişli seçimleri korumaz.

## Yüklenen bağlam

Testler mevcut [`examples/rust-scenarios/Cargo.toml`](../examples/rust-scenarios/Cargo.toml) projesini `load_workspace_at` ile açar. Her test kendi tek DB/VFS bağlamını kullanır; bütün beklenen bildirimler ve gerçek çağrı sorguları o bağlamdadır. SourceReader veya sahte tip kataloğu yoktur.

- Analiz edilen hedef normal **edition 2024 lib**; `set_test=false`, default feature seçimi ve mevcut host target'tır. Probe'un kendisinin Cargo test hedefi olması, analiz edilen lib'e `cfg(test)` eklemez.
- `rustc --print sysroot` sonucu ve onun `lib/rustlib/src/rust/library` yolu **açıkça** `CargoConfig`'e verilir. Kaynak yoksa test hata döndürür; otomatik rust-src keşif/kurulum yoluna güvenilmez.
- `CfgOverrides` ile yükleyicinin eklediği `rust_analyzer` flag'i kaldırılır. Sonuç yalnız config nesnesinden değil, gerçek `Crate::cfg` ve etkin modüllerden denetlenir.
- `load_out_dirs_from_check=false`, `ProcMacroServerChoice::None`, cache ön doldurma kapalıdır. Bu macro-free normal fixture için analiz edilen projenin build.rs/proc kodu çalıştırılmadı; Cargo metadata ve rustc bağlam sorguları çalıştı. Probe bağımlılıklarının yayımlanmış build/proc kodu derleme sırasında çalışabilir ve bu edinimin kapsamındadır.
- Metadata `--locked --offline`; hedef dizini mutlak `CARGO_TARGET_DIR/analyzed` olarak verilir. Test çıktıları worktree'nin `target/rust-backend-probe` altındadır; canonical checkout target'ı kullanılmaz.

Kaynak/edition ilişkisi `Semantics::parse(EditionedFileId::new(...))` ile korunur. Bildirim seçiciler fixture'daki `Wallet`, `Order`, `PaymentService` adlarını kaynak çapası olarak kullanır; bunlar genel Rust çözümleyicisi değildir. Beklenen `Struct`/`Function` kimlikleri bu bildirimlerdeki `Semantics::to_def` sonuçlarıdır. Gerçek alıcı/dönüş/metot için `type_of_expr` ve `resolve_method_call` kullanılır; beklenen tip modele enjekte edilmez, display/hover/debug metni ayrıştırılmaz. API'ler [0.0.358 HIR kaynağındadır](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/semantics.rs).

## Doğrulanan sorgular

[`tests/semantics.rs`](../examples/rust-backend-probe/tests/semantics.rs) üç doğrudan dependency öğrenme testi içerir:

| Test / gerçek gözlem | Bağımsız denetim ve sınır |
|---|---|
| `the_parameter_call_resolves_to_the_wallet_declaration`: S2 `wallet.debit` | Alıcı tipi unknown içermez; shared referansın ADT kimliği kaynak Wallet bildirimiyle aynıdır. Hedef, Wallet impl'indeki debit bildiriminin Function kimliğidir. VFS kaynak yolu ve **satır 13** doğrulanır. |
| `the_returned_chain_resolves_the_order_method_and_wallet_result`: S5 `order.wallet().debit` | `wallet` hedefi Order::wallet, `debit` hedefi Wallet::debit kimliğidir. İç çağrı dış çağrının gerçek receiver AST'sidir. İç çağrının sonuç tipi unknown içermeyen **`&Wallet`** olarak çözülür; iki çağrı **satır 29** ve aynı gerçek dosyaya bağlıdır. |
| `the_normal_library_excludes_test_items_without_removing_their_source` | Gerçek crate edition'ı 2024; `test` ve `rust_analyzer` flag'leri kapalı. Ham AST hâlâ `tests` modülünü içerir, etkin root modülünde child yoktur. `Module::diagnostics(..., false)` çıktısı **tek `InactiveCode`** içerir; başka tanı yoktur. Bu API çıktısı compiler'ın bütün tanıları diye sunulmaz. |

Yükleme/rustc/dosya okuma hataları döndürülür; parser hataları assertion ile görünür olur. Eksik bildirim/hedef/tip/original range başarısızlıktır; unknown tip uygun/literal/yeni nesne yapılmaz. Bu pozitif sorgular, bütün std gövdelerinin çözüldüğünü veya gelecekteki tüm eksik model durumlarının kapsandığını kanıtlamaz. Etkin çözülemeyen çağrı negatifi sonraki gate'tir. Özellikle proc desteği bilerek kapalıdır; proc uyumu veya makro tamlığı çıkarılmaz.

## Çalıştırılan komutlar ve sonuçlar

Repo kökünden; kök workspace testleri çalıştırılmadı. İlk edinim `CARGO_TARGET_DIR="$PWD/target/rust-backend-probe/cargo" cargo test --manifest-path examples/rust-backend-probe/Cargo.toml --no-run` idi; aşağıdaki kilit düzeltmelerinden sonra gerçek testler çalıştırıldı:

```sh
rustup component add rust-src --toolchain stable-aarch64-apple-darwin
CARGO_TARGET_DIR="$PWD/target/rust-backend-probe/cargo" cargo update --manifest-path examples/rust-backend-probe/Cargo.toml -p unicode-properties --precise 0.1.4
CARGO_TARGET_DIR="$PWD/target/rust-backend-probe/cargo" cargo update --manifest-path examples/rust-backend-probe/Cargo.toml -p salsa --precise 0.28.5
CARGO_TARGET_DIR="$PWD/target/rust-backend-probe/cargo" cargo update --manifest-path examples/rust-backend-probe/Cargo.toml -p salsa-macros --precise 0.28.5
CARGO_TARGET_DIR="$PWD/target/rust-backend-probe/cargo" cargo update --manifest-path examples/rust-backend-probe/Cargo.toml -p salsa-macro-rules --precise 0.28.5
CARGO_TARGET_DIR="$PWD/target/rust-backend-probe/cargo" cargo fetch --locked --manifest-path examples/rust-backend-probe/Cargo.toml
CARGO_TARGET_DIR="$PWD/target/rust-backend-probe/cargo" cargo metadata --locked --offline --format-version 1 --manifest-path examples/rust-backend-probe/Cargo.toml
CARGO_TARGET_DIR="$PWD/target/rust-backend-probe/cargo" cargo test --locked --offline --manifest-path examples/rust-backend-probe/Cargo.toml --test semantics -- --test-threads=1
CARGO_TARGET_DIR="$PWD/target/rust-backend-probe/offline" RUSTFLAGS='-D warnings' cargo test --locked --offline --manifest-path examples/rust-backend-probe/Cargo.toml --test semantics -- --test-threads=1
CARGO_TARGET_DIR="$PWD/target/rust-backend-probe/offline" RUSTFLAGS='-D warnings' cargo clippy --locked --offline --manifest-path examples/rust-backend-probe/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path examples/rust-backend-probe/Cargo.toml --check
```

Tekrar için mevcut lock ile fetch/test komutları yeterlidir; update satırları yalnız edinimdeki düzeltmenin kaydıdır. İlk düzeltilmiş koşuda **3/3**, edinim sonrası önceden bulunmayan `offline` target dizininde yeniden derlenip **3/3** geçti; skipped test yoktur. Clippy `-D warnings` ve fmt check geçti; lint istisnası yoktur. Bunlar üç farklı testtir, altı farklı senaryo değildir. Diğer gate'ler, Java/core/workspace/Maven ve eski runtime fixture testleri yeniden çalıştırılmadı. Süre/RAM karşılaştırması yapılmadı.

### İlk yeşil assertion'ların hata yakalaması

Geçici kopya `target/rust-backend-probe/fault-layout/examples/rust-backend-probe` altındadır; aynı seviyedeki `rust-scenarios` symlink'i değişmeyen gerçek fixture'a gider. Probe manifest/lock/testi kopyalandı; yalnız aşağıdaki query/config satırları birer kez değiştirildi, beklentiler ve fixture aynı kaldı. Her varyant ayrı exact testte **derlendi ve runtime assertion ile çıkış 101** verdi:

| Geçici bozulma | Yakalanan assertion |
|---|---|
| S2'nin gerçek target sorgusundaki `&call` yerine `on_order_parameter` içindeki `debit` çağrısını sorgula. | Aynı adlı Order::debit, beklenen Wallet::debit Function kimliğine eşit değildir. Alıcı tipi sorgusu değiştirilmedi. |
| S5'in `projection_target` sorgusunda `&projection` yerine `&debit` kullan. | Wallet::debit, beklenen Order::wallet kimliği değildir. |
| `global: CfgDiff::new(Vec::new(), vec![flag("rust_analyzer")])` yerine `global: CfgDiff::default()` kullan. | Gerçek modelde `rust_analyzer` flag'inin kapalı olması assertion'ı kırılır. |

Her varyant için komut; `test_name` yukarıdaki ilgili tam test adıdır:

```sh
CARGO_TARGET_DIR="$PWD/target/rust-backend-probe/cargo" cargo test --locked --offline --manifest-path target/rust-backend-probe/fault-layout/examples/rust-backend-probe/Cargo.toml --test semantics "$test_name" -- --exact
```

Geçici kaynaklar/loglar `target/rust-backend-probe/{parameter-target,chain-target,cfg-override}.{rs,log}` altındadır. Kopyanın doğru kaynağı geri kondu ve aynı komut filtresiz `-- --test-threads=1` ile çalıştırıldı: **3/3 geçti**. Compiler/setup hataları bu üç assertion bozulmasının kanıtı değildir; önceki bölümde ayrı kaydedildi. Gerçek sahip olunan test kaynağı doğru kaldı.

Test okunabilirliği ayrı, salt okunur geçişte incelendi; bulgu yok. İsimler assertion'larla uyumlu; kaynak çapaları, bağımsız kimlik beklentileri ve 13/29 satırları görünürdür. Kur/işlet/denetle blokları boş satırla ayrıdır; helper'lar yalnız yükleme/çapa seçimi ayrıntılarını taşır. İnceleme kaynak değiştirmedi.

## Girdi kimliği ve kalan kapsam

Yeniden kullanılan üç girdinin SHA-256 değerleri ayrıca `target/rust-backend-probe/REUSED-SHA256SUMS` içindedir; teslim manifestine katılmaz:

| Önceden onaylı girdi | SHA-256 |
|---|---|
| `examples/rust-scenarios/Cargo.toml` | `f71e31b7dd2a3ab99b00e2f214a79db7a0a551277719cc59aec4514b32c0fa3f` |
| `examples/rust-scenarios/Cargo.lock` | `c2ec775831b47a2f06c6cc124d7f6da709ce6efe26891437dc4ac0e406398160` |
| `examples/rust-scenarios/core.rs` | `26d4efdfb74120a81822feb55a19d1e5fd7acd6bcda75e5e101270c770029f70` |

Gerçek çok-crate kimliği, generic/ilişkili tip/trait/dyn hedefleri, macro/build/proc ABI ve expansion kaynak eşlemesi, etkin çözülemeyen çağrı negatifi **çalıştırılmadı**. Sonraki yolculuk bunlardan yalnız küçük bir dilim seçmelidir. LoD köken eşlemesi, Value/Hybrid, kaçan değişim yetkisi, getter/factory/fluent/kapsayıcı politikaları ve eksik analiz raporu/çıkış kararı açık kalır. Bu dilimde bağımlılığı başarıyla sorgulamak, Rust analiz adaptörü veya ürün raporu uygulanmış demek değildir.
