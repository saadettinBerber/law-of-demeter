# Rust backend deneyi: generic, trait ve dyn

**RA 0.0.358 bu örneklerde generic projeksiyonu, somut çağrı tipini ve trait/impl bildirimlerini ayırdı.** Önceki beş test ile üç yeni Semantics testi geçti. Bu yalnız [generic/trait/dyn geçiş koşuludur](rust-backends.md#sonraki-sınırlı-öğrenme-deneyi); genel deney satırı ve nihai backend seçimi açık kalır.

Taban `00294c5b1238cbf9c08eea085b5b19bd29db15a0`, tarih 23 Kasım 2026. TypeSource/SourceReader, CheckProject raporu, LoD köken/istisna veya Value/Hybrid uygulaması yoktur.

## Girdi ve gerçek cfg bağlamı

Yeni [`fixtures/traits/Cargo.toml`](../examples/rust-backend-probe/fixtures/traits/Cargo.toml), edition **2024** bağımsız pakettir. İki `[[test]] path` doğrudan onaylı [`selection.rs`](../examples/rust-method-resolution/selection.rs) ve [`helpers.rs`](../examples/rust-visibility/helpers.rs) dosyalarına gider. Kaynaklar kopyalanmadı/değiştirilmedi; ikinci parser veya elle test attribute silme yoktur. Yeni fixture dış bağımlılık, lib, build.rs veya proc-macro taşımaz.

Analiz edilen üç yeni test bağlamında yükleyiciye `set_test=true` verilir. Her test gerçek `Crate::cfg` içinde `test=true`, `rust_analyzer=false`, edition 2024 ve seçtiği kaynak `#[test]` fonksiyonunun HIR'da `is_test=true` olduğunu denetler. Önceki beş testin analiz ettiği normal lib snapshot'ları **`set_test=false`** kalır; önceki cfg/unknown/kimlik/kaynak kontrolleri yeniden geçti. Probe'un kendi Cargo test cfg'si ile analiz edilen snapshot'ın cfg'si farklı girdilerdir.

## Bağımsız beklenti ve gözlenen typed sonuçlar

[`tests/traits.rs`](../examples/rust-backend-probe/tests/traits.rs) beklenen Function, Trait, TypeParam ve ilişkili tip kimliklerini aynı DB'nin kaynak bildirimlerinden `Semantics::to_def` ile alır. Adlar yalnız fixture çapasıdır; gerçek hedef/tip sorgusunun yerine geçmez. Beklenen tip enjekte edilmez, display/hover/debug/MIR metni ayrıştırılmaz.

| Sınanan ilişki | Gerçek sorgu sonucu ve bağımsız kontrol |
|---|---|
| Generic gövdede `source.quote()` | `resolve_method_call`, **Quote::quote** bildirimi olur; inherent QuoteSource::quote değildir. Çağrı sonucu, generic fonksiyonun dönüşü ve T ile Output bildirimi üzerinden sorgulanan projeksiyon aynı typed değerdir. Parent trait Quote, içerdiği generic parametre yalnız kaynak **T**'dir; unknown içermez ve `u32` değildir. Kaynak çağrı `selection.rs:218`. |
| Somut `quote_from_bound(&source)` | Gerçek çağrı ifadesinin çıkarılmış tipi **u32**'dir; beklenti `generic: u32` annotation'ından değil, bağımsız **QuoteSource.amount: u32** alanının HIR tipinden gelir. Callee kimliği quote_from_bound, instantiated ilk parametrenin referent ADT'si gerçek QuoteSource'tur. Kaynak çağrı `selection.rs:65`. |
| Callable dönüş imzası / normalize sonuç | Somut callee'nin `Callable::return_type()` sonucu bu örnekte hâlâ Quote'a ait **ilişkili tip projeksiyonu** taşır; artık T içermez. Actual parametreden bulunan QuoteSource üzerinde kaynak Output alias'ını `normalize_trait_assoc_type` ile sorgulamak u32 verir; bu sonuç gerçek CallExpr tipiyle eşittir. Ham imza ile çıkarılmış çağrı sonucu karıştırılmaz. |
| `source.quote()` ve `QuoteSource::quote(&source)` | İkisi aynı inherent Function kimliğine çözülür. Noktalı callable bound receiver verir, sıradan parametre sayısı 0'dır. Tip yolu callable'ı unbound gösterir: `receiver_param=None`, fakat kaynak Function'da **self vardır**; parametre 0'ın binding'i aynı SelfParam kimliğidir. Gerçek `&source` argümanının tipi instantiated self tipiyle eşittir ve QuoteSource'a bağlanır. |
| `<QuoteSource as Quote>::quote(&source)` | `resolve_path` somut **impl Quote for QuoteSource::quote** Function kimliğini verir. Aynı callee'nin `Callable::as_function()` değeri ise **Quote::quote trait bildirimi**dir. İkisi kaynakta ayrı kimliklerdir. Callable parametre 0 yine gerçek self'tir; `receiver_param=None` olması alıcısızlık değildir. |
| `QuoteSource::forwarded_wallet(&wallet)` kontrolü | Ayrı Function kimliği, **self yok**, bound receiver yok; tek parametrenin binding'i sıradan argümandır. Yol yazımı diğer iki `::quote` çağrısıyla aynıdır ama imza ilişkisi farklıdır. Buradan factory veya genel argüman-alias köken kuralı çıkarılmaz. |
| `from_public_source` içindeki dyn çağrı | Gerçek alıcı unknown içermeyen **`&dyn WalletSource`**; hedef/callable kimliği WalletSource::selected trait bildirimidir. Bu bildirimin default gövdesi ve **Replaced::selected override gövdesi** kaynakta bulunur ve Function kimlikleri farklıdır. Çağrı `helpers.rs:109`, override bildirimi `:192` doğrulandı. |

Dyn sorgusunun trait bildirimi vermesi, default gövdenin kesin runtime hedefi olduğunu göstermez. Aynı kaynakların bu dilimde yeniden çalışan runtime testleri, Replaced üzerinden `from_public_source` çağrısının **other Wallet** referansını seçtiğini doğrular. Bu soyut dyn parametreden tek concrete gövde çıkardığımızı iddia etmiyoruz; lexical gövdenin bulunması, kesin dispatch hedefi ve dönüş kökeni ayrı bilgilerdir.

## API sınırı ve kaynak kontrolü

Kullanılan typed yüzeyler: `type_of_expr`, `resolve_method_call`, `resolve_expr_as_callable`, `resolve_path`; `Callable::{as_function,receiver_param,params,return_type}`; `Function::self_param`, `Param::as_local`, `Local::as_self_param`; `Type::{as_associated_type_parent_trait,generic_params,normalize_trait_assoc_type,contains_unknown}`.

RA **0.0.358** için [HIR](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/lib.rs), [Semantics](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/semantics.rs) ve [source_analyzer](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/source_analyzer.rs) dosyaları sürümlü primary kaynakla byte olarak eşleşti. Kaynakta `receiver_param` bound-method koşuluyla sınırlıdır; method-call-as-callable için doğru trait impl'ini çözmediği uyarısı da vardır. Gözlenen açık trait yolu ayrımı, yalnız callable kimliğini implementation/gövde kimliği olarak kullanmamayı gerektirir.

Bu dilim genel substitution haritası veya tam generic solver API'si üretmez: yerine koyma, gerçek callable parametreleri ve ilişkili tip sorgusuyla bu küçük örnekte doğrulandı. `Type` owner bağlamını korur; karşılaştırmalar aynı DB'nin `attach_db` kapsamındadır. Normalizasyon implementasyonu GAT için FIXME içerir; fixture GAT/const/HRTB karmaşıklığını sınamaz. Ham Function/Callable dönüşünü körlemesine somut rapor tipi saymak yerine gerçek ifade tipi ve gereken normalizasyon ayrı okunmalıdır. Bunlar adaptör gözlemleridir; yeni domain sözleşmesi veya büyük çağrı şeması eklenmedi.

## Helper taşıması ve korunan kurulum

Ortak yükleme, DB/VFS/kök dosya seçimi, parser hata kontrolü, fiziksel kaynak/satır bağı ve bildirim/çağrı çapaları **`tests/support/mod.rs`** içine taşındı. Yalnız test modüllerinin kullanımı için `pub(super)` görünürlüğü vardır. Normal-lib'e özgü modül/tanı yardımcıları semantics.rs'de, trait'e özgü çapalar traits.rs'de kaldı; genel harness/CLI/rapor yoktur.

İki gerekli değişiklik: loader `set_test` girdisini açık alır; manifest dışındaki `[[test]] path` için kaynak yolu canonicalize edilir. Eski iki identity çağrısına `false` eklendi; eski beş testin diğer gövdeleri/assertion'ları başlangıç metniyle aynıdır. Satır denetimi aynı fiziksel-yol yardımcısını kullanır; 13/29 ve önceki identity kontrolleri korunur.

Explicit sysroot/rust-src, `rust_analyzer` override, locked/offline metadata, kapalı build/proc ve cache ön doldurma ayarları değişmedi. Yeni bağlamların tüm yerel modüllerinde `Module::diagnostics(..., false)` çıktısı boştur; bu genel model/stdlib tamlığı veya compiler'ın bütün tanıları değildir. Eksik hedef/tip/bildirim/kaynak ve unknown sorguları testi kırar. Yeni toolchain/bileşen/global ayar değişikliği yoktur.

Rust/Cargo **1.99.1**, host `aarch64-apple-darwin`, rustc commit'i `58cf8f9c35b355199727627d07ae34a4e2846d67`; önce/sonra kimlik aynıdır. Probe Cargo.toml/Cargo.lock byte olarak değişmedi: sekiz doğrudan RA pini **=0.0.358**, metadata'da **179 paket / aynı sürümde 27 ra_ap paketi**; Salsa ailesi **0.28.5**, unicode-properties **0.1.4**, unicode-ident **1.0.24** korunur. Yeni fixture kilidi tek yerel pakettir, yeni dış dependency yoktur.

Önceki build target'ı kopyalanmadı/kullanılmadı; mevcut registry kaynak cache'i kullanıldı. Bütün çıktılar worktree'nin **target/rust-backend-traits** altındadır; Cargo dizinleri `cargo` ve `fault-cargo`, RA için ayarlanan yol ilgili Cargo dizininin `analyzed` altıdır. Analiz edilen fixture için build/proc çalıştırılmadı. Kaynak Cargo testleri kendi kontrollü runtime kodunu, probe derlemesi normal dependency build/proc adımlarını çalıştırır. Bu, proc ABI veya macro/build desteği kanıtı değildir.

## Exact koşular ve gerçek sonuç

Repo kökünden çalıştırılan komutlar:

```sh
CARGO_TARGET_DIR="$PWD/target/rust-backend-traits/cargo" cargo generate-lockfile --offline --manifest-path examples/rust-backend-probe/fixtures/traits/Cargo.toml
CARGO_TARGET_DIR="$PWD/target/rust-backend-traits/cargo" RUSTFLAGS='-D warnings' cargo test --locked --offline --manifest-path examples/rust-backend-probe/fixtures/traits/Cargo.toml --tests
CARGO_TARGET_DIR="$PWD/target/rust-backend-traits/cargo" RUSTFLAGS='-D warnings' cargo check --locked --offline --manifest-path examples/rust-backend-probe/fixtures/traits/Cargo.toml --tests
CARGO_TARGET_DIR="$PWD/target/rust-backend-traits/cargo" RUSTFLAGS='-D warnings' cargo test --locked --offline --manifest-path examples/rust-backend-probe/Cargo.toml --tests -- --test-threads=1
CARGO_TARGET_DIR="$PWD/target/rust-backend-traits/cargo" cargo metadata --locked --offline --format-version 1 --manifest-path examples/rust-backend-probe/Cargo.toml
CARGO_TARGET_DIR="$PWD/target/rust-backend-traits/cargo" RUSTFLAGS='-D warnings' cargo clippy --locked --offline --manifest-path examples/rust-backend-probe/Cargo.toml --all-targets -- -D warnings
CARGO_TARGET_DIR="$PWD/target/rust-backend-traits/cargo" RUSTFLAGS='-D warnings' cargo clippy --locked --offline --manifest-path examples/rust-backend-probe/fixtures/traits/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path examples/rust-backend-probe/Cargo.toml --check
cargo fmt --manifest-path examples/rust-backend-probe/fixtures/traits/Cargo.toml --check
```

Kaynak Cargo testleri **17/17** (helpers 8, selection 9); source check geçti. Probe ilk başarılı koşuda **5+3=8/8**, locked/offline tekrarda **8/8** geçti. İki Clippy ve iki fmt check geçti; lint istisnası/skipped test yoktur. İlk API derlemesindeki Field::name DB argümanı ve HasArgList import hataları düzeltildi; bunlar davranış assertion'ı veya ürün TDD kırmızısı değildir. Geniş core/workspace/Maven ve önceki tüm dil deneyleri çalıştırılmadı; dört-edition veya benchmark iddiası yoktur.

### İlk yeşil öğrenme testlerinin fault kontrolü

Geçici paket `target/rust-backend-traits/fault-layout/examples/rust-backend-probe` altındadır. Manifest/lock/testler kopyalandı; `fixtures` ve yan `rust-scenarios` symlink'leri değişmeyen gerçek girdilere gider. Beklenen bildirim/tip değerleri ve fixture kaynakları sabit tutuldu; yalnız gerçek sorgu/seçim glue'su değiştirildi:

| Varyantlar | Yanlış gerçek sorgu; yakalanan bağımsız ilişki |
|---|---|
| generic-target | Generic quote hedefi yerine concrete inherent quote sorgulandı; trait Function kimliği beklentisi kırıldı. |
| generic-projection | Generic gövde sonucu yerine somut çağrı tipi sorgulandı; T::Output beklentisi kırıldı. |
| concrete-result | Somut generic çağrı yerine inherent quote ifadesinin tipi sorgulandı; u32 beklentisi kırıldı. |
| dot-target, type-target, trait-target | Noktalı çağrı generic trait hedefine; iki qualified yol birbirinin hedefine yöneltildi. Inherent/impl Function kimlikleri ayrımı kırıldı. |
| path-self | Parametre 0'ın gerçek self binding'i yerine unbound callable.receiver_param kullanıldı; None ile kaynak SelfParam kimliği ayrımı kırıldı. |
| receiverless-target | forwarded_wallet callee sorgusu yerine type-qualified quote callee'si seçildi; alıcısız fonksiyon kimliği beklentisi kırıldı. |
| dyn-target | Dyn çağrının bildirimi yerine override'ın concrete bildirimi sorgulandı; trait bildirimi beklentisi kırıldı. |
| dyn-receiver | Dyn receiver yerine selected çağrısının &Wallet sonuç tipi sorgulandı; dyn trait tipi beklentisi kırıldı. |

**10/10 varyant derlendi, runtime assertion ile çıkış 101 verdi.** Her logda çalışmış traits testi ve 0 passed / 1 failed doğrulandı; compiler hatası yoktu. `test_name` ilgili üç testten birinin tam adıdır:

```sh
CARGO_TARGET_DIR="$PWD/target/rust-backend-traits/fault-cargo" RUSTFLAGS='-D warnings' cargo test --locked --offline --manifest-path target/rust-backend-traits/fault-layout/examples/rust-backend-probe/Cargo.toml --test traits "$test_name" -- --exact
```

Doğru traits.rs geri kondu; kopyada aynı manifestle `--tests -- --test-threads=1` **8/8 geçti**. Varyant kaynak/logları `target/rust-backend-traits/<varyant>.{rs,log}` altındadır. Beklenti değiştirmek veya geçersiz Rust üretmek red kanıtı yapılmadı; bu dependency öğrenmesidir, ürün analiz TDD'si değildir.

Test okunabilirliği ayrı, salt okunur geçişte incelendi; bulgu yok. AAA blokları, kaynak çapalarından gelen bağımsız kimlikler ve karşı çağrılar görünürdür; her test tek ilişki grubunu sınar. İnceleme kaynak değiştirmedi.

## Değişen girdiler ve açık kapsam

Başlangıçtan değişen/yeni altı dosya: **docs/rust-backend-traits.md**, değiştirilmiş **examples/rust-backend-probe/tests/semantics.rs**, yeni **tests/traits.rs**, **tests/support/mod.rs**, **fixtures/traits/Cargo.toml**, **fixtures/traits/Cargo.lock** (son dört yol probe dizinine göredir). Eski fixture'lar, önceki belgeler, kök Cargo/production/domain/decision.md değişmedi.

Teslim `target/rust-backend-traits/SHA256SUMS`, yeni belge ve bütün probe kaynak/manifest/lock girdilerini kapsar. Yeniden kullanılan dış girdiler ayrı **REUSED-SHA256SUMS**'tadır: rust-scenarios Cargo.toml/Cargo.lock/core.rs ve selection.rs/helpers.rs. Son iki SHA-256: selection `902b4d92412401623c7f953992d068b4637f76f737a9cc145c14e682796ab800`, helpers `0e01bf14d9dec445eee59be8088e4512a1b431990382b0ea1e642548f91dcb1c`; beş kayıt yeniden doğrulandı.

Bu dilimde somut backend engeli kalmadı; destek yalnız bu typed sorgu örnekleri için doğrulandı. Genel generic/trait kapsamı, GAT/HRTB/const ve çoklu dispatch bütünlüğü tamamlanmadı. Macro/build/proc ABI/expansion kaynak eşleme ve etkin unresolved-call negatifi sonraki dilimlerdir. Gerçek adaptör, rapor/çıkış politikası, LoD/Value/Hybrid ve diğer açık ürün kararları bu sonuçla kapanmaz.
