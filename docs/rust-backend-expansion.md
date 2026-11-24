# Rust backend deneyi: build bilgisi ve proc yürütme

**RA 0.0.358, iki seçili build bağlamını ve mevcut Rust 1.99.1 sunucusuyla gerçek function-like proc açılımını sorgulayabildi.** Bu, [expansion kapısının](rust-backends.md#sonraki-sınırlı-öğrenme-deneyi) yalnız dar bir dilimidir. Test cfg'sindeki başka bir karışık-edition makroda RA tip uyuşmazlığı da gözlendi; tam dosya tanısız değildir. Genel deney ve nihai backend seçimi açık kalır.

Taban `5f3573ae47c0914be4deb1d62e7807dad40bb8bc`, tarih 23 Kasım 2026. TypeSource/SourceReader, CheckProject, LoD/Value/Hybrid veya CLI hata politikası uygulanmadı.

## Gerçek girdiler ve bağlamlar

Onaylı [`rust-expansion` workspace'i](../examples/rust-expansion/Cargo.toml) değiştirilmeden yüklendi: legacy edition 2018, proc provider 2021, consumer 2024. Dış bağımlılığı olmayan üç yerel crate, kendi lockfile'ı ve gerçek build.rs kullanıldı. Kaynak kopyalama, elle makro açma, tip enjekte etme veya ikinci parser yoktur.

| Analiz snapshot'ı | Seçili girdiler | Bağımsız beklenti ve gerçek gözlem |
|---|---|---|
| Normal/default lib | `set_test=false`, default feature'lar, mode `base` | Tek etkin selected_value gövdesi **17**; build cfg `build_bonus=false`; generated_charge ve OUT_DIR/generated.rs içindeki çağrı argümanı **7**. |
| Normal/alternate lib | `set_test=false`, `alternate`, mode `bonus` | Tek etkin selected_value gövdesi **23**; `build_bonus=true`; iki kaynak görünümündeki argüman **11**. |
| Proc çağrısının test bağlamı | `set_test=true`, default feature'lar, mode `base` | Gerçek Crate cfg'sinde test açık; seçilen consumer fonksiyonu HIR'da is_test. Function-like generated_charge!(wallet) açılımında **charge(13)** ve **balance()**. |
| Başarısız build | Mode `invalid` | WorkspaceBuildScripts::error dolu; `LOD_FIXTURE_MODE must be base or bonus` açık Result hatasına taşınır. |

Her bağlamda `rust_analyzer` cfg override ile kaldırılır. Host/varsayılan Cargo hedefi `aarch64-apple-darwin`; cross hedef denenmedi. Yeni helper global süreç ortamını değiştirmez: mode, bozuk-kaynak değişkeninin kaldırılması, feature'lar ve mutlak hedef dizini CargoConfig üzerinden verilir. Önceki beş normal-lib ve üç test-cfg sorgusu kendi eski bağlamlarıyla korunur.

## Build çıktısının modele aktarılması

[`tests/expansion.rs`](../examples/rust-backend-probe/tests/expansion.rs) şu gerçek sırayı kullanır: ProjectWorkspace::load → metadata/sysroot hata kontrolü → run_build_scripts → **error() kontrolü** → set_build_scripts → load_workspace. İkinci kez build çalıştırması istenmez. `load_workspace_at` kolaylık yolu aynı build hatasını yalnız loglayıp DB döndürebildiğinden yeni yürütülen yüklemede kullanılmaz. Eski build/proc kapalı loader değişmedi.

OUT_DIR, gerçek consumer crate'inin DB ortamından `consumer.base().env(db)` ile alınır. İlgili generated.rs VFS kimliğiyle, consumer'ın edition bağlamında parse edilir; parse hatası denetlenir. Etkin generated_charge HIR bildirimi consumer modülüne/crate'ine aittir. Bu bildirimin Semantics kaynağındaki gerçek çağrı hedefi, aynı DB'deki bağımsız **Wallet::charge** bildirimiyle eşittir; alıcının referent ADT'si gerçek Wallet'tır ve unknown içermez. Fiziksel generated dosyanın ve include üzerinden etkin gövdenin 7/11 argümanları ayrı sorgulanır.

Her snapshot'ın hedefi `target/rust-backend-expansion/cargo/analyzed-<mode>-<alternate>-<test>` altındadır. DB'nin OUT_DIR yolu bu seçili hedef altında olmalı; gerçek build-runs.log son modu doğrular. Base ve bonus ayrı hedef/DB'lerdir. Bu, eski dosyayı yeni bağlam diye kullanmaya karşı dar kontroldür; tam cache invalidation kanıtı değildir. Tekrar koşularında Cargo önbelleği kullanılır; build çıktısı olayının görünmesi her seferinde script'in yeniden çalıştığı iddiası değildir.

RA'nın sürümlü [run_build_scripts](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/project-model/src/workspace.rs) ve [build komutu](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/project-model/src/build_dependencies.rs) incelendi. Bu toolchain için kütüphane Cargo check komutuna `--compile-time-deps --all-targets --keep-going -Zunstable-options` ekler ve yalnız alt komut ortamında `__CARGO_TEST_CHANNEL_OVERRIDE_DO_NOT_USE_THIS=nightly` ayarlar. Compiler/default toolchain veya global yapılandırma değiştirilmez. Bu yol ana crate'in genel tip denetimi yerine compile-time bağımlılıkları hazırlar; ayrıca normal `cargo check --lib` koşuları yapıldı.

## Gerçek proc sunucusu ve sorgu sınırı

Explicit sysroot `/Users/deep/.rustup/toolchains/stable-aarch64-apple-darwin`; eşleşen kaynak `lib/rustlib/src/rust/library`; sunucu aynı sysroot'un **libexec/rust-analyzer-proc-macro-srv** dosyasıdır. Yeni toolchain/bileşen kurulmadı. Rustc önce/sonra kimliği **1.99.1**, commit `58cf8f9c35b355199727627d07ae34a4e2846d67`; Cargo 1.99.1, rustfmt 1.10.0, Clippy 0.1.99.

Explicit sunucu yolu gerçek loader'a verilir; dönen ProcMacroClient DB/VFS sorguları bitene kadar tutulur ve en son bırakılır. Loader istemciyi döndürmezse doğrudan ProcMacroClient::spawn yalnız somut başlangıç hatasını göstermek için denenir; başarılı bir alternatif istemciyle eksik loader çıktısı örtülmez. Sunucu/protokol sürümü taklit edilmez.

Semantics::resolve_macro_call bildirimin proc macro olduğunu gösterir. `to_def(MacroCall)` ve **expand(MacroCallId)** gerçek açılımı verir; ExpandResult.err ayrıca kontrol edilir. Açılım AST'sindeki iki farklı metot çağrısının hedefleri kaynak Wallet::charge / Wallet::balance Function kimliklerine eşittir. İki gerçek alıcı tipi Wallet ADT'sidir ve unknown değildir; balance sonucu bağımsız balance bildiriminin dönüş tipiyle eşittir. Argüman RA'nın AST integer API'sinden **13** okunur. Adlar yalnız kaynak çapasıdır; display/debug/hover/MIR metni semantik protokol yapılmaz.

Bu sonuç yalnız dosya varlığından/Some(client)'tan fazladır: yayımlanmış RA istemcisi, mevcut sunucu ve aynı compiler ile üretilmiş yerel proc kütüphanesi gerçek token üretip çözülmüş çağrılar verdi. Yalnız **function-like** macro/proc ABI uyumu bu fixture'da doğrulandı; attribute/derive ve başka compiler/sunucu sürümleri kapsanmaz. Consumer'ın hedef runtime testi ayrıca charge etkisini **37 bakiye / 1 ödeme** olarak doğruladı.

**Açık legacy tanısı:** `an_exported_macro_keeps_the_method_tokens_edition` içindeki definition_items! çağrısında RA hâlâ slice Iter yerine array IntoIter bildirir. İlk geniş assertion bunu yakaladı (`first-expansion.log`, 3 geçti/1 kaldı). Ardından bütün non-InactiveCode tanıları yalnız loglayan ilk çözüm yetersizdi: bağımsız inceleme, seçili proc açılımındaki i32/u32 hatasının da kabul edildiğini gösterdi. Bu tarihsel yanlış yeşil aşağıdaki P2 karşı örneğiyle yeniden üretildi; başarı kanıtı değildir.

**Son tanı sınırı:** assert_proc_context_diagnostics, aynı DB'de legacy invocation'ın gerçek MacroCallId'sini çözer/açar. Açılımın tek into_iter düğümünün `HirFileId::MacroFile` kimliği ve SyntaxNodePtr değeri alınır. Yalnız **TypeMismatch + aynı expansion dosyası + aynı AST düğümü** eşleşmesi bekleyen legacy kusuru olarak loglanır. Metin/satır, bütün fonksiyon/dosya veya genel TypeMismatch muafiyeti yoktur. Consumer kök modülünün tanıları, bu fixture'ın inline test modülü dahil toplanır; InactiveCode ve bu tek izin dışında hepsi boş olmalıdır. Seçili proc hatası da assertion'ı kırar. Bu dar istisna legacy edition kusurunu çözmez; genel expansion kapısı açık kalır.

API kaynakları sürüm commit'ine sabitlendi: [loader](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/load-cargo/src/lib.rs), [proc istemcisi](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/proc-macro-api/src/lib.rs), [protokol anlaşması](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/proc-macro-api/src/process.rs), [Semantics](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/semantics.rs), [HIR](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/lib.rs), [DB girdileri](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/base-db/src/input.rs). İlk teslimde bu sekiz dosya, yukarıdaki iki project-model dosyası dahil, yayımlanmış 0.0.358 kaynaklarıyla byte olarak eşleşti. Public API kullanımı kararlılık garantisi değildir.

## İlk teslimin hata ve ilk-yeşil denetimleri (tarihsel)

Yeni hata yolu önce build.error() denetimi olmadan koştu: gerçek build script exit 1 verdi, fakat helper Ok kabul ettiği için **a failed fixture build was accepted** assertion'ı kırıldı. En küçük error()→Err aktarımıyla aynı test geçti. Ham build hatası loglanır. Bu, öğrenme projesinin Result davranışıdır; ürün CLI çıkış kararı değildir. Derleme/API uyarlama hataları ve disk dolması kırmızı TDD kanıtı sayılmadı.

Geçici kaynak kopyasında, değişmeyen beklentilerle dokuz bozulma derlendi ve assertion ile yakalandı:

| Bozulma | Bağımsız kontrol |
|---|---|
| Build hata kontrolünü kaldır | Err beklenirken Ok kabulü. |
| Build çıktılarını boş çıktıyla değiştir | DB'de OUT_DIR yokluğu. |
| Bonus snapshot'ta alternate'ı kapat | 17 ≠ 23. |
| Base snapshot'a bonus build modu ver | 11 ≠ 7. |
| Generated charge yerine balance bildirimini sorgula | Farklı Function kimliği. |
| Proc charge hedefi yerine balance çağrısını sorgula | Farklı Function kimliği. |
| Proc argümanı yerine kaynak constructor argümanını sorgula | 50 ≠ 13. |
| Proc alıcısı yerine çağrının sonuç tipini sorgula | Sonuç tipi Wallet ADT'si değildir. |
| Proc snapshot'ında test cfg'sini kapat | Gerçek modelin test cfg assertion'ı. |

Kopyalar/komutlar `target/rust-backend-expansion/faults`, `fault-cases.json`, `fault-results.json` altında; dokuzunda da exit 101, runtime panic ve 0 geçti/1 kaldı gözlendi. Beklenen değer değiştirilmedi; compiler/setup hatası yakalanmış assertion sayılmadı. Asıl kaynaklar doğru kaldı.

Kopya ile asıl projenin aynı ad/sürüm için Cargo cache'i paylaşması, ilk geri dönüşte son bozuk test binary'sinin yeniden kullanılmasına yol açtı. Bu sonuç başarı sayılmadı. Yalnız probe paketinin kendi target çıktıları `cargo clean -p rust-backend-probe` ile kaldırılıp gerçek proje yolu için yeniden derleme doğrulandı. Geçici kopya doğrulaması ardından doğru kaynakla paket temizliği/rebuild yapılmadan cache sonucu güvenilir sayılmamalıdır.

## Değişen girdiler ve kurulum maliyeti

Başlangıçtan değişen dosyalar: bu yeni belge; probe Cargo.toml/Cargo.lock; tests/support/mod.rs; yeni tests/support/anchors.rs ve tests/expansion.rs. Eski semantics.rs ve traits.rs byte olarak değişmedi. Dört AST çapa fonksiyonu gövdeleri değişmeden küçük anchors modülüne taşındı; test içi görünürlük dışında loader davranışı değişmedi. Yeni build/proc yolu ayrı dosyada kalır; genel harness/CLI/rapor yoktur.

Gerekli yeni doğrudan dependency `ra_ap_proc_macro_api = "=0.0.358"`; zaten transitive kilitliydi. Dokuz doğrudan RA pini, aynı sürümde 27 RA paketi ve toplam 179 paket korunur. Cargo.lock farkı yalnız root dependency listesine bu girdinin eklenmesidir; Salsa 0.28.5, unicode-properties 0.1.4, unicode-ident 1.0.24 dahil hiçbir transitive seçim yenilenmedi.

İlk teslimde önceki target cache'leri kullanılmadı/kopyalanmadı; registry cache'i kullanıldı. Disk dolunca yalnız bu dilimin yeniden üretilebilir derleme çıktıları temizlendi; debug bilgisi ve incremental kapatılarak aynı dilimin cache'i yeniden kullanıldı. İlk hata-yolu kırmızı/yeşil koşuları varsayılan debug profiliyle, sonraki koşular aşağıdaki profille yapıldı. Tekrarlar taze tüm dependency build'i diye sunulmaz.

Çalıştırılan proje kodu bu repo için yazılmış local build.rs, proc provider ve seçilen consumer runtime testidir. Build script kendi iki girdisini okur, OUT_DIR'a generated.rs/build-runs.log yazar. Cargo/proc derleme çıktıları yalnız bu dilimin target altındadır. Probe derlemesindeki normal yayımlanmış dependency build adımları da çalışır; başka kullanıcı projesi yüklenmez.

## İlk teslimin doğrulaması (tarihsel)

İlk inceleme öncesindeki kaynakla **iki locked/offline koşunun her birinde 12/12 test** geçti: önceki 8 ve yeni 4; atlanan yok. Probe check ve Clippy tüm hedeflerde, probe/expansion workspace fmt denetimleri geçti; yeni lint izni yoktur. Clippy'nin istediği iç içe koşul sadeleştirmesinden sonra iki tam koşu yapıldı. Kaynak fixture için base ve bonus/alternate **iki lib check**, ayrıca **bir proc runtime testi** geçti (diğer beş runtime testi filtrelendi). Bunlar eski bütün runtime paketlerinin yeniden koşusu değildir; geniş core/workspace/Maven çalıştırılmadı.

Test okunabilirliği ayrı salt okunur geçişte ve koşul sadeleştirmesi ardından tekrar incelendi; bulgu yok. Adlar davranışla uyumlu, kur/işlet/denetle araları boşluklu, 17/23–7/11–13 beklentileri bağımsız ve görünürdür. Kaynak kimlikleri ile sorgulanan hedefler ayrıdır; negatifte önce Err doğrulanır. Test cfg'si ve tanı kapsamı açık kalır. İnceleme kaynak değiştirmedi.

İlk teslimde repo kökünde kullanılan ortak ayarlar ve komutlar:

```sh
probe_manifest=examples/rust-backend-probe/Cargo.toml
fixture_manifest=examples/rust-expansion/Cargo.toml
probe_target="$PWD/target/rust-backend-expansion/cargo"
probe_cargo() {
  env CARGO_TARGET_DIR="$probe_target" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RUSTFLAGS='-D warnings' cargo "$@"
}
fixture_cargo() {
  env CARGO_TARGET_DIR="$PWD/target/rust-backend-expansion/source" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RUSTFLAGS='-D warnings' cargo "$@"
}
probe_cargo clean --manifest-path "$probe_manifest" -p rust-backend-probe
probe_cargo test --locked --offline --manifest-path "$probe_manifest" --tests -- --test-threads=1 --nocapture
probe_cargo test --locked --offline --manifest-path "$probe_manifest" --tests -- --test-threads=1 --nocapture
probe_cargo check --locked --offline --manifest-path "$probe_manifest" --all-targets
probe_cargo clippy --locked --offline --manifest-path "$probe_manifest" --all-targets -- -D warnings
LOD_FIXTURE_MODE=base fixture_cargo check --locked --offline --manifest-path "$fixture_manifest" -p expansion-consumer --lib
LOD_FIXTURE_MODE=bonus fixture_cargo check --locked --offline --manifest-path "$fixture_manifest" -p expansion-consumer --lib --features alternate
LOD_FIXTURE_MODE=base fixture_cargo test --locked --offline --manifest-path "$fixture_manifest" -p expansion-consumer --lib -- --exact tests::a_function_like_proc_macro_generates_a_charge_in_the_consumer
cargo fmt --manifest-path "$probe_manifest" -- --check
cargo fmt --manifest-path "$fixture_manifest" --all -- --check
```

Her fault için aynı probe_cargo ayarlarıyla `cargo test --locked --offline --manifest-path target/rust-backend-expansion/fault-layout/examples/rust-backend-probe/Cargo.toml --test expansion <test_name> -- --exact` kullanıldı. Yukarıdaki package clean ardından doğru kaynak koşusu gelir. Profil farkları dışında hedefli ilk hata-yolu komutu aynı test dosyasında `a_failed_fixture_build_is_returned_as_an_error -- --exact` idi.

## P2 düzeltmesi: seçili proc hatasını reddetme

Önceki incelemeden yalnız **bu belge ve tests/expansion.rs** değişti; diğer 17 teslim girdisi aynıdır. Yeni helper yukarıdaki dar tanı sınırını uygular; numeric 13 kontrolü ve bütün diğer beklentiler korunur. Yeni bağımlılık, kaynak fixture veya toolchain değişikliği yoktur.

Karşı örnek yalnız `target/rust-backend-expansion/correction/examples` altındaki izole provider/probe kopyasında üretildi. Generator'ın `Literal::i32_unsuffixed(13)` ifadesi `Literal::u32_suffixed(13)` yapıldı. Gerçek açılım hâlâ sayısal 13 verir, fakat charge i32 ister. Sonuç zinciri:

| Aynı hatalı kaynak ve değişmeyen beklentiler | Gerçek sonuç |
|---|---|
| Önceki probe testi | **1/1 geçti**, seçili proc MacroFile içindeki literal için i32/u32 TypeMismatch loglandı; yanlış yeşil. |
| Compiler `cargo check --tests` | Exit **101**, tek hata kodu **E0308**. Bu dil kanıtıdır, ürün TDD kırmızısı değildir. |
| Güçlendirilmiş tanı assertion'ı | Derlendi; **0 geçti/1 kaldı**, `unexpected consumer diagnostics` içinde aynı proc i32/u32 TypeMismatch. Legacy çağrı ayrıca dar izin olarak loglandı. |

Ek dar bozulmada generator doğru 13'e döndürüldü, yalnız muafiyet çapası definition_items yerine aynı fonksiyondaki **callsite_items** invocation'ına bağlandı. Farklı gerçek MacroCallId yüzünden asıl legacy tanı muaf tutulmadı ve test assertion ile kırıldı. Yanlış u32 kalıntısı yoktu. Bu kontrol eski dokuz fault'un yeniden koşusu değildir.

Düzeltme sonrası doğru kaynakla **12/12 probe testi** (önceki 8 + yeni 4), locked/offline geçti; atlanan yok. İlgili probe fmt, check ve Clippy tüm hedeflerde geçti. Yeni assertion/helper için ayrı salt okunur okunabilirlik geçişinde bulgu yok; inceleme kaynak değiştirmedi. Bu düzeltmede yalnız bu 12'li tam koşu, yukarıdaki iki assertion kırılması ve tek E0308 compiler karşı örneği yeni kanıttır. İlk teslimin iki lib check'i, proc runtime koşusu, dokuz fault'u ve bağımsız incelemenin ek runtime kontrolleri tarihsel kanıttır; yeniden çalıştırılmış sayılmaz. Genel core/workspace/Maven koşusu yapılmadı.

Ayrı çıktı dizinleri stale-binary riskini sınırlar: doğru probe **cargo**, izole probe **correction-cargo**, compiler karşı örneği **correction-source** altını kullanır. Yalnız bu yazarın önceki cargo dependency cache'i correction-cargo'ya kopyalandı; analyzed-* çıktıları kopyalanmadı. İlk izole koşudan önce yalnız kopyanın probe paketi temizlendi; doğru ve bozuk paket aynı target'ı paylaşmadı. Komutların Compiling satırları gerçekten kullanılan iki farklı kaynak yolunu kaydeder. Tam taze dependency build'i iddia edilmez. İlk generator geri yüklemesi eski mtime/DLL cache'ini bıraktı; u32 kalıntı kontrolü bu ara koşuyu kanıt olarak reddetti. Kaynak taze yazılıp RA'nın gerçek analyzed-base-false-true alt hedefindeki yalnız generator cache'i temizlendi. Yeni koşuda u32 tanısı yoktu; yalnız yanlış muafiyet yüzünden reddedilen legacy tanı vardı. Debug0/incremental0 ayarları korundu; başka target/toolchain/global ayar değiştirilmedi.

Repo kökünden exact komutlar, yukarıdaki probe_cargo ayarlarıyla:

```sh
copy_probe="$PWD/target/rust-backend-expansion/correction/examples/rust-backend-probe/Cargo.toml"
copy_fixture="$PWD/target/rust-backend-expansion/correction/examples/rust-expansion/Cargo.toml"
correction_cargo() {
  env CARGO_TARGET_DIR="$PWD/target/rust-backend-expansion/correction-cargo" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RUSTFLAGS='-D warnings' cargo "$@"
}
correction_cargo clean --manifest-path "$copy_probe" -p rust-backend-probe
correction_cargo test --locked --offline --manifest-path "$copy_probe" --test expansion the_real_proc_server_expands_calls_resolving_to_the_consumers_wallet -- --exact --nocapture
env -u LOD_FIXTURE_BROKEN CARGO_TARGET_DIR="$PWD/target/rust-backend-expansion/correction-source" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RUSTFLAGS='-D warnings' LOD_FIXTURE_MODE=base cargo check --locked --offline --manifest-path "$copy_fixture" -p expansion-consumer --tests --message-format=json
env CARGO_TARGET_DIR="$PWD/target/rust-backend-expansion/correction-cargo/analyzed-base-false-true" cargo clean --manifest-path "$copy_fixture" -p expansion-generator
probe_cargo check --locked --offline --manifest-path "$probe_manifest" --all-targets
probe_cargo clippy --locked --offline --manifest-path "$probe_manifest" --all-targets -- -D warnings
probe_cargo test --locked --offline --manifest-path "$probe_manifest" --tests -- --test-threads=1 --nocapture
cargo fmt --manifest-path "$probe_manifest" -- --check
```

İzole test komutu önce eski probe, sonra güçlendirilmiş probe ile aynı 13u32 fixture üzerinde çalıştı; ardından doğru generator/yanlış invocation çapasıyla tekrarlandı. Durumlar `correction/old-expansion.rs`, `strengthened-expansion.rs`, `wrong-legacy-invocation.rs` ve üç test log'unda kayıtlıdır. Compiler JSON kaydı tam E0308 kodunu denetler. Orijinal 20 dış fixture girdisinin hash'leri değişmedi.

Bu düzeltmede [Semantics::hir_file_for/expand](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/semantics.rs), [TypeMismatch::expr_or_pat ve inline tanı toplama](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/diagnostics.rs), [ExprOrPatPtr](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir-def/src/expr_store.rs) ve [AstPtr::syntax_node_ptr](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/syntax/src/ptr.rs) kaynakları yeniden byte eşleşmesiyle doğrulandı. Kimlik çifti aynı DB'deki tek açılmış düğümü ayırır; kullanıcıya gösterilecek invocation/tanım/generated konum eşlemesi değildir. Bu aktif tip hatası deneyi, çözülemeyen çağrı/inactive cfg kapısını tamamlamaz.

Teslim SHA256SUMS belgenin ve probe'un **19 girdisini** kapsar; build çıktıları hariçtir. REUSED-SHA256SUMS eski scenario üç girdisi, selection.rs/helpers.rs ve expansion workspace'inin bütün 15 girdisi için ayrı **20 hash** tutar.

## Açık kalan sınır

2018/2024 makro token method lookup için gözlenen RA TypeMismatch çözülmedi. Invocation/tanım/generated konum eşlemesi, aynı invocation'daki aynı adlı iki çağrının ayrı kimliği, raw+expanded dedup ve etkin unresolved-call/inactive cfg negatifi sonraki dilimlerdir. Burada iki farklı proc metot hedefini bulmak bunları tamamlamaz. Bütün feature/hedef/cache birleşimleri, özel attribute/derive ve başka proc ABI'ları denenmedi. Eksik/kısmi model için ürün hata/çıkış politikası ve nihai backend kararı açık kalır.
