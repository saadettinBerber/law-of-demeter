# Rust analiz sonuçları sözleşmesi

Application, bilinen bulgularla eksik analizin kanıtını aynı raporda taşır. Bu dilimde Rust adaptörü, terminal çıktısı ve çıkış kodu uygulanmadı. Aşağıdaki çıkış kodları gelecek Rust CLI akışının sözleşmesidir; mevcut Java CLI davranışı değişmez.

## Application sözleşmesi

- `SourceReader::read` başarılı dosyaları ve mevcut `ReadError` değerlerini döndürür. Kısmi bir dosyanın çözülebilen metotları normal `SourceFile` içinde kalır; eksik gövde için boş `Method` üretilmez.
- `SourceReader::issues` aynı okumanın analiz sorunlarını ayrıca döndürür. `CheckProject`, bu metodu `read` sonrasında çağırır. Adaptör yalnızca o okumaya ait sorunları sağlamalıdır; önceki çalıştırmanın sorunlarını biriktirmemelidir. Varsayılanı boştur.
- `AnalysisIssue`, türünü, dosya yolunu, isteğe bağlı satırını ve mesajını taşır. Konum ve mesaj mevcut `ReadError` ile kurulur. Bu kullanım, sorunun ayrıca `source_errors` listesine eklendiği anlamına gelmez.
- Yeni sorun varsayılan olarak proje kapsamındadır. Okuma, yükleme, derleme ve build sorunları bu kapsamda kalır; seçim bunları süzmez. Bilinen konum tek başına gövde kapsamı anlamına gelmez.
- `.in_body(Target::method(...))` yalnızca ilgili gövdeyi etkileyen sorunu belirtir. `.in_body(Target::class(...))` sınıfa ait gövde sorununu belirtir. `owner()` bu kapsamın sahibini verir. Bunlar mevcut sınıf/metot seçimiyle süzülür. Okunan bir `Method` varsa `declared_within` ilişkisi de kullanılır; tamamen eksik bir gövdenin çevreleyen metodu yalnız adından çıkarılmaz.
- `TypeSource::project_types` kaynak okumasından bağımsız proje tipi kimliklerini verir. `CheckProject` bu adları kaynak okuyucuya önceden verir ve başarılı dosyaların bildirdiği sınıflarla birleştirir. Okunamayan modülün bilinen tipi böylece kütüphane muafiyeti kazanmaz. Tip üyeliği bir `TypeShape` değildir; bulunamayan şekil yerine boş şekil üretilmez. Varsayılan katalog boştur.
- `Report::issues()` seçime uygun analiz sorunlarını verir. Bilinen ihlaller ve hibrit bulgular ayrı listelerinde korunur. Mevcut `source_errors`, `type_errors`, `Counts` ve `Report::new` sözleşmeleri korunur; analiz sorunları çağrı veya metot sayısını artırmaz.

| `AnalysisIssueKind` | Kanıt |
|---|---|
| `Read` | Dosya veya modül okunamadı. |
| `Parse` | Etkin kaynak ayrıştırılamadı. |
| `TypeResolution` | Alıcı veya dönüş tipi belirlenemedi. |
| `TargetResolution` | Çağrının hedefi bulunamadı ya da belirsiz kaldı. |
| `Origin` | Nesnenin kökeni izlenemedi. |
| `Expansion` | Etkin makro ya da üretilmiş kaynak açılımı elde edilemedi. |
| `Unsupported` | Etkin bir yapı analiz tarafından desteklenmiyor. |
| `Compilation` | Derleyici tanısı analizin tamamlanmasını engelliyor. |
| `Build` | Build script veya benzeri hazırlık adımı analizi eksik bırakıyor. |

Türler dilden bağımsızdır; hangi etkin yapının hangi sorunu ürettiğine adaptör karar verir. Koşullu derlemeyle dışlanan kod etkin analiz sorunu sayılmaz. Mevcut okuma hataları yeni listeye otomatik kopyalanmaz.

## Gelecek Rust CLI çıkış kodları

| Kod | Sonuç |
|---|---|
| `0` | Analiz tamamlandı; ihlal ve hibrit yok. Geçerli, boş etkin hedef de sıfır sayımla bu gruptadır. |
| `1` | Analiz tamamlandı; ihlal veya hibrit var. |
| `2` | Analiz başlayamadı: kullanım, başlangıç veya manifest hatası; ya da gerçekten eşleşmeyen seçim. |
| `3` | Başlayan analiz eksik: okuma, çözümleme, açılım, etkin desteklenmeyen yapı, derleme veya build sorunu var. |

Eksiklik ve ihlal birlikteyse `3` önceliklidir; bilinen bulgular yine yazılır. Bu değerlendirme `issues()` yanında mevcut okuma hatalarını ve çözülemeyen çağrı bulgularını da kapsamalıdır. Tam yüklemeyi baştan engelleyen hata `2`, devam edilebilen fakat eksik kalan analiz `3` olur.

`Report::unmatched_targets()` bugün yalnızca başarıyla okunan dosyalardaki eşleşmeleri değerlendirir. Eksik yükleme varsa bu liste tek başına kesin bir eşleşmeme kanıtı değildir; gelecekteki Rust CLI belirsiz seçimi `2` diye raporlamamalıdır. Proje tipi kataloğu da tek başına bir metodun varlığını veya yokluğunu kanıtlamaz.

## Doğrulama sınırı

[Application davranış testleri](../crates/core/src/application/tests/analysis_issues.rs), gerçek `CheckProject` çağrısını küçük port stub'larıyla çalıştırır. İhlal ile sorunun birlikte korunmasını, gövde ve proje kapsamını, seçim kurallarını, eksik kaynakta proje üyeliğini ve varsayılan portlarla eski raporun korunmasını sınar. Bu testler Rust ayrıştırıcısını, Cargo yüklemesini veya yukarıdaki çıkış kodlarını uçtan uca doğrulamaz.
