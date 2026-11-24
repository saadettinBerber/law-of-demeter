# Rust makro token edition teşhisi

**Uyuşmazlık build/proc gerektirmeyen iki crate'te yeniden üretildi.** RA 0.0.358, 2018 metot token'ının bağlamını koruyor; fakat 2024 tüketici gövdesinde array metot seçimini owned IntoIterator olarak yapıyor. Doğrudan 2018 çağrısı ve tüketiciden gelen 2024 token karşı kontrolleri doğru. Sürümlü kaynak izi, edition filtresinin token yerine gövdenin crate edition alanını kullanmasına gidiyor. Adaptör yükleyicisinde dar bir düzeltme bulunmadı; **destek kabul deneyi kırmızı kaldı**.

Taban `feb3b22c55accdd9ce2fa4d29294e4c97314b606`, tarih 23 Kasım 2026. Önceki araştırma/fixture'lar, build/proc diliminin dar tanı koruması, production, Cargo pinleri ve decision.md değiştirilmedi. Nihai backend seçimi veya Rust ürün desteği tamamlanmadı.

## Küçük karşı örnek

Yeni [`fixtures/macro-edition`](../examples/rust-backend-probe/fixtures/macro-edition/Cargo.toml) yalnız yerel **edition-provider (2018)** ve **edition-consumer (2024)** içerir. Dış dependency, build.rs ve proc-macro yoktur. Sağlayıcı iki macro_rules tanımlar:

```rust
macro_rules! definition_items {
    ($array:expr) => { $array.into_iter() };
}
macro_rules! callsite_items {
    ($array:expr, $method:ident) => { $array.$method() };
}
```

Gerçek fixture bunları macro_export ile dışa açar. İlk metot adı token'ı sağlayıcıda yazılıdır; ikincisi tüketiciden gelir. [Rust Reference](https://doc.rust-lang.org/reference/expressions/method-call-expr.html#r-expr.method.edition2021), array için 2021 öncesi IntoIterator adayının elenmesinde **metot adı token'ının** edition'ının kullanılacağını belirtir.

[`consumer/src/lib.rs`](../examples/rust-backend-probe/fixtures/macro-edition/consumer/src/lib.rs) içindeki definition_count ve callsite_count, makro sonucunu **tip annotation'ı olmadan** `let iterator = ...` ile bağlar. İkisi de yalnız count sonucu usize döndürür; iterator tipi dönüş annotation'ından alınmaz. Ayrı reference binding'i, definition_count'ta açık `(&values).into_iter()`, callsite_count'ta doğrudan owned array çağrısıdır. Probe her iki binding'de de annotation bulunmadığını denetler.

Compiler'ın bağımsız iki testi aynı makroları kullanır: legacy çağrının ilk öğesi orijinal array'in ilk String referansıyla ptr::eq olmalıdır; tüketici token'ının ilk öğesinde String::into_bytes sahip olunan String'i tüketebilmelidir. Iterator tipi bu testlerde de annotate edilmez. Rustc 1.99.1 ikisini kabul edip çalıştırdı: **2/2**. Sağlayıcıdaki direct_count, aynı array sözdizimini doğrudan 2018 gövdesinde kullanarak std metadata/edition filtresi için yakın kontroldür. Yeni bir edition matrisi veya üçüncü crate gerekmedi.

## Gerçek model ve typed gözlem

Yeni [`examples/macro_edition_gate.rs`](../examples/rust-backend-probe/examples/macro_edition_gate.rs) gerçek ProjectWorkspace/load_workspace yolunu kullanır. Metadata/sysroot hataları, parse hatası, çözülmeyen Macro/Function ve unknown tipler görünür hata olur. Explicit sysroot ve eşleşen rust-src korunur; build çalıştırma ve proc sunucusu kapalıdır. Kaynaklar kopyalanıp açılım enjekte edilmez; Semantics::expand çıktısı aynı DB'de sorgulanır.

Modelde provider ve consumer, gerçek VFS kök yollarından bulunan **farklı Crate kimlikleri**dir. Edition'ları sırasıyla 2018/2024'tür. Normal kontrol `set_test=false`, başarısız kabul deneyi `set_test=true` kullanır; iki yerel crate'in gerçek cfg bitleri aynı seçime eşit, rust_analyzer bitleri override ile kapalıdır. RA'nın set_test seçimi yerel crate'lere uygulanır; bunu Cargo'nun bütün target çeşitlerinin bire bir modeli saymıyoruz. Provider'da cfg'ye bağlı makro gövdesi yoktur. Normal modelde test modülünün ham AST'si vardır, HIR'da etkin modül yoktur; test bağlamında compiler tanık fonksiyonu gerçekten is_test'tir.

Macro bildirimi sağlayıcı AST'sinden to_macro_def ile bağımsız alınır; invocation'ın resolve_macro_call sonucu bununla eşittir ve provider crate'ine aittir. to_def(invocation) gerçek MacroCallId verir. Açılmış çağrının HirFileId'si aynı MacroFile kimliğidir. Metot NameRef aralığından expansion_span_map ile okunan Span, hem SyntaxContext::edition hem kaynak anchor edition bilgisini sağlar; elle edition atanmaz.

| Çağrı | Compiler beklentisi | RA 0.0.358 gözlemi |
|---|---|---|
| Provider'ın doğrudan 2018 array.into_iter çağrısı | Borrowed slice Iter | **Doğru:** Borrow(Ref(Shared)) adjustment; `&[String;2]` alıcı; açık borrowed karşı kontrolle aynı tip ve Function kimliği. |
| Consumer'ın callsite_items!(values, into_iter) çağrısı | Owned array IntoIter | **Doğru:** token context/anchor 2024, adjustment yok; doğrudan consumer çağrısıyla aynı tip ve Function kimliği. Macro bildirimi hâlâ 2018 provider'ındadır. |
| Consumer'ın definition_items!(values) çağrısı | Borrowed slice Iter ve borrowed implementation | **Yanlış:** token context/anchor **2018** olmasına rağmen adjustment yok, bound self owned `[String;2]`, ifade tipi **array IntoIter**. Açık borrowed kontrolün slice Iter tipi ve Function kimliğiyle farklıdır. |

Son satırda hem **orijinal makro ifade tipi** hem **açılmış metot ifade tipi** aynı yanlış owned sonucu verir; ikisi unknown değildir. Bu, yalnız yanlış AST parçasını sorgulama veya annotation'ı sonuç sanma açıklamasını dışlar. resolve_method_call somut implementation kimliğini verir; callable receiver_param owned array self bilgisini gösterir. Callable::return_type ise ilişkili IntoIter projeksiyonudur: ham imza ile normalize edilmiş gerçek ifade tipi karıştırılmaz.

Test modelinin tanısı, compiler'ın kabul ettiği ptr::eq tanığında **beklenen *const String / bulunan String TypeMismatch**'idir. İlk eski fixture'daki Iter/IntoIter annotation tanısıyla aynı yazım değildir; bu kez annotation olmadan yanlış item tipi görülür. Normal modelde yalnız InactiveCode vardır: tanısız aktif gövdede bile yanlış iterator tipi üretilebilmesi, boş tanının doğruluk kanıtı olmadığını gösterir. Pozitif kontrol diğer tanıları assertion ile reddeder; başarısız kabul deneyi borrowed sonuç, borrowed hedef ve hata tanısı yokluğu koşullarının üçünü de ister. Hiçbiri RA'nın yanlış sonucuna çevrilmedi.

## İlk ayrışma ve sınırlandırılmış neden

RA sürüm commit'i **2ad81e72b7e978cb800dffe1bcce5c9f98032acf** için iz:

1. [InferenceContext kurulumu, infer.rs:1453](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir-ty/src/infer.rs#L1453), edition alanını `resolver.krate().data(db).edition` değerinden alır. Bu örneğin consumer fonksiyon gövdesi 2024'tür.
2. [with_method_resolution, method_resolution.rs:196](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir-ty/src/method_resolution.rs#L196), MethodResolutionContext'e `edition: self.edition` taşır.
3. [Array aday filtresi, probe.rs:1633](https://github.com/rust-lang/rust-analyzer/blob/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir-ty/src/method_resolution/probe.rs#L1633), `self.ctx.edition.at_least_2021()` ile karar verir. Burada metot token'ının korunan 2018 context'i okunmaz; consumer gövdesinin 2024 bilgisi owned array adayını elemez.

Bu kaynak yolu, typed sonuç ve yakın kontroller birlikte **yayımlanmış backend'in method lookup edition sınırını** gösterir. Cargo crate edition'ları doğru, Macro kimliği doğru, açılım token context'i doğru, ham/açılmış ifade sorguları tutarlıdır. Aynı backend'in doğrudan 2018 çağrısında borrowed adayı seçmesi, std'deki edition filtresinin bütünüyle eksik olduğu açıklamasını da dışlar. İlk saptanan ayrışma bu metot seçimidir; genel token hygiene desteğinin her yönünün doğru olduğu iddia edilmez.

İç fonksiyonlar debugger ile enstrümante edilmedi; registry/fork patch'iyle bir düzeltme denenmedi. Bu, sürümlü kaynak ve dar davranış kanıtıyla sınırlı teşhistir. Public typed API bulunması kararlılık sözü değildir. [Semantics](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/semantics.rs), [source_analyzer](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/source_analyzer.rs), [expansion span map](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir-expand/src/span_map.rs), [SyntaxContext](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/span/src/hygiene.rs) ve [Span anchor](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/span/src/lib.rs) dahil sekiz dosya yayımlanmış 0.0.358 kaynaklarıyla byte olarak eşleşti; kayıt upstream/verified.json'dadır.

## Paketleme düzeltmesi ve doğrulama

İlk teşhisin `tests/macro_edition_gate.rs` hedefi **1 kontrol geçti / 1 kabul kaldı**, exit 101 veriyordu. Bağımsız inceleme teknik teşhisi doğruladı, fakat normal test keşfini kıran bu paketleme P2 idi. Bu tarihsel test sonucu yeni executable koşusuyla karıştırılmaz.

Kaynak **examples/macro_edition_gate.rs** yoluna taşındı; eski test dosyası kaldırıldı. Yalnız iki test attribute'u kaldırıldı ve sade main eklendi. Main normal kontrol fonksiyonunu çağırır, başarıyla döndükten sonra **Direct and forwarded token control passed** yazar; ardından doğru borrowed kabul fonksiyonunu çalıştırır. Typed edition/kimlik/tip/tanı sorguları ve bütün eski assertion'lar aynıdır. Dev-dependency'li normal Cargo example yeterlidir; manifest, feature, ignore/should_panic veya başarısızlığı 0'a çeviren akış yoktur.

| Son paketleme koşusu | Sonuç |
|---|---|
| Normal cargo test --tests | **12/12**, exit 0. |
| Normal cargo test --all-targets | **12/12**, exit 0; example harness'inde **0 test**, main çalışmadı. |
| Açık cargo run --example macro_edition_gate | Kontrol tamamlandı mesajı, ardından gerçek borrowed assertion paniki; **exit 101**. Test harness sayacı yoktur. |
| Probe check/Clippy tüm hedefler ve fmt | Geçti, exit 0; yeni lint izni yoktur. |

Normal test keşfi örneğin main'ini çalıştırmaz; example test harness'i varsa 0 testtir. Böylece standart regresyon yeşili, explicit diagnostic'in başarısız borrowed kabulünü gizlemez. Backend kapısı hâlâ başarısızdır; normal test başarısı bu kapının geçtiği anlamına gelmez. Programın ikinci Model'i kendi host test cfg'sinden bağımsız olarak set_test=true kurmaya devam eder.

**Tarihsel kanıt:** değişmeyen compiler fixture'ı 2/2 geçti; provider'da 0 test vardı. Önceki fixture check/Clippy/fmt, sekiz primary kaynak eşleşmesi ve aşağıdaki dört fault bu paketleme düzeltmesinde tekrar koşulmadı. Üç RA bozulması yanlış caller, ters test cfg ve normal lib'de yanlış bool dönüşünü runtime assertion ile yakalamıştı; compiler fault'u skip(1) ile yanlış ilk referansı yakalamıştı. Eski test kaynağı/komut/log kayıtları faults, fault-results.json ve compiler-fault-result.json altında korunur. Beklentiler değiştirilmedi; bunlar yeni main doğrulaması sayılmaz. Geniş core/workspace/Maven veya önceki dil paketleri koşulmadı.

Taşınan kaynak ve yeni main için ayrı salt okunur okunabilirlik incelemesi yapıldı; bulgu yok, inceleme kaynak değiştirmedi. Kontrolün tamamlanması ile sonraki başarısız kabul adımı çıktıda ve akışta ayrıdır; doğru borrowed beklentisi korunur.

## Exact komutlar ve kurulum

Rust/Cargo **1.99.1**, rustc commit'i **58cf8f9c35b355199727627d07ae34a4e2846d67**, host aarch64-apple-darwin. Explicit sysroot `/Users/deep/.rustup/toolchains/stable-aarch64-apple-darwin`, rust-src bunun `lib/rustlib/src/rust/library` dizinidir. Toolchain/component/global config değişmedi. Dokuz doğrudan RA pini **=0.0.358**, 27 RA paketi, 179 paketlik ana lock ve Salsa/Unicode seçimleri başlangıçla byte olarak aynıdır; yeni fixture lock'u yalnız iki yerel paket içerir.

Paketleme düzeltmesinde repo kökünden kullanılan komutlar:

```sh
probe_manifest=examples/rust-backend-probe/Cargo.toml
probe_cargo() {
  env CARGO_TARGET_DIR="$PWD/target/rust-macro-edition/cargo" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RUSTFLAGS='-D warnings' cargo "$@"
}
probe_cargo test --locked --offline --manifest-path "$probe_manifest" --tests -- --test-threads=1
probe_cargo test --locked --offline --manifest-path "$probe_manifest" --all-targets -- --test-threads=1
probe_cargo run --locked --offline --manifest-path "$probe_manifest" --example macro_edition_gate
probe_cargo check --locked --offline --manifest-path "$probe_manifest" --all-targets
probe_cargo clippy --locked --offline --manifest-path "$probe_manifest" --all-targets -- -D warnings
cargo fmt --manifest-path "$probe_manifest" -- --check
```

Explicit run komutu bu pinlerle gerçek borrowed assertion'ında exit **101** verir; normal test/check/lint/format komutları exit 0 olmalıdır. Koşu log'ları packaging/ altındadır. Cargo metadata, macro_edition_gate için yalnız example target'ı gösterir; eski test dosyası yoktur. Yeni kaynak yolu ilk run sırasında gerçekten derlendi ve debug/examples/macro_edition_gate çalıştı; eski test binary'si kullanılmadı. Main panik/exit kodu yakalamaz; kontrol başarı mesajı yalnız kontrol Result'ı başarıyla döndükten sonra yazılır.

İlk teşhisin değişmeyen compiler/fixture komutları (tarihsel):

```sh
fixture_manifest=examples/rust-backend-probe/fixtures/macro-edition/Cargo.toml
fixture_cargo() {
  env CARGO_TARGET_DIR="$PWD/target/rust-macro-edition/source" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RUSTFLAGS='-D warnings' cargo "$@"
}
CARGO_TARGET_DIR="$PWD/target/rust-macro-edition/source" cargo generate-lockfile --offline --manifest-path "$fixture_manifest"
fixture_cargo test --locked --offline --manifest-path "$fixture_manifest" --workspace --lib
fixture_cargo check --locked --offline --manifest-path "$fixture_manifest" --workspace --all-targets
fixture_cargo clippy --locked --offline --manifest-path "$fixture_manifest" --workspace --all-targets -- -D warnings
cargo fmt --manifest-path "$fixture_manifest" --all -- --check
```

Üç eski RA fault'u fault-cargo hedefinde, geçici kopyanın eski `--test macro_edition_gate the_model_preserves_editions_and_direct_or_forwarded_token_dispatch -- --exact --nocapture` komutuyla çalıştı. Her bozulmadan önce yalnız kopyanın probe paketi temizlendi. Eski compiler fault'u fault-source hedefinde `cargo test --locked --offline --manifest-path target/rust-macro-edition/compiler-fault/Cargo.toml -p edition-consumer --lib -- --exact tests::a_definition_token_keeps_reference_identity` kullandı. Hepsinde debug0/incremental0/RUSTFLAGS ayarları korundu; bu fault'lar yeniden koşulmadı.

Yalnız yazarın önceki dependency cache'i yeniden kullanıldı; analyzed-* dizinleri taşınmadı. Fault cache'inde probe binary/fingerprint'leri taşınmadı ve paket temizliği yapıldı; doğru/fault kaynaklar ayrı hedeflerdedir. Ana model hedefleri analyzed-macro-edition-false/true altındadır. Yeni fixture için build/proc veya kullanıcı projesi kodu çalıştırılmaz; compiler testleri yalnız kendi iki tanığını çalıştırır. Önceki 12 testin regresyonu, yetkili eski yerel expansion fixture'ının build/proc kodunu kendi yeni target'ımızda çalıştırdı. Tüm çıktılar target/rust-macro-edition altındadır; taze bütün dependency build'i iddia edilmez.

## Teslim ve sonraki sınır

Tabandan yalnız **yeni bu belge, examples/macro_edition_gate.rs ve fixtures/macro-edition altındaki altı girdi** eklendi. Önceki 18 probe girdisi ve altı yeni minimal fixture girdisi önceki incelemeyle byte olarak aynıdır. Paketleme incelemesinden yalnız belge içeriği, kaynak yolu ve giriş noktası değişti. SHA256SUMS yeni belge ve tüm probe'un **26 girdisini**, REUSED-SHA256SUMS önceki regresyonda kullanılan **20 dış fixture girdisini** kapsar; target çıktıları hariçtir.

Bu sürümün mixed-edition kapısı başarısızdır. Sonraki öneri, bu küçük kaynak/typed gözlemle upstream sorunu ele almak ve yalnız ayrı yolculukta tutarlı bir RA sürümünde aynı borrowed beklentisini doğrulamaktır. **Doğrulanmış bir düzeltme sürümü yoktur; belirli sürümün çözdüğü iddia edilmez.** Crate edition'ını zorla değiştirmek iki farklı token bağlamını tek karara indirger; burada denenmedi/önerilmez. Token'ı yeniden yazmak, registry patch'i ve daha geniş tanı muafiyeti yapılmadı. Genel deney/nihai backend seçimi, kaynak eşleme/dedup, unresolved-call/inactive cfg ve gerçek ürün adaptörü açık kalır.
