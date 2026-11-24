# İlk Rust kaynak adaptörü dilimi

`crates/rust-source`, gerçek Cargo kaynağını mevcut `SourceReader` ve `TypeSource` portlarına bağlar. `RustProject::load(manifest, LoadOptions)` tek Cargo/rust-analyzer snapshot'ı yükler; iki port aynı snapshot'tan çıkarılan bildirimleri kullanır. `CheckProject` ve domain üretim kodu değişmez. Bu bir kütüphane API'sidir; CLI henüz Rust seçmez ve Docker imajı hâlâ Java analizi sunar.

## Doğrulanan davranış

`examples/rust-scenarios` normal kütüphane hedefinde S2/S5, `CheckProject` üzerinden sınanır:

- `PaymentService::on_parameter`, `core.rs:13`: `debit` izinli, gerekçe `parameter("wallet")`.
- `PaymentService::on_returned_chain`, `core.rs:29`: `wallet` izinli, `debit` tek ihlal; gerekçe `return_of("wallet")`.

Tip şekilleri gerçek alan ve metot bildirimlerinden gelir. Test sınıflandırma etiketi enjekte etmez. Ayrı RA testi, aynı snapshot'ta gerçek parametre bağını, `resolve_method_call` ile bildirim hedeflerini, alıcı ve sonuç tiplerini, referansın altındaki ortak Wallet kimliğini ve portların okuduğu imzaları sınar. Farklı adları ve ayrı modülü olan Processor/Parcel/Account kaynağı aynı ilişki ayrımını doğrular.

Sınıf kimliği Cargo paket kimliği, kütüphane hedefi ve bildirim modülü yolundan oluşur. Seçim `TypeSource::project_types()` içindeki tam adı mevcut `Target::method` API'sine verir. Metot seçimi katalog yüklemesini daraltmaz. Bütün workspace kütüphanelerinin proje kimlikleri önce yüklenir; çağrı raporu seçilen paketin normal kütüphanesinden çıkar. Bağımlılıklar RA Cargo modeliyle bulunur; target dizini bağımsız kaynak olarak taranmaz.

Hibrit sayısı doğrulanmaz. Erişimci/kaçan değişiklik yetkisi ve genel Rust sınıflandırması sonraki dilimlerdedir; bu dilimde alan/metot imzaları taşınır, getter/setter etiketleri çıkarılmaz. S2/S5'in davranış taşıyan Wallet ayrımı bu açık politikaları kapatmaz.

## Yükleme ve eksik analiz

`LoadOptions::new(package, absolute_output)` yalnız normal lib hedefini seçer. `with_features` ve `for_target`, Cargo özellik ve hedef yapılandırmasını RA modeline taşır. Normal bağlamda `test` ve `rust_analyzer` cfg bayrakları kapalıdır. Paket, edition, etkin cfg, kaynak kökleri ve bağımlılık bağlamı aynı veritabanında korunur. Proc macro client snapshot ömrü boyunca tutulur.

PATH'te uyumlu Cargo/rustc, eşleşen rust-src ve `libexec/rust-analyzer-proc-macro-srv` gerekir. Otomatik kurulum ve global Cargo ayarı yoktur. Metadata ve build komutları locked/offline çalışır; bağımlılık cache'i önceden hazır olmalıdır. Yükleme kullanıcının projesindeki build script ve proc macro kodunu çalıştırabilir.

Metadata, sysroot, build veya proc yükleme hatası `Result::Err(ReadError)` olur. Okunamayan modül ve parse hatası kaynak hatasına ve ilgili `AnalysisIssue` kaydına dönüşür. Çözülemeyen bildirim tipi type reading error olarak kalır; proje kimliği katalogdan silinmez. Çözülemeyen metot hedefi `TargetResolution` üretir; bilinen alıcı tipi izinli çağrı üretmek için yeterli değildir. Tip uyuşmazlığı `Compilation` issue'sudur.

İlk gövde köprüsü normal alıcılı metotlarda doğrudan parametre/self, gerçek literal ve normal metot zincirlerini taşır. Bütün gövde aktarılamazsa metot domain'e verilmez; sahibi ve konumu olan issue döner. Bu nedenle okunamayan gövdenin metot seçimi mevcut application sözleşmesinde ayrıca unmatched olabilir. Bilinen çağrıların kısmi gövdeden kurtarılması bu dilimin kapsamı değildir. Bağımsız fonksiyonlar ve desteklenmeyen aggregate bildirimleri açık issue taşır; modül sahte self yapılmaz.

Yereller, alan kökenleri, oluşturma, reborrow, genel kontrol akışı, trait/generic dispatch, closure/async, makro traversal/tekrar önleme, standart koleksiyon/sarmalayıcı politikaları ve Rust CLI/çıkış politikası sonraki işlerdir. Mevcut senaryo dosyasındaki bu tür etkin gövdeler boş Method/Literal/Operation ile temiz gösterilmez.

## Bağımlılık ve doğrulama kaydı

Dokuz doğrudan RA bağımlılığı exact `=0.0.358`; çözüm 27 RA paketi ve iki repo-relative vendor patch'i kullanır. Metadata beş ürün workspace üyesi, RA MSRV 1.98 ve probe ile aynı RA feature kümelerini doğrular. Vendor kaynakları değiştirilmez. Bu koşu Rust/Cargo 1.99.1 ile yapılır.

Kök lock ile probe lock birleştirilirken Cargo, aynı semver aralığında ayrı hashbrown 0.17.1/0.17.2 ve unicode-ident 1.0.27/1.0.24 kilitlerini reddetti. Probe'un hashbrown 0.17.2 ve gerekçeli unicode-ident 1.0.24 seçimi korundu. Salsa 0.28.5 ve unicode-properties 0.1.4 aynı kaldı. Eski Java lock'undaki diğer paket sürümleri ve doğrudan Java bağımlılıkları korunur. Metadata karşılaştırması `target/rust-source-journey/metadata-summary.json` içindedir.

İlk gerçek kırmızı, kaynak yüklemesi başarıyla bittikten sonra beklenen üç verdict yerine boş bulgu listesinde oluştu (`logs/bridge-red.log`). Minimal köprüyle aynı beklenti geçti (`logs/bridge-green.log`). Eksik Compilation/Read/Parse kayıtları ayrı davranış kırmızılarıyla yakalandı; struct hata konumu ilk testte yanlış yazılmıştı, kaynağın attribute başlangıcı olan satır 27'ye düzeltildi. Bu konum düzeltmesi yeni davranış kırmızısı sayılmaz.

Koşuların log'ları worktree'nin `target/rust-source-journey/logs` dizinindedir. Tüm kaynak→rapor testleri gerçek Cargo/RA yüklemesini çalıştırır; önkoşul yokluğunda test atlanmaz.

Geçici bozulma kontrolleri gerçek assertion'larda kırıldı: referansı unit tipe indirgeme (snapshot imzası ve iki kaynak→rapor karşılaştırması), olmayan metodu yalnız alıcı bilindiği için kabul etme, type error'ı gizleme, desteklenmeyen gövdeyi boş metot yapma, Rust adaptörüne Java adaptörü bağımlılığı ekleme ve Rust dil adını mimari filtreden çıkarma. Bütün değişiklikler geri alındı. İlk eksik-hedef bozulma denemesinin derleme hatası davranış kanıtı sayılmadı; düzeltilen deneme `missing_debit` için yanlış `Allowed` bulgusunu üreterek gerçek assertion'da kırıldı (`missing-target-fault-corrected.log`).

Son doğrulama (24 Kasım 2026, macOS ARM64; bu worktree ve yeni Cargo.lock):

| Komut | Gerçek sonuç |
|---|---|
| `cargo test --workspace --offline --locked` | 1257 geçti, 0 hata, 5 ignored. Rust bölümü 1 snapshot + 12 kaynak→rapor testi; mevcut Java testleri de aynı koşuda geçti. |
| `cargo fmt --all --check` | exit 0. |
| `cargo clippy --workspace --all-targets --offline --locked -- -D warnings` | exit 0. |
| `docker build --progress=plain -t law-of-demeter:rust-source-slice .` | exit 0; vendor COPY ve kısıtlı build context doğrulandı. |
| `CONTAINER_IMAGE=law-of-demeter:rust-source-slice cargo test --offline --locked -p law-of-demeter-cli --test container -- --ignored` | 3/3 geçti; Java çıkış kodları 0/1/2 korundu. |

Koşular `CARGO_TARGET_DIR=target/rust-source-journey` mutlak yoluyla, dev/test debug 0, incremental 0, jobs 1 ve test threads 1 ile çalıştı. Java denetimleri mevcut JDK 21 ve Maven yollarını yalnız süreç ortamından aldı. Workspace'teki beş ignored testin üçü yukarıdaki açık Docker koşusunda çalıştırıldı; iki ignored Maven örnek testi tekrar çalıştırılmadı. `verification.json` exact komut/çıkış/süre kayıtlarını tutar. Test okunabilirliği ayrı salt okunur geçişte incelendi; bulgu yok. Bağımsız nihai incelemeyi koordinatör atayacak.
