# Rust analiz altyapısı adayları

**İlk kontrollü deney için öneri: gömülü rust-analyzer `ra_ap_* = 0.0.358` ailesi.** Kaynak, metot seçimi ve tip bilgisine aynı modelden erişmek, Rust çözümleyicisini yeniden yazmaktan daha küçük iştir. Bu bir seçim/build kanıtı değildir: sonraki öğrenme deneyi sürüm kümesini derlemeli ve aşağıdaki geçiş koşullarını gerçek kaynakta doğrulamalıdır. Syn tek başına gereksinimi karşılamaz; derleyici bağlantısı, somut bir anlamsal eksiklik gösterilirse ikinci adaydır.

Taban `3ae2a74`, inceleme 23 Kasım 2026. Yalnız bu belge eklendi. Probe projesi, Rust adaptörü veya yeni ürün testi yoktur. [Senaryo kataloğunun](rust-scenarios.md) S2/S5 çağrı beklentileri ve önceki araştırmanın açık politika sınırları korunur.

## Gereksinime göre karşılaştırma

| Gereksinim | Syn 3.0.10 / yalnız sözdizimi | rust-analyzer 0.0.358 | rustc_driver / rustc_interface |
|---|---|---|---|
| S2/S5'te gerçek Wallet alıcısı ve dönüşü | `ExprMethodCall` alıcı ifadesi, ad ve argümanları tutar; hangi Wallet/metot olduğu ve çıkarılmış dönüş tipi yoktur. | `Semantics` tip, adjustment ve metot/hedef sorguları sunar. S2/S5 üzerinde kullanılabilirliği denenmedi. | Type checking sonucu tip, seçilen bildirim ve adjustment içerir. Entegrasyon derlenmedi. |
| Modül/bağımlılık kimliği, generic/trait/ilişkili tip | AST'den proje ve Rust çözümleme kurallarını ayrıca kurmak gerekir. | Cargo proje modeli, crate graph, HIR kimlikleri ve generic çözümleme vardır; declaration ile somut impl/gövde ayrımı yine denetlenmelidir. | Derleyicinin crate/DefId ve generic argümanları kullanılabilir; generic veya dyn çağrı tek somut gövde demek değildir. |
| cfg, edition, makro, build üretimi | Ham attribute/token okumak etkin programı belirlemez; Cargo/build/expansion sağlamaz. | Seçili Cargo bağlamı, deklaratif açılım ve dış proc-macro sunucusu var; OUT_DIR/cfg toplama ayrıca Cargo çalıştırabilir. | Gerçek rustc argümanlarıyla seçili programı derler/açar; Cargo'nun build girdileri ve bağımlılık argümanları yine gerekir. |
| Eksik etkin kod ve konum | Parse hatası bulunabilir; anlamsal belirsizlik görünmez. | Kısmi modele izin verir: `Option`, unknown tip ve tanılar kontrol edilmelidir. Açılmış kaynak→asıl kaynak bağı her zaman bire bir değildir. | Compiler tanıları vardır; hata yüzünden `after_analysis` aşamasına ulaşılmayabilir. Span/açılım bağları vardır, raporlama politikası hazır değildir. |
| Kurulum ve güncelleme | Normal Cargo kütüphanesi; en az kurulum, fakat eksik semantiği yazmanın maliyeti büyük. | Normal yayımlanmış crate yolu; std kaynakları ve proc-macro uyumu gerekir. Public API olması kararlılık garantisi değildir; birlikte sürümle. | Kararsız compiler-private API, eşleşen toolchain/kütüphane/ABI dağıtımı gerekir; operasyon yükü daha yüksek. |

Syn'un [sürüm kaynak AST'si](https://raw.githubusercontent.com/dtolnay/syn/97ec37dda8a893a460066da694fd1eb7cb30538a/src/expr.rs), [rust-analyzer mimarisi](https://rust-analyzer.github.io/book/contributing/architecture.html) ve [compiler bağlantısı rehberi](https://rustc-dev-guide.rust-lang.org/rustc-driver/intro.html) bu ayrımı destekler. Mevcut Java adaptörünün tree-sitter yolu da sözdizimini ayrı çözümleyiciyle tamamlar; bunu Rust'a taşımak trait/çıkarım/makro motoru sağlamaz. RA kendi syntax ağacını verir; ilk deneyde ikinci bir Syn/tree-sitter ağacı eklemek için ihtiyaç yoktur.

**Maliyet çıkarımı, ölçüm değil:** Syn'un parse kurulumu dar; gereken semantiği sonradan eklemek geniş iştir. RA dosya/crate verisini ve sorgu önbelleklerini tutar; tek seferlik yükleme, bağımlılık derleme, Cargo check ve proc süreci maliyeti vardır. `prefill_caches` seçilebilirdir; ilk dar deney ön doldurma olmadan başlayabilir. Driver derleyicinin seçili analiz aşamalarını çalıştırır. Bu üç yol için süre, RAM, binary boyutu veya performans sıralaması ölçülmedi; sayı verilmez.

## Kontrol edilen sürümler ve yerel kurulum

Rust/Cargo **1.99.1**, host `aarch64-apple-darwin`; rustc commit'i `58cf8f9c35b355199727627d07ae34a4e2846d67`. Aktif toolchain stable'dır. `rust-src`, `rustc-dev`, `llvm-tools` ve `rust-analyzer` bileşenleri kurulu değildir. `command -v rust-analyzer` bir rustup shim'i gösterir; `rust-analyzer --version` çıkış **1**, “Unknown binary” verir. Bu kullanılabilir bir analyzer kurulumu değildir.

Sysroot'ta `libexec/rust-analyzer-proc-macro-srv` dosyası vardır. `--version` çıkış **122**, dahili/kararsız araç uyarısı verir; protokol anlaşması veya makro çalışması sınanmadı. Std kaynak dizini `lib/rustlib/src/rust/library` yoktur. [Kurulum belgesi](https://rust-analyzer.github.io/book/installation.html) std kaynaklarını ve toolchain uyumunu ayrıca şart koşar.

| Metadata kontrolü | Sonuç ve sınırı |
|---|---|
| `syn@3.0.10` | `rust-version = 1.71`; tam dosya AST'si için `full` ve parsing gerekir. Derlenmedi. |
| `ra_ap_hir`, `ra_ap_ide_db` **0.0.363** | İkisi bulundu, MSRV 1.98. Aynı sürüm `ra_ap_load-cargo` ve `ra_ap_project_model` bulunamadı; iki sorgu çıkış 101. Yeni HIR'ı eski loader ile karıştırma. |
| **0.0.358**: `ra_ap_load-cargo`, `ra_ap_project_model`, `ra_ap_hir`, `ra_ap_ide_db`, `ra_ap_syntax`, `ra_ap_vfs`, `ra_ap_proc_macro_api`, `ra_ap_hir_ty`, `ra_ap_hir_def`, `ra_ap_hir_expand`, `ra_ap_base_db`, `ra_ap_span`, `ra_ap_ide_diagnostics` | On üçü bulundu; her birinin MSRV'si 1.98. Loader/project-model/HIR/IDE-DB manifestlerinde iç `ra_ap` bağımlılıkları `=0.0.358`. Bu metadata uyumudur; tüm geçişli bağımlılıklar çözülmedi, lock/build yapılmadı. |

İlk deneyin doğrudan API başlangıcı `ra_ap_load-cargo`, `ra_ap_project_model`, `ra_ap_hir`, `ra_ap_ide_db`, `ra_ap_syntax`, `ra_ap_vfs` için **tam `=0.0.358`** olsun; gerçekten kullanılan ek doğrudan RA crate'i de aynı sürümden gelsin. `ide_diagnostics` yalnız alternatif tanı katmanı olarak incelendi; ilk probe için zorunlu sayılmıyor. Bütün bağımlılık ağacı ve kilit dosyası sonraki deneyde doğrulanacaktır. [Yayımlanmış loader manifesti](https://docs.rs/crate/ra_ap_load-cargo/0.0.358/source/Cargo.toml) sürüm kenarlarını gösterir.

## Kaynakta görülen API'ler; entegrasyon kanıtı değil

RA 0.0.358 kaynaklarının kayıtlı upstream commit'i `2ad81e72b7e978cb800dffe1bcce5c9f98032acf`'tır. Aşağıdaki bağlantılar bu sürüme gider. `hir` kütüphane sınırıdır; doğrudan `hir_ty` iç sorgularına bağımlılık ilk seçenek değildir. LSP hover/debug metni tip/kimlik protokolü olarak ayrıştırılmaz.

| İhtiyaç | Kontrol edilen giriş ve önemli sınır |
|---|---|
| Projeyi yükle | [`load_workspace_at` / `load_workspace`](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/load-cargo/src/lib.rs#L38-L91) → `RootDatabase`, `Vfs`, isteğe bağlı proc istemcisi. AST aynı DB/edition bağlamından alınmalı. |
| Gerçek alıcı, tip ve hedef | [`Semantics::type_of_expr`, `expr_adjustments`, `resolve_method_call`, `resolve_expr_as_callable`](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/semantics.rs#L1728-L1820). Original/adjusted tip ile gerçek receiver ilişkisi ayrı okunur; `Type::as_reference`/`as_adt` ile yapı incelenir, display adı kimlik yapılmaz. |
| Çağrıya uygulanmış dönüş ve generic bilgi | [`Callable::return_type`, `Function::ret_type`, `Type::contains_unknown`](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/lib.rs). Bildirimdeki `T::Output` ile çağrıdaki çözülen sonuç aynı şey değildir; tip/bounds/substitution deneyde ayrıştırılmalı. |
| Bildirim ile impl | [`resolve_method_call_as_callable`](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/semantics.rs#L1900-L1907) için kaynak açıkça doğru trait impl'i çözmediğini söyler. [`resolve_method_call` iç yolu](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/source_analyzer.rs#L630-L653) impl veya trait bildirimine gider. Tek callable sonucu somut helper gövdesi diye kullanılmaz; dyn hedefi açık kalır. |
| Açılım/konum | `expand_macro_call`, `expand_attr_macro`, `expand_derive_macro` vardır. [`original_range`](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/semantics.rs#L1551-L1577) eşleyemezse invocation aralığına döner; `original_range_opt` boş olabilir. `original_ast_node` genel ters açılım değildir, attribute kapsamıyla sınırlıdır. |
| Eksik model | [`Module::diagnostics`](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/lib.rs#L693) ve [`AnyDiagnostic::UnresolvedMethodCall`](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/diagnostics.rs) vardır. `None`/unknown, parse/expansion/build hatalarıyla birlikte taşınmalı; tanısız sorgu veya başarılı yükleme tamlık kanıtı değildir. |

### Cargo bağlamı ve kod çalıştıran sınır

[`CargoConfig`](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/project-model/src/cargo_workspace.rs#L93-L177) özellik, target, sysroot, cfg override, environment ve hedef dizini girdilerini tutar. Paket/target kimliği ve edition da proje modelindedir. Buna rağmen aşağıdaki varsayımlar deneyde denetlenmelidir:

- **IDE cfg'si compiler cfg'si değildir:** [workspace kodu](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/project-model/src/workspace.rs#L1293-L1307) yerel crate'e `rust_analyzer`, `set_test` açıksa `test` ekler, sonra override uygular. Normal S2/S5 analizinde test kapalı olmalı; `cfg(rust_analyzer)` farkı da giderilmeli/doğrulanmalı. Bir konfigürasyon bütün feature/target'ları kanıtlamaz.
- **Otomatik kurulum yolu vardır:** [`Sysroot::discover`](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/project-model/src/sysroot.rs#L467-L481) eksik rust-src için `rustup component add` çağırabilir. Sonraki deney kaynak gereksinimini önceden çözmeli; açık sysroot **ve** kaynak yolu verilen [yükleme dalı](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/project-model/src/workspace.rs#L231-L245) keşif/kurulum yoluna girmez. Bu karşılaştırmada çağrılmadı, bileşen kurulmadı.
- **Build bilgisi bedelsiz dosya okuma değildir:** `load_out_dirs_from_check` ile `run_build_scripts`, varsayılan [Cargo check komutunu](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/project-model/src/build_dependencies.rs) çalıştırır; build.rs, bağımlılık derleme ve compiler sırasında proc kodu çalışabilir. OUT_DIR/cfg/env/proc dylib çıktıları buradan gelir. Cache her çağrıda build.rs'nin tekrar çalıştığı anlamına gelmez.
- **`Ok(database)` yeterli değildir:** `load_workspace_at` build hatasını debug log'a yazıp devam edebilir; proc sunucusu hatası da DB yüklemesini engellemeyebilir. Deney `ProjectWorkspace::load` → `run_build_scripts` → `WorkspaceBuildScripts::error` denetimi → çıktıları ekleme → `load_workspace` yoluyla bu durumu gözleyebilir; proc/semantic hataları ayrıca kontrol etmelidir.
- **Proc ABI/protokolü ayrı koşuldur:** [`ProcMacroClient` sürüm kontrolü](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/proc-macro-api/src/process.rs) fazla yeni sunucuyu reddedebilir. Mevcut executable, MSRV veya sunucunun başlaması gerçek açılım kanıtı değildir. Seçili toolchain'in sunucusuyla yerel makro deneyi geçmelidir.

Dosya/metadata okumak ve `cargo info` proje build/proc kodunu çalıştırmaz. Cargo check/build ile build.rs; proc açılım isteğiyle makro sağlayıcısının kodu çalışabilir. RA semantik çekirdeği IO yapmaz, yükleyici ve proc süreci yapar; IDE API sınırları açıkça kararsızdır. [Mimari açıklaması](https://rust-analyzer.github.io/book/contributing/architecture.html#stability-guarantees) bu ayrımı ve API sınırlarını belgeler. Bu yolculukta yalnız metadata/kaynak indirme ve araç sorgulama yapıldı; proje yüklenmedi/derlenmedi.

### Compiler alternatifi

Yerel rustc commit'inin [driver kaynağında](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/compiler/rustc_driver_impl/src/lib.rs#L120-L172) `run_compiler` ve `Callbacks::{after_expansion, after_analysis}` vardır. Güncel rehberin giriş adı `compiler_entrypoint`'tir; bu yüzden güncel rehber örneği sabitlenmiş eski compiler API'si sayılmaz. `rustc_interface` daha alt seviyedir; gereksizse driver tercih edilir.

[`TypeckResults`](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/compiler/rustc_middle/src/ty/typeck_results.rs#L288-L425) üzerindeki `type_dependent_def_id`, `node_args`, `expr_ty`, `expr_adjustments` gerçek bildirim/tip/substitution için aday girişlerdir; [hygiene/expansion verisi](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/compiler/rustc_span/src/hygiene.rs) kaynak bağını destekler. Bunlar hazır LoD veya alias/effect özeti değildir. Derleme hatası analiz callback'ini kesebilir; MIR debug metni anlamsal protokol yapılmaz.

`rustc_private` nightly özellik ve toolchain'e bağlı kararsız bağlantıdır; resmi toolchain için [rustc-dev/llvm-tools gereksinimi](https://doc.rust-lang.org/unstable-book/language-features/rustc-private.html) vardır. Burada bu bileşenler yoktur; driver derlenmedi. Dağıtılacak binary, compiler kütüphaneleri ve sysroot/ABI birlikte sürümlenmelidir. Cargo'nun gerçek extern/cfg/target/edition argümanlarını aktarmak ve compiler tanılarını korumak da gerekir; compiler backend seçmek bu yükü kaldırmaz.

## Sonraki sınırlı öğrenme deneyi

Önce tek küçük probe'da exact RA sürümlerini çöz/derle, kilitle; açık std kaynak yolu ve uyumlu proc sunucusunu doğrula. Kurulum hatası öğrenme koşulunu karşılamaz, eksik ürün davranışının kırmızı testi değildir. Yalnız sahip olunan fixture/probe build kodu çalıştırılsın; çıktı ayrı target dizinine gitsin. Eski girdileri değiştirmeden kullan veya geçici kopyayı paketle.

| Girdi / kapsam | Bağımsız başarı gözlemi | Adayı durduran durum |
|---|---|---|
| [S2/S5, normal lib](../examples/rust-scenarios/core.rs) | Enjekte edilmeden Wallet ADT kimliği, `&Wallet` alıcı/dönüş ve `Wallet::debit` / `Order::wallet` hedefleri; `core.rs:13/29` bağı. İki hedef aynı DB/konfigürasyondadır. | Yalnız ad/hover metni; dönüşe beklenen tip verilmesi; zincirin veya çağrının kaybolması. Bu aşama port implementasyonu veya LoD rapor testi değildir. |
| [Başka modül/gerçek crate sınırı](rust-visibility.md) | Re-export ve adı dışarıdan yazılamayan dönen public tip aynı provider bildirimiyle çözülür; consumer'ın benzer adlı tipiyle karışmaz. | Dosya adı/short name kimlik sayılır; private yol yüzünden değerden erişilen public üye kaybolur. |
| [selection.rs](../examples/rust-method-resolution/selection.rs), açık test cfg'si; [dyn helper](../examples/rust-visibility/helpers.rs) | `quote_from_bound` içinde `Quote::quote` ve `T::Output`, somut çağrısında `u32`; concrete inherent alternatifinden ayrılır. Qualified gerçek receiver korunur. Dyn `selected` bildirimi bilinir, `Replaced` override'ı yerine varsayılan gövde kesinleştirilmez. | Generic gövde somut inherent metoda çevrilir; generic dönüş kaybolur; tek trait bildirimi kesin runtime gövdesi yapılır. |
| [Expansion workspace](../examples/rust-expansion/Cargo.toml), belgelenmiş base/bonus ve build-input seçimleri | Seçili cfg/feature/target/edition ve generated.rs sahipliği; gerçek function-like proc açılımının sonucu; 2018 makro token bağlamı korunur. | Aktif makro açılmadan temiz sonuç; yanlış cfg/edition; eski OUT_DIR başka konfigürasyon sanılır. Bütün proc türleri kapsandı denmez. |
| Aynı expansion fixture'ının konumları | Invocation, tanım ve generated dosya bağı ayrılır; aynı invocation'a eşlenen iki gerçek üretilmiş çağrı iki kimliktir. | `(dosya,satır,ad)` ile iki çağrı birleşir veya raw+expanded çağrı iki kez sayılır; fallback span kesin köken sayılır. |
| [Etkin çözülemeyen çağrı](../examples/rust-expansion/consumer/negative/cfg_boolean.rs), karşısında inactive cfg | Etkin hata için tanı ve çözülemeyen hedef/tip gözlenir; yükleme/build/proc hatası görünürdür. Etkin olmayan item ayrı kalır. | `None` literal/yeni nesne olur; parse/load başarısı yüzünden eksik aktif kod uygun sayılır. |

Bu kapılardan biri sürüm/API uyumsuzluğu yüzünden kapanırsa önce **bütün** RA ailesinin başka tutarlı yayımlanmış sürümü değerlendirilir; karışık sürüm çözümü yapılmaz. Gerçek alıcı/trait/ilişkili tip, expansion veya kaynak kimliği için gereken bilgi alınamıyorsa, somut karşı örnekle sabit toolchain'li driver deneyi açılır. Syn tek başına semantik fallback değildir. Son seçim ve version pinning bu deneylerden sonra yapılır; bugün hiçbir kapı çalıştırılmadı.

## Mevcut portlara en küçük bağlantı

Öneri: CLI/adaptör tek seçili proje snapshot'ını yükler. Aynı DB/VFS bağlamını kullanan Rust `TypeSource`, sahip olunan `TypeShape`/`Platform`; `SourceReader` ise `SourceFile`/`Method`/`Call` üretir. [CheckProject](../crates/core/src/application/check_project.rs) önce tipleri, sonra kaynakları okur; [portlar](../crates/core/src/application/ports.rs) değişmeden ilk bağlantı denenebilir. İki port için iki bağımsız Cargo yüklemesi yapıp cfg kaydırma. HIR/DB ömürleri ve Cargo bilgisi domain'e taşınmaz.

`returning`/`resolving` gibi mevcut köprüler gerçek anlamları uyduğunda kullanılır; ordinary argüman alias'ını functional `returning_from` diye sunma. Tip/çağrı hedefi, bilinen gövde, nesne kökeni ve etki farklı bilgilerdir. Backend hiçbiri için tam program alias/effect kanıtı veya LoD/Value/Hybrid politikası vermez. Bağlama/projection/body kökeni eşlemesi hâlâ adaptör işidir; eksik bilgi ancak kanıtlanmış ihtiyaçla dil bağımsız ek sözleşme gerektirir. [Köken](rust-origins.md), [kapsayıcı](rust-containers.md), [değişebilirlik](rust-mutability.md) ve [sınıflandırma](rust-classification.md) açıkları korunur. Eksik analiz raporu/çıkış kodu sonraki karar satırıdır; mevcut `ReadError` tamlık garantisi sayılmaz.

İlk ürün kabulü daha sonra katalogdaki **call-only S2/S5 → gerçek adaptörler → CheckProject → rapor** karşılaştırmasıdır. Wallet parametresi izinli, `wallet()` dönüşündeki `debit` tek ihlal olmalı; hibritlik assertion'ı yoktur. Bu belge onu çalıştırmış veya uygulamış sayılmaz.

## Gerçekte yapılan doğrulama

Araç/kurulum sorguları: `rustc --version --verbose`, `cargo --version`, `rustc --print sysroot`, `rustup show active-toolchain`, `rustup component list --installed`, `command -v rust-analyzer`, `rust-analyzer --version`; sysroot kaynak/server dosyalarının varlığı ve server `--version` kontrol edildi. Yalnız son iki version sorgusu yukarıdaki beklenen hataları verdi. Araç kurulmadı/değiştirilmedi.

Repo dışında `/tmp` çalışma dizininde metadata komutları; paket kaynakları Cargo cache'e indi:

```sh
cargo info --registry crates-io syn@3.0.10 --verbose
for package in ra_ap_hir ra_ap_ide_db ra_ap_load-cargo ra_ap_project_model; do
  cargo info --registry crates-io "$package@0.0.363"
done
for package in ra_ap_load-cargo ra_ap_project_model ra_ap_hir ra_ap_ide_db ra_ap_syntax ra_ap_vfs ra_ap_proc_macro_api ra_ap_hir_ty ra_ap_hir_def ra_ap_hir_expand ra_ap_base_db ra_ap_span ra_ap_ide_diagnostics; do
  cargo info --registry crates-io "$package@0.0.358" --verbose
done
```

`cargo search` loader/project-model adayını bulmak için kullanıldı; seçim kanıtı yapılmadı. Dört giriş manifesti ve `.cargo_vcs_info.json` yerel registry kaynağından okundu. RA'nın VCS kaydı `dirty:true` içerir: tüm paket arşivi upstream commit ile özdeş denmedi. RA'nın `hir/{lib,semantics,source_analyzer,diagnostics}.rs`, `load-cargo/lib.rs`, `project-model/{lib,cargo_workspace,build_dependencies,workspace,sysroot}.rs`, `proc-macro-api/process.rs` dosyaları ve Syn'un `expr.rs` dosyası kayıtlı commit'lerin raw GitHub içeriğiyle Python `urlopen(...).read() == Path(...).read_bytes()` karşılaştırmasında **12/12 aynıydı**. Syn commit'i `97ec37dda8a893a460066da694fd1eb7cb30538a`'dır. Compiler'ın yukarıdaki üç kaynak dosyası ve `rustc_interface/src/interface.rs`, yerel compiler commit'inden ayrıca okundu; compiler kütüphanelerine bağlanan program derlenmedi.

Güncel birincil belgeler ve belgedeki yerel/uzak bağlantılar kontrol edildi. Not/indirilen kaynaklar yalnız `target/rust-backends` ve Cargo cache'tedir. Yeni Rust kaynak/probe olmadığından test, rustfmt, Clippy, workspace veya Maven çalıştırılmadı; önceki fixture sonuçları yeni koşu diye sunulmadı. Ayrı salt okunur metin geçişinde öneri/kanıt/denenmemiş entegrasyon ayrımı incelendi; kaynak veya test okunabilirliği değişmedi.
