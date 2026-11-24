# Rust backend deneyi: crate kimliği

**Gerçek provider/consumer sınırında re-export edilen ve private modülden dönen Wallet, RA 0.0.358 ile doğru provider bildirimlerine çözüldü.** Consumer'ın aynı adlı tipi/metodu ayrı kimliklerdir. İki yeni öğrenme testi ve önceki üç test birlikte geçti. Bu yalnız [crate kimliği geçiş koşulunu](rust-backends.md#sonraki-sınırlı-öğrenme-deneyi) sınar; genel deney satırı ve nihai backend seçimi açık kalır.

Taban `c9f461e5300b625191f54937618905e834351792`, tarih 23 Kasım 2026. Ürün adaptörü, TypeSource/SourceReader, CheckProject raporu veya LoD/Value/Hybrid kararı uygulanmadı.

## Küçük kaynak ve bağımsız beklenti

[`fixtures/identity`](../examples/rust-backend-probe/fixtures/identity/Cargo.toml), yalnız iki yerel edition **2024 lib** paketinden oluşur; dış bağımlılık, feature, build.rs veya proc-macro yoktur. Ayrı workspace ve kilit dosyası vardır.

| Kaynak olgusu | Gerçek sorguda denetlenen |
|---|---|
| Provider'ın private `exposed` modülündeki public Wallet, `pub use exposed::Wallet` ile açılır. Consumer, Cargo bağımlılığını `ledger` adıyla alır ve `pub use ledger::Wallet as ImportedWallet` yazar. | `debit_imported` alıcısı unknown içermeyen **`&mut` provider Wallet**; hedef provider Wallet::debit kimliğidir. `Crate::dependencies` içindeki `ledger` kenarı aynı provider Crate kimliğine gider. |
| Consumer kendi `exposed::Wallet` tipini ve aynı imzalı `debit` metodunu taşır. İki crate'in metot tanım dosyalarının adı da **`exposed.rs`**'dir. | `debit_local` alıcısı consumer Wallet, hedefi consumer Wallet::debit olur. İki ADT (tip bildirimi) ve Function kimliği farklıdır; gerçek kaynak yolları provider/consumer dosyalarını ayırır. Aynı dosya adı ayrıca doğrulanır. |
| Provider'ın ayrı private `hidden` modülündeki public Wallet re-export edilmez. Public `hidden_wallet()` onu değer olarak döndürür. | Consumer'daki çağrı yolu gerçek provider fonksiyonuna bağlanır. Çağrının sonuç tipi ve `debit_returned` içindeki yerel alıcı, **aynı hidden Wallet ADT** kimliğidir; exposed Wallet ve consumer Wallet kimliklerinden farklıdır. |
| Consumer dönen değerin public `balance` alanına yazar ve public `debit` metodunu çağırır; private tip yolunu yazmaz. | `debit` hedefi hidden Wallet::debit olur. Gerçek alıcı tipinin struct tanımı ve seçilmiş metodun tanımı, provider'ın **`src/hidden.rs`** dosyasına ve provider crate'ine bağlıdır. |

Normal `cargo check --locked --offline` iki lib'i, beş Rust kaynak dosyasını modülleriyle derledi. Alan/metot erişimi bu nedenle geçerli kaynak olgusudur; runtime bakiye veya LoD sonucu testi değildir. Adı dışarıdan yazılamayan public tipin değer üzerinden kullanımı [onaylı görünürlük araştırmasında](rust-visibility.md) ayrıca compiler kanıtıyla açıklanır; o negatif deneyler burada yeniden çalıştırılmadı.

Beklenen kimlikler aynı DB'deki provider/consumer kaynak bildirimlerinin `Semantics::to_def` sonuçlarından gelir. Çağrıların gerçek tip/hedef sorguları ayrı yoldur; beklenen tip enjekte edilmez. Kaynak adları yalnız fixture çapalarını seçer. HIR kimlikleri, `Module::krate`, gerçek kök dosyaları ve VFS yolları karşılaştırılır; display/debug/hover/MIR metni ayrıştırılmaz.

Ek kullanılan API'ler RA **0.0.358**'in [HIR kaynağında](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/lib.rs) (`Crate::dependencies`, `Module::krate`) ve [Semantics kaynağında](https://raw.githubusercontent.com/rust-lang/rust-analyzer/2ad81e72b7e978cb800dffe1bcce5c9f98032acf/crates/hir/src/semantics.rs) (`module_definition_node`, `source`, `resolve_path`) kontrol edildi. Bu iki dosya kayıtlı commit'in primary içeriğiyle yerel registry kaynağında byte olarak eşleşti. Public API olması sürümler arası kararlılık iddiası değildir.

## Korunan bağlam ve hata sınırı

Probe manifesti ve Cargo.lock başlangıçla byte olarak aynıdır. Sekiz doğrudan RA pini **`=0.0.358`**; gerçek locked/offline metadata yine **179 paket**, tamamı 0.0.358 olan **27 `ra_ap_*` paketi** içerir. Önceki Salsa **0.28.5** ailesi, unicode-properties **0.1.4** ve unicode-ident **1.0.24** seçimleri korunur. Yeni fixture'ın kilidi yalnız iki yerel paketi içerir. Yeni doğrudan RA/dış bağımlılık yoktur.

Rust/Cargo **1.99.1**, host `aarch64-apple-darwin`, rustc commit'i `58cf8f9c35b355199727627d07ae34a4e2846d67`. Önceki dilimde kurulmuş rust-src kullanıldı; bu dilimde bileşen/toolchain/global ayar değiştirilmedi. Önce/sonra compiler kimliği aynı bulundu.

Mevcut explicit sysroot ve kaynak yolu, `set_test=false`, `rust_analyzer` override, `--locked --offline` metadata, kapalı build/proc ve cache ön doldurma sınırları korundu. Yeni testler iki crate'in edition/cfg bilgilerini gerçek modelden denetler. Her iki crate'in tüm yerel modüllerinde `Module::diagnostics(..., false)` çıktısı boştur; bu compiler'ın bütün tanıları veya bütün std gövdeleri için tamlık garantisi değildir.

Yükleyici hataları bağlamıyla döner; eksik crate/modül/bildirim/hedef/tip/kaynak aralığı testi başarısız yapar. Sorgulanan alıcı/sonuç tiplerinin unknown içermediği denetlenir. Etkin unresolved-call negatifinin hata sözleşmesi hâlâ sonraki dilimdir. Analiz sırasında yalnız bu fixture'lar ve std bağlamı yüklendi; analiz edilen proje için build/proc çalıştırılmadı.

Önceki target dizini/build cache'i kullanılmadı. Bu dilimin derleme/metadata/fault çıktıları yalnız worktree'nin **`target/rust-backend-identity`** altındadır; mevcut Cargo registry kaynak cache'i kullanıldı. Yayınlanmış probe bağımlılıkları yeni hedef dizinlerinde derlendi; dependency build/proc adımları normal Cargo derlemesinin parçasıdır. Cache tekrarı fresh build veya performans karşılaştırması diye sunulmaz.

## Exact doğrulama

Repo kökünden çalıştırıldı:

```sh
CARGO_TARGET_DIR="$PWD/target/rust-backend-identity/cargo" cargo generate-lockfile --offline --manifest-path examples/rust-backend-probe/fixtures/identity/Cargo.toml
CARGO_TARGET_DIR="$PWD/target/rust-backend-identity/cargo" RUSTFLAGS='-D warnings' cargo check --locked --offline --manifest-path examples/rust-backend-probe/fixtures/identity/Cargo.toml --workspace --lib
CARGO_TARGET_DIR="$PWD/target/rust-backend-identity/cargo" RUSTFLAGS='-D warnings' cargo test --locked --offline --manifest-path examples/rust-backend-probe/Cargo.toml --test semantics -- --test-threads=1
CARGO_TARGET_DIR="$PWD/target/rust-backend-identity/cargo" cargo metadata --locked --offline --format-version 1 --manifest-path examples/rust-backend-probe/Cargo.toml
CARGO_TARGET_DIR="$PWD/target/rust-backend-identity/cargo" RUSTFLAGS='-D warnings' cargo clippy --locked --offline --manifest-path examples/rust-backend-probe/Cargo.toml --all-targets -- -D warnings
CARGO_TARGET_DIR="$PWD/target/rust-backend-identity/cargo" RUSTFLAGS='-D warnings' cargo clippy --locked --offline --manifest-path examples/rust-backend-probe/fixtures/identity/Cargo.toml --workspace --all-targets -- -D warnings
cargo fmt --manifest-path examples/rust-backend-probe/Cargo.toml --check
cargo fmt --manifest-path examples/rust-backend-probe/fixtures/identity/Cargo.toml --all --check
```

Sonuç: fixture check geçti; ilk test koşusu **5/5**, küçük isim/hata-bağlamı refactor'undan sonraki aynı locked/offline test komutu **5/5** geçti. İki Clippy ve iki fmt check geçti; lint istisnası veya skipped test yoktur. Bağımlılıklar önceden edinilmişti; yeni paket güncellemesi/edinimi gerekmedi. Geniş workspace/core/Java/Maven ve önceki tüm dil deneyleri çalıştırılmadı.

İlk dilimin üç test fonksiyonu başlangıç commit'indeki metinle byte olarak aynıdır: `the_parameter_call_resolves_to_the_wallet_declaration`, `the_returned_chain_resolves_the_order_method_and_wallet_result`, `the_normal_library_excludes_test_items_without_removing_their_source`. S2/S5 unknown, 13/29 satırları, cfg ve inactive-source denetimleri birlikte yeniden geçti. Yükleyiciye yalnız fixture dizini/kök kaynak parametresi eklendi; özel crate/kaynak çapası yardımcıları aynı dosyada paylaşıldı. Genel harness/CLI/rapor yazılmadı.

### İlgili geçici bozulmalar

İlk yeşil bağımlılık gözlemlerinden sonra probe, fixture'ıyla birlikte `target/rust-backend-identity/fault-layout/examples/rust-backend-probe` altına kopyalandı. Yanındaki `rust-scenarios` symlink'i eski değişmeyen girdiye gider. Beklentiler ve fixture kaynakları değiştirilmeden her varyantta yalnız yeni testin target sorgusu bozuldu:

| Exact test | Gerçek sorgudaki bozulma ve gözlenen sonuç |
|---|---|
| `a_reexported_wallet_keeps_its_provider_identity_across_a_dependency_alias` | `resolve_method_call(&imported_call)` yerine `resolve_method_call(&local_call)`: aynı adlı consumer debit, provider debit kimliği beklentisinde kırıldı. |
| `a_returned_unnameable_wallet_keeps_its_provider_identity` | Debit hedefi için consumer `debit_local` gövdesindeki aynı adlı çağrı sorgulandı: hidden provider debit kimliği beklentisinde kırıldı. Sonuç/alıcı tipi sorguları değiştirilmedi. |

İkisi de **derlendi ve kimlik assertion'ı ile çıkış 101** verdi; compiler/setup hatası değildir. `test_name` ilgili tam ad olmak üzere kullanılan komut:

```sh
CARGO_TARGET_DIR="$PWD/target/rust-backend-identity/fault-cargo" RUSTFLAGS='-D warnings' cargo test --locked --offline --manifest-path target/rust-backend-identity/fault-layout/examples/rust-backend-probe/Cargo.toml --test semantics "$test_name" -- --exact
```

Doğru test kaynağı kopyaya geri kondu; aynı komut exact filtre olmadan `-- --test-threads=1` ile **5/5 geçti**. Kopya son durumda sahip olunan test kaynağıyla byte olarak aynıdır. Varyantlar/loglar `target/rust-backend-identity/{reexport-target,returned-target}.{rs,log}` altındadır. Bunlar dependency öğrenme assertion'larının etkinliğidir; ürün TDD kırmızısı veya LoD analiz doğrulaması değildir.

Test okunabilirliği ayrı salt okunur geçişte incelendi; bulgu yok. İsimler beklenen ilişkiyi söyler, kur/işlet/denetle blokları ayrıdır; karar veren bilgi kaynak kimlikleri ve aynı adlı karşı örneklerdir. İnceleme kaynak değiştirmedi.

## Değişen girdiler ve teslim

Başlangıç tabanından değişen dosyalar: yeni **`docs/rust-backend-identity.md`**, değiştirilmiş **`examples/rust-backend-probe/tests/semantics.rs`** ve aşağıdaki dokuz yeni dosya. Dokuzunun ortak öneki `examples/rust-backend-probe/fixtures/identity/`'dir:

```text
Cargo.toml
Cargo.lock
provider/Cargo.toml
provider/src/lib.rs
provider/src/exposed.rs
provider/src/hidden.rs
consumer/Cargo.toml
consumer/src/lib.rs
consumer/src/exposed.rs
```

`target/rust-backend-identity/SHA256SUMS`, bu belgeyi ve probe projesinin tüm kaynak/manifest/lock girdilerini kapsar; başlangıçtan değişmeyen probe Cargo.toml/Cargo.lock da içindedir. Yeniden kullanılan dış fixture'ın üç girdisi ayrı `REUSED-SHA256SUMS`'tadır; teslim manifestine katılmaz. SHA-256 değerleri: rust-scenarios Cargo.toml `f71e31b7dd2a3ab99b00e2f214a79db7a0a551277719cc59aec4514b32c0fa3f`, Cargo.lock `c2ec775831b47a2f06c6cc124d7f6da709ce6efe26891437dc4ac0e406398160`, core.rs `26d4efdfb74120a81822feb55a19d1e5fd7acd6bcda75e5e101270c770029f70`. Eski fixture, önceki belgeler, kök Cargo dosyaları, üretim/domain ve decision.md değişmedi.

Somut engel kalmadı; kapsam yalnız bu iki crate/normal lib örneğidir. Paket sürüm çakışmaları ve diğer target/edition çeşitleri sınanmadı. Generic/ilişkili tip/trait/dyn, makro/build/proc ABI/açılım kaynak eşlemesi ve etkin unresolved-call negatifi sonraki dilimlerdir. Value/Hybrid, LoD köken/politika kararları ve gerçek adaptör→CheckProject akışı açık kalır; bu sonuç nihai backend seçimi değildir.
