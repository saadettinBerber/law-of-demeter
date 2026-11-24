# Rust backend sürüm adayı: uygunluk engeli

**Bu sınırlı araştırmada doğru mixed-edition lookup için güçlü, tutarlı yayımlanmış bir RA adayı belirlenemedi.** Dokuz doğrudan API crate'inin en yeni ortak sürümü hâlâ **0.0.358**; bu sürümün borrowed kabulü [önceki deneyde](rust-macro-edition.md) başarısızdır. Daha yeni HIR **0.0.363** tam aile değildir ve kaynakta aynı edition yolu vardır. Probe yükseltilmedi; nihai pinler **=0.0.358** kaldı.

Taban `7bab3f97a85f94c90584e34dab99eb4b32b7247c`, araştırma 23 Kasım 2026. Yalnız bu belge eklendi. Production, önceki belgeler, fixture'lar, Cargo dosyaları, dar legacy tanı istisnası ve decision.md değişmedi. Başarılı bir yeni sürüm veya nihai backend seçimi kanıtı yoktur.

## Güncel paket uygunluğu

Dokuz paketin crates.io API'sindeki bütün sürüm metadata'sı yeniden alındı; yanked sürümler elenip ortak sürüm kesişimi çıkarıldı. Bu, önceki araştırmanın tarihsel bulunamama sonucunun tekrarı değildir. [Loader metadata'sı](https://crates.io/api/v1/crates/ra_ap_load-cargo), [project-model](https://crates.io/api/v1/crates/ra_ap_project_model) ve [proc API](https://crates.io/api/v1/crates/ra_ap_proc_macro_api) son yayımlanmış sürüm olarak 0.0.358 gösterir.

| Doğrudan crate'ler | En yeni bulunan sürüm | 0.0.363 durumu |
|---|---|---|
| ra_ap_cfg, ra_ap_intern, ra_ap_syntax, ra_ap_vfs | 0.0.364 | Mevcut, MSRV 1.98 |
| ra_ap_hir, ra_ap_ide_db | 0.0.363 | Mevcut, MSRV 1.98 |
| ra_ap_load-cargo, ra_ap_project_model, ra_ap_proc_macro_api | 0.0.358 | Üçü de yok: version API **404**, cargo info **exit 101** |

En yeni ortak 0.0.358 ailesinin dokuz MSRV kaydı **1.98**'dir; mevcut Rust **1.99.1** bu alt sınırı karşılar. MSRV metadata'sı transitive çözüm/build veya proc ABI kanıtı değildir. 0.0.363 hir_ty manifestindeki alias'lar package alanıyla çözüldü: **sekiz RA dependency kenarının tamamı =0.0.363** ister. Eski loader ile yeni HIR birleştirilmedi; bu, gereken aynı-aile koşulunu sağlamaz.

## İzole çözüm ön kontrolü

`target/rust-backend-version/availability-0.0.363/probe` gerçek probe'un kopyasıdır. Yalnız dokuz doğrudan pin =0.0.363 yapıldı; macro edition fixture'ı ve borrowed assertion kaynağı byte olarak aynı kaldı. Başlangıç Cargo.lock'u ayrı baseline-Cargo.lock olarak saklandı, kopyanın kök lock'u kaldırıldı; eski lock yeni adayın çözümü gibi sunulmadı.

Gerçek **cargo generate-lockfile**, crates.io index'inde `ra_ap_load-cargo = "=0.0.363"` bulamayarak **exit 101** verdi. **Aday lockfile oluşmadı, build/runtime yapılmadı.** Bu bir uygunluk reddidir; seçilmiş güçlü aday veya ürün TDD kırmızısı değildir. Kopyanın manifest/baseline-lock/borrowed-example hash'leri ve exact komut `availability-0.0.363/result.json`, hata `resolve.log` içinde tutulur.

## Düzeltme kaynağı araması

| Kaynak | Kontrol ve sonuç |
|---|---|
| Yayımlanmış hir_ty **0.0.363** | VCS commit **946b80a756ac1a1ef8882ec65e4f3199e38a23c7**. [InferenceContext](https://github.com/rust-lang/rust-analyzer/blob/946b80a756ac1a1ef8882ec65e4f3199e38a23c7/crates/hir-ty/src/infer.rs#L1451) edition'ı resolver crate'inden alır; [method context](https://github.com/rust-lang/rust-analyzer/blob/946b80a756ac1a1ef8882ec65e4f3199e38a23c7/crates/hir-ty/src/method_resolution.rs#L196) self.edition taşır; [array filtresi](https://github.com/rust-lang/rust-analyzer/blob/946b80a756ac1a1ef8882ec65e4f3199e38a23c7/crates/hir-ty/src/method_resolution/probe.rs#L1633) aynı ctx edition'ını okur. Token edition düzeltmesi bu yolda yoktur. |
| [Resmî 2026-11-23 release](https://github.com/rust-lang/rust-analyzer/releases/tag/2026-11-23), **v0.3.3130** | Commit **37721d141370e0904033f7fc3a3abe11c446c4c1**'in aynı üç dosyası da aynı yolu taşır. Binary release numarası, ra_ap paket ailesi sürümü değildir. Binary/LSP semantic backend olarak kullanılmadı. |
| Araştırma anındaki resmî upstream snapshot'ı | Mutable master önce **9c74e78e01d7cd31663f4ecf71e65b1cc4b9ccb8** commit'ine çözüldü; sonra yalnız bu sabit commit'in kaynakları okundu. Aynı üç edition satırı korunur. Git dependency veya başarı üreten patch yapılmadı. |
| Eski solver hattından **0.0.300** kaynak karşı kontrolü | Dokuz API paketi mevcut, MSRV 1.88. VCS **9db05508ed08a4c952017769b45b57c4ad505872** [array aday filtresi](https://github.com/rust-lang/rust-analyzer/blob/9db05508ed08a4c952017769b45b57c4ad505872/crates/hir-ty/src/method_resolution.rs#L1278) yine crate edition kullanır; metot token span edition'ının gerektiğini belirten FIXME vardır. Bu da doğrulanmış doğru lookup adayı sayılmadı; build, API uyarlaması, transitive çözüm ve Rust 1.99.1 proc uyumu denenmedi. |

Arama sonucundaki [2024 edition fix commit'i](https://github.com/rust-lang/rust-analyzer/commit/f5e4eb2ef992118dc5260146676e8c047c24e432) yalnız CURRENT karşılaştırmasını Edition2021 yapar; token-span FIXME'sini bırakır. Başlığı mixed-edition çözümü sayılmadı. Upstream method-resolution geçmişi ve release notlarında bu kapıyı geçen bir yayımlanmış sürüm bağı kurulamadı.

İki `.crate` arşivinin SHA-256 değeri kendi version API checksum'ıyla eşleşti: hir_ty 0.0.363 ve 0.0.300. Bu iki paketteki beş ilgili Rust dosyası VCS commit'lerinin resmî raw kaynaklarıyla byte olarak eşleşti; VCS metadata'sındaki dirty=true alanı tek başına eşitlik sayılmadı. Kayıtlar `sources/verified.json` ve `metadata/ra_ap_hir_ty-<version>.json` içindedir. Bunlar kaynak uygunluk bulgularıdır; yeni derlenmiş semantik sonuç iddiası değildir.

**Kapsam sınırı:** 0.0.300 ile 0.0.358 arasındaki her sürüm çalıştırılmadı; hiçbir tarihsel sürümün hiçbir yoldan geçemeyeceği kanıtlanmış değildir. Kontrol edilen en yeni ortak aile, daha yeni kısmi aile, güncel resmî kaynak ve bir eski-solver karşı kontrolü içinde güçlü bir düzeltme adayı bulunmadı. Aynı yanlış kaynağı taşıyan sürümlerde kör build taraması yapılmadı. Yeni sürüm **build sayısı 0**; yalnız 0.0.363 için yukarıdaki izole çözüm ön kontrolü vardır.

## Bu yolculukta ne doğrulandı?

| İşlem | Yeni gözlem |
|---|---|
| Dokuz paket için current metadata ve sürüm kesişimi | En yeni ortak 0.0.358; kaynaklar metadata/ altında saklandı. |
| Üç eksik 0.0.363 paketi | Üç HTTP 404; üç cargo info exit 101. |
| İzole dokuz-pin lock üretimi | Exit 101; lock/build yok. Borrowed beklenti değiştirilmedi. |
| İki yayımlanmış arşiv / beş upstream dosyası | Checksum ve byte eşitliği doğrulandı. |
| Teslim probe'u | Bütün **25 girdi**, manifest/lock ve fixture'lar dahil tabanla aynı; 9 doğrudan pin / toplam 27 RA paketi **0.0.358**, toplam 179 paket korunur. |

Önceki 12 testin yeşili, compiler'ın iki tanığı ve explicit example'ın borrowed assertion'ında exit 101 vermesi **7bab3f9 tabanının tarihsel kanıtıdır**; bu yolculukta yeniden koşulmadı. Kod/lock/API değişmediğinden fmt, Clippy, eski fault'lar, geniş core/workspace/Maven de tekrar edilmedi. Yeni learning testi veya sahte başarı eklenmedi. Korunan diagnostic'in akışı/doğru beklentileri ayrı salt okunur okunabilirlik geçişinde gözden geçirildi; bulgu yok, kaynak değişmedi.

## Exact araçlar ve çıktı ayrımı

Rust/Cargo **1.99.1**, rustc commit **58cf8f9c35b355199727627d07ae34a4e2846d67**, host aarch64-apple-darwin. Compiler/component/default toolchain/global config değiştirilmedi. Bu araştırma yalnız metadata/kaynak indirme ve Cargo çözümü yaptı; proje build.rs/proc/runtime kodu çalıştırmadı.

Paket listeleri `https://crates.io/api/v1/crates/<crate>` üzerinden dokuz kez alındı. Sürüme özel üç yokluk sorgusu `https://crates.io/api/v1/crates/<crate>/0.0.363` kullandı. İndirilen kaynaklar `https://crates.io/api/v1/crates/ra_ap_hir_ty/<version>/download`, checksum metadata'sı aynı yolun download'sız biçimidir. Upstream GET istekleri resmî repo commit/release/search API'lerine ve yukarıdaki sabit commit raw kaynaklarına gitti; dış issue/PR yayımlanmadı.

Repo kökünde çalıştırılan Cargo komutları:

```sh
CARGO_TARGET_DIR="$PWD/target/rust-backend-version/metadata-target" cargo info ra_ap_load-cargo@0.0.363
CARGO_TARGET_DIR="$PWD/target/rust-backend-version/metadata-target" cargo info ra_ap_project_model@0.0.363
CARGO_TARGET_DIR="$PWD/target/rust-backend-version/metadata-target" cargo info ra_ap_proc_macro_api@0.0.363
env CARGO_TARGET_DIR="$PWD/target/rust-backend-version/availability-0.0.363/cargo" CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RUSTFLAGS='-D warnings' cargo generate-lockfile --manifest-path "$PWD/target/rust-backend-version/availability-0.0.363/probe/Cargo.toml"
```

Dördü de exit **101** verdi; compiler/setup başarısızlığı semantik kırmızı/yeşil sayılmadı. Index güncellemesi güncel registry doğrulaması içindir; offline çözüm iddia edilmez. Aday cache'i/proc DLL'i veya ikinci build denemesi oluşmadı. Eski cache/çıktılar silinmedi; bütün deney kopyası ve kayıtlar kendi target/rust-backend-version dizinindedir. Geçersiz 0.0.363 kopyasının manifesti ve ayrı baseline lock'u teslim probe'unun lock'u değildir.

## Teslim ve sonraki seçenek

Yeni **docs/rust-backend-version.md** dışında taban farkı yoktur. SHA256SUMS bu belge ve değişmeyen probe'un **26 girdisini** kapsar. REUSED-SHA256SUMS'taki **20 dış fixture girdisi** tabanla aynıdır; yeni çalıştırılmış kanıt sayılmaz.

Somut engel, tam yayımlanmış ailenin ve doğru token-edition yolunun birlikte doğrulanamamış olmasıdır. Sonraki küçük seçenek, düzeltmeyi gerçekten içeren sabit upstream kaynak/sürüm bağı bulunduğunda aynı compiler tanıklarını ve borrowed örneğini ilk kapı yapmak; yayımlanmamış/fork bir düzeltme veya compiler backend deneyi düşünülürse bunu ayrı kapsam ve yetkiyle ele almaktır. Burada böyle bir alternatif uygulanmadı. Mevcut doğru beklenti ve dar legacy koruması korunur; genel deney/nihai seçim, kaynak eşleme/dedup, unresolved-call/inactive cfg ve ürün adaptörü açık kalır.
