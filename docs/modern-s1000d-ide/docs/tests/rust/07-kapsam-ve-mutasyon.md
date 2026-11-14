---
title: "Kapsam ve mutasyon"
ikon: 🧬
kapak: red
tarih: 2026-11-08
etiketler: [rust, testler]
ozet: Kapsam hangi kodun çalıştığını, mutasyon testi hangi değişikliğin fark edildiğini gösterir. Rust'ta kapsam -C instrument-coverage ile ölçülür, cargo-llvm-cov bunu tek komuta indirir. Kararlı Rust'ta dal kapsamı, doc testlerin kapsamı ve #[coverage(off)] yoktur; aynı dosyadaki test modülü de rapora girer. cargo-mutants kodu bilerek bozar ve testlerin bunu fark edip etmediğine bakar, ama her sınır hatasını üretmez. İkisi de bir hedef sayı değil, bakılacak yeri verir.
---

:::callout 🧬 red
**Kapsam hangi kodun çalıştığını, mutasyon testi hangi bozulmanın fark edildiğini gösterir. İkisi de sınır testinin yerini tutmaz.** Rust'ta kapsam `cargo llvm-cov` ile tek komutla ölçülür. Kararlı Rust'ta satır ve bölge kapsamı alınır, dal kapsamı ise nightly ister. Aynı dosyadaki `mod tests` kapsam raporuna girer ve yüzdeyi yükseltir. `cargo mutants` kodu bilerek bozar ve testlerin bunu yakalayıp yakalamadığına bakar. Yakalanmayan her mutant bakılacak bir yerdir, ama araç her işleci her biçimde bozmaz: sınır testleri yine tek tek yazılır.
:::

Clean Code serisinin [Kapsam ve uç durumlar](clean_code/05-kapsam-ve-uc-durumlar.html) notu kitabın görüşünü anlattı: testler bozulabilecek her şeyi sınayana kadar yetersizdir, kapsam aracı nereye bakılmadığını gösterir, yüzdesi bir hedef değil bir sinyaldir. Bu not aynı soruların Rust'taki araçlarını anlatıyor: kapsam nasıl ölçülür, neyi göstermez, mutasyon testi bu boşluğun ne kadarını kapatır.

Notun kod örnekleri ve aktarılan çıktılar 8 Kasım 2026'da Rust 1.98.1, cargo-llvm-cov 0.9.1 ve cargo-mutants 27.1.0 ile denendi. cargo-mutants bu not için kuruldu.

## Kapsam nasıl ölçülür?

Rust'ın kapsam ölçümü derleyicinin içindedir ve LLVM'in araçlarıyla okunur. rustc kitabının *Instrumentation-based Code Coverage* bölümü düzeneği *How it works* başlığında anlatır. Kısaca, `-C instrument-coverage` ile derlenen program iki şey taşır: kodun hangi parçasından kaç kez geçildiğini sayan sayaçlar ve bu sayaçları kaynak koddaki yerlere bağlayan bir eşleme. Sayaç değerleri program kapanırken bir `.profraw` dosyasına yazılır. Rapor, bu dosyadaki sayılar ile programdaki eşleme bir araya getirilerek üretilir.

Elle yapıldığında bu birkaç adımdır: `RUSTFLAGS` ile derlemek, oluşan dosyaları `llvm-profdata` ile birleştirmek, her test programını `llvm-cov`'a tek tek vermek. cargo-llvm-cov bu adımları tek bir komuta indirir. Aracın README dosyası kendini `-C instrument-coverage` üzerine kurulmuş bir sarmalayıcı olarak tanıtır. `cargo llvm-cov` testleri çalıştırır ve özeti yazar, `--text`, `--html` gibi seçenekler satır satır rapor üretir.

Aynı bölümün *Interpreting reports* başlığı raporun dört ölçüsünü tanımlar:

- **Fonksiyon kapsamı:** en az bir kez çalışan fonksiyonların oranı. Kitaba göre genellikle en kaba ölçüdür.
- **Somutlaştırma (instantiation) kapsamı:** generic fonksiyonların ve makroların ürettiği her somut kopyanın ayrı sayıldığı oran.
- **Satır kapsamı:** fonksiyon gövdelerindeki çalıştırılabilir satırlardan en az bir kez çalışanların oranı.
- **Bölge (region) kapsamı:** kod bölgelerinden çalışanların oranı. Bir satırda birden çok bölge olabilir. Kitaba göre genellikle en ince ölçüdür.

Bu ayrımın pratikte ne demek olduğunu bir denemede gördük. `form` adlı küçük bir crate'te, formun kabul ettiği miktarı denetleyen tek satırlık bir fonksiyonu yalnızca sıfırla sınadık:

```rust
// src/lib.rs
/// A quantity the form accepts: from 1 to 999.
pub fn quantity_ok(quantity: u32) -> bool {
    quantity >= 1 && quantity <= 999
}

// tests/quantity.rs
#[test]
fn zero_is_refused() {
    assert!(!form::quantity_ok(0));
}
```

Satır kapsamı yüzde 100 çıktı. Bölge kapsamı ise yüzde 75'te kaldı ve `--text` raporu, `&&`'in hiç değerlendirilmeyen sağ tarafının altına `^0` koydu:

```console
    3|      1|    quantity >= 1 && quantity <= 999
                                   ^0
```

### Kararlı Rust'ta olmayanlar

cargo-llvm-cov'un README dosyasına göre dal (branch) kapsamı ve doc testlerin kapsamı şimdilik isteğe bağlıdır ve nightly derleyici ister. rustc kitabının *Including doc tests* başlığı da doc testleri kapsama katmak için kararsız `-Z unstable-options --persist-doctests` seçeneğini kullanır. Denemede `cargo llvm-cov --branch` kararlı 1.98.1 ile şu hatayla durdu:

```text
warning: --branch flag requires nightly toolchain; consider using `cargo +nightly llvm-cov`
error: the option `Z` is only accepted on the nightly compiler
```

Bir fonksiyonu ya da modülü kapsamdan çıkaran `#[coverage(off)]` özniteliği de kararsızdır. README onu, cargo-llvm-cov'un açtığı `cfg(coverage)` ya da yalnızca nightly'de açılan `cfg(coverage_nightly)` koşuluyla birlikte kullanmayı önerir. Denemede kararlı derleyici özniteliği `E0658` hatasıyla reddetti: "the `#[coverage]` attribute is an experimental feature".

Bizce kararlı Rust'ta dal kapsamının yerini bir ölçüde bölge kapsamı tutar. Yukarıdaki denemede satır raporunun gizlediği, hiç çalışmayan koşul parçasını bölge raporu gösterdi.

## Testler de rapora girer

cargo-llvm-cov'un README dosyası, *Exclude file from coverage* başlığında rapordan varsayılan olarak çıkarılan yolları sayar. Bunlar arasında `tests`, `examples` ve `benches` klasörleri ile adı `tests.rs` ya da `_tests.rs` gibi biten dosyalar vardır. Kaynak dosyanın içindeki `#[cfg(test)] mod tests` bloğu ise bu kalıplara uymaz.

Denemede iki fonksiyonluk küçük bir crate'i üç testiyle iki biçimde ölçtük. Crate'in iki testli ilk hali aşağıdaki bölümde, üçüncü test de *cargo-mutants sınır hatasını üretmedi* bölümünde. Testler `src/lib.rs` içindeki `mod tests` bloğundayken rapor 20 satır ve 5 fonksiyon saydı: ürün kodunun 6 satırı ve 2 fonksiyonu, geri kalanı testlerin kendisi. Testler `#[cfg(test)] mod tests;` ile ayrı bir `src/tests.rs` dosyasına taşınınca rapor yalnızca ürün kodunun 6 satırını ve 2 fonksiyonunu saydı.

Bizce bunun sonucu açık: testler aynı dosyadayken her geçen testin satırları "kapsanmış" sayılır ve yüzdeyi yükseltir. README, `#[coverage(off)]`'un yerine bu dosya adı kuralının da kullanılabileceğini söyler. Kararlı Rust'ta çalışan yol budur. Serinin [Testlerin yeri](01-testlerin-yeri.html) notunda anlatılan görünürlük bu taşımayla değişmez, çünkü `mod tests;` ile bağlanan dosya yine aynı modülün alt modülüdür ve özel öğeleri görür.

## Yüzde yüz kapsam, hiçbir şey denetlemeyen test

Clean Code'un *Kokular ve Sezgisel Kurallar* bölümündeki *Yetersiz Testler* başlığında Martin, denenmemiş koşulların yanında doğrulanmamış hesaplardan da söz eder. Bizce kapsam bunun en çok ilk yarısını görebilir, o da yukarıdaki denemede olduğu gibi ancak bölge düzeyinde. *Software Engineering at Google* kitabının *Testing Overview* bölümünü Adam Bender yazdı. Bölümün *A Note on Code Coverage* başlığına göre kapsam bir satırın çağrıldığını ölçer, sonucunda ne olduğunu ölçmez. Az sayıda testle çok satır çalıştırıp hiçbirinin işe yarar bir şey yaptığını denetlememek mümkündür.

Denemede bunu kurduk. İki fonksiyonun testleri sonuçlarına neredeyse hiç bakmadı: biri sonucu hiç denetlemedi, öteki dosya yazma işleminin yalnızca `Ok` döndüğüne baktı, dosyanın içeriğine bakmadı. Crate'in `src/lib.rs` dosyası şuydu:

```rust
use std::io;
use std::path::Path;
use std::time::Duration;

const AUTOSAVE_DELAY: Duration = Duration::from_secs(2);

/// True once two seconds have passed since the last change.
pub fn autosave_due(since_last_change: Duration) -> bool {
    since_last_change >= AUTOSAVE_DELAY
}

/// Writes the module's text to its file.
pub fn save_module(path: &Path, text: &str) -> io::Result<()> {
    std::fs::write(path, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autosave_after_a_while() {
        autosave_due(Duration::from_secs(3));
    }

    #[test]
    fn saving_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        assert!(save_module(&dir.path().join("dm.xml"), "<dmodule/>").is_ok());
    }
}
```

`cargo llvm-cov` bölge, fonksiyon ve satır kapsamının üçünü de yüzde 100 buldu. cargo-mutants'ın kitabı, *How is mutation testing different to coverage measurement?* bölümünde tam bu durumu örnek verir: bir dosya yazıp `Result` döndüren fonksiyonun testi yalnızca dönüş değerine bakıyorsa, araç fonksiyonun gövdesini `Ok(())` ile değiştirir ve bunu yakalayan bir test olmadığını bildirir.

## Mutasyon testi: kodu bozup testlere sormak

Goran Petrovic, Google Testing Blog'daki *Mutation Testing* yazısında (12 Nisan 2021) yöntemi test paketinin kalitesini ölçmenin bir yolu olarak tanıtır: koda bilerek hatalar konur ve testlerin bunları fark edip etmediğine bakılır. Bu hataların her birine mutant denir. Testlerden biri kırılırsa mutant öldürülmüş, hepsi geçerse canlı kalmış sayılır. Carlos Arguelles, Marko Ivanković ve Adam Bender'in *Code Coverage Best Practices* yazısı (2020) da, kapsanan satırların gerçekten denetlenip denetlenmediğini anlamak için mutasyon testini daha iyi bir yol olarak gösterir. Bu yazı [Kapsam ve uç durumlar](clean_code/05-kapsam-ve-uc-durumlar.html) notunda ayrıntılı anlatılmıştı.

Bu notta kullanılan araç cargo-mutants. Kitabının *How it works* bölümüne göre araç önce kaynak ağacını geçici bir klasöre kopyalar ve testlerin değişikliksiz kodda geçtiğini doğrular. Sonra her mutant için kodu değiştirir, derler, testleri çalıştırır ve kodu geri alır. *Mutation patterns* bölümü üretilen değişiklikleri sayar. Başlıcaları bir fonksiyonun gövdesini dönüş türüne uyan bir değerle değiştirmek (`bool` için `true` ve `false`, `Result` için `Ok(...)`) ve işleçleri değiştirmektir. Testler ve `#[cfg(test)]` altındaki kod hiç değiştirilmez.

Yukarıdaki crate'te araç dört mutant üretti ve dördü de yakalanmadı:

```text
MISSED   src/lib.rs:9:5: replace autosave_due -> bool with true
MISSED   src/lib.rs:9:5: replace autosave_due -> bool with false
MISSED   src/lib.rs:9:23: replace >= with < in autosave_due
MISSED   src/lib.rs:14:5: replace save_module -> io::Result<()> with Ok(())
4 mutants tested in 3s: 4 missed
```

cargo-mutants'ın kitabı, *Using the results* bölümünde dört sonucu ayırır: testlerden birinin kırıldığı **caught**, hiçbirinin kırılmadığı **missed**, değişen kodun derlenmediği **unviable** ve testlerin zaman aşımına uğradığı **timeout**. Kitaba göre yakalanmayan her mutant testlerde bir boşluk olabileceğini gösterir, ama bazen değişen kod asıl koddan ayırt edilemez. Aynı bölüm, yakalanmayan bir mutantın nasıl ele alınacağına da değinir. Kitap, testi tam o mutanta göre biçimlendirmekten kaçınmayı önerir, çünkü mutant olası hatalardan yalnızca biridir. Önerdiği test, doğru davranışı uygun bir düzeyde, mümkünse dışa açık arayüz üzerinden denetleyen testtir.

Testleri sonuçlara bakacak biçimde yeniden yazdık: birinci fonksiyon için bir saniyede `false`, üç saniyede `true` beklendi, ikincisi için de yazılan dosya geri okundu. Dört mutantın dördü de yakalandı.

## cargo-mutants sınır hatasını üretmedi

Yeni testlerde iki saniyenin tam kendisi yoktu. `>=` işaretini elle `>` yaptığımızda, yani tam iki saniyede kaydetmeyen bir hata eklediğimizde testler yine geçti. cargo-mutants bu hatayı hiç üretmemişti. Kitabın *Mutation patterns* bölümündeki tabloya göre `>=` yalnızca `<` ile, `<=` de yalnızca `>` ile değiştirilir. Kitap bir sınırı bir birim kaydıran `>=` yerine `>` gibi değişikliklerin neden üretilmediğini açıklamıyor.

Bu hatayı ancak sınırın iki yanını denetleyen bir test yakaladı:

```rust
#[test]
fn autosave_starts_exactly_at_two_seconds() {
    assert!(!autosave_due(Duration::from_millis(1999)));
    assert!(autosave_due(Duration::from_millis(2000)));
}
```

Martin, *Sınır Koşullarını Test Edin* başlığında sınırların özel bir özen istediğini söyler, çünkü yanılgı çoğu zaman algoritmanın ortasında değil kenarlarındadır. Bizce Rust'ta da durum aynı: mutasyon testinin "hepsi yakalandı" sonucu bir örneklemin sonucudur, testlerin eksiksiz olduğunu kanıtlamaz.

## Maliyet ve gürültü

cargo-mutants'ın kitabı, *Goals* bölümünde çalışma süresini kabaca geçerli mutant sayısı ile her mutantın derleme ve test süresinin çarpımı olarak verir. *How is mutation testing different to coverage measurement?* bölümü de bunu yöntemin bir sakıncası olarak sayar: büyük ve testleri yavaş bir ağaçta yavaş olabilir. Süreyi kısaltmanın yolları kitabın kendi bölümlerinde anlatılır:

- `-f` ile yalnızca belirli dosyaları denemek.
- `--in-diff` ile yalnızca bir değişikliğin dokunduğu kodu denemek. Kitap bunun tam bir çalıştırmanın yerini tutmadığını, bir bölgedeki değişikliğin başka bir bölgenin testlerini zayıflatabileceğini belirtir.
- İlgisiz bulunan kodu `#[mutants::skip]` ile işaretlemek. Bunun için `mutants` crate'i normal bağımlılık olarak eklenir. Öznitelik derlenen kodu etkilemez.

Petrovic'in yazısının *From Prototype To Launch* başlığı ölçeğin öteki ucunu anlatır. Yazar aracı Google'da ilk denediğinde birkaç dosyada binlerce mutantla karşılaştı ve çoğu işe yaramıyordu, örneğin bir günlük mesajındaki işleci değiştiren mutantlar. Yazar bunlara verimsiz mutant der: onları öldürmek için yazılacak testler test paketini iyileştirmez, kırılganlaştırır. Petrovic'in çözümü mutantları kod incelemesinde göstermekti, yalnızca değişen ve kapsanan satırlarda ve satır başına en çok bir tane olarak. Verimsiz kod parçalarını da kurallarla ayıkladı. Kod yazarının ve incelemecilerin "işe yaramaz" diye işaretlediği bulguların oranı başta yüzde 80 dolayındaydı, zamanla yaklaşık yüzde 15'e indi. Yazara göre bu oran hiçbir zaman sıfır olamaz, çünkü bazı mutantlar asıl kodla aynı davranır.

Yöntemin neye dayandığı, yazının *Fault Coupling* başlığında anlatılır. Mutantlar gerçek hatalardan daha basittir. Bu yüzden mutasyon testinin değeri bağlaşım (coupling) varsayımına bağlıdır: basit mutantları yakalayan bir test paketi, daha karmaşık gerçek hataları da yakalar. Petrovic bu varsayımı Google'ın kendi kodunda, hata düzelten değişiklikler üzerinde sınadı. Vakaların yaklaşık yüzde 70'inde hata bir mutantla eşleşti.

cargo-mutants'ın kitabı, *Limitations* bölümünde aracın ancak hermetik testlerle doğru sonuç verdiğini söyler: kararsız ya da dış duruma bağlı testlerde yanlış sonuca varır. *Cautions* bölümü de bir risk anlatır. Araç dosya silmek gibi bir yan etkisi olan kodu bozabilir. Testler yan etkilerini geçici kaynaklarla sınırlamıyorsa bu, istenmeyen bir dizinin silinmesine yol açabilir. Serinin [Her ortamda çalışma](03-her-ortamda-calisma.html) notundaki "her teste kendi klasörü" kuralı burada bir güvenlik önlemine dönüşür.

## Clean Code ile bağ

- **Yetersiz testler.** Bizce Martin'in doğrulanmamış hesapları kapsamın göremediği şeydir, Rust'ta onları aramanın bir yolu da cargo-mutants'tır.
- **Bir kapsam aracı kullanın.** cargo-llvm-cov, Martin'in istediği aracın Rust'taki karşılığıdır. Bizce kararlı Rust'ta satır raporu yerine bölge raporuna bakmak, kitabın renklendirilmiş satırlarından daha fazlasını gösterir.
- **Sınır koşullarını test edin.** cargo-mutants her sınır hatasını üretmez. Sınırın iki yanını denetleyen test yine elle yazılır.

## Bu projede yaşadığımız

- **Araçlar bu not için ilk kez çalıştırıldı.** 8 Kasım 2026 sabahına kadar projede ne kapsam ne de mutasyon aracı kullanılmıştı. Commit geçmişinde "kapsam" geçen iletiler kapsam ölçümünü değil, belgelerin ve özelliklerin kapsamını anlatıyordu.
- **Yüzde yüksekti, ölçülen satırların yarısından fazlası testti.** 8 Kasım 2026'da main dalının o sabahki hali (10.57'deki commit) geçici bir klasöre çıkarıldı. Derlemeyi ve testleri çalıştıran `cargo llvm-cov --no-report` 151 saniyede bitti: 407 test geçti, 5 test `#[ignore]` ile atlandı, bellek ölçme örneğinin 4 testi de geçti. Ardından `cargo llvm-cov report` satır kapsamını yüzde 97,4, bölge kapsamını yüzde 97,2 verdi. Testlerin hepsi kaynak dosyalarının içinde, `#[cfg(test)]` ile işaretli modüllerdeydi. Ayrı bir `tests/` klasörü yoktu. Satır satır raporu, `#[cfg(test)]` altındaki kod ve geri kalanı diye kendi betiğimizle ayırdığımızda ölçülen satırların yaklaşık yüzde 57'si testlerin kendisi çıktı. Ürün kodunun satır kapsamı yaklaşık yüzde 96,2'ydi. Satır satır raporun toplam satır sayısı özettekinden biraz farklı olduğu için bu iki oran yaklaşıktır.
- **Asıl bilgi kapsanmayan yerlerdeydi.** Programın giriş noktası hiçbir testte çalışmıyordu. Onun dışında ürün kodundaki en düşük kapsam blok editörünün referans kodundaydı (yaklaşık yüzde 82). Onu blok editörünün kendisi ve BREX denetiminin XPath kodu (ikisi de yaklaşık yüzde 92) izliyordu. Arguelles, Ivanković ve Bender'in yazısı da kapsam verisinin değerinin önemli bir kısmını, kapsanmayanı göstermesinde görür.
- **Mutasyon iki dosyada 7 dakika sürdü.** Projenin tamamında 1209 mutant üretiliyordu. 8 Kasım 2026'da bunlardan yalnızca iki küçük dosyanınkileri denedik: editörde yazılan kelimeye uyan önerileri bulan eşleştirme kodu ve BREX kurallarındaki izinli değerleri okuyan kod. Aracı kaynak ağacını kopyalamadan, `--in-place` ile, yalnızca bu iş için çıkarılmış geçici klasörde çalıştırdık. Kitabın *Cautions* bölümü bu seçeneğin asıl kaynak ağacını değiştirdiğini söyler ve bunun için atılabilir bir kopya önerir. Değişikliksiz kodun ilk derlemesi 133 saniye, testleri 4 saniye sürdü. 69 mutantın tamamı 448 saniyede bitti: 42'si yakalandı, 26'sı derlenmedi, 1'i yakalanmadı. Derlenmeyen mutantlar teste geçmediği için hızlı bitti.
- **Derlenmeyen mutantların çoğunda tür yanlış tahmin edilmişti.** Eşleştirme kodunun 44 mutantının 24'ü derlenmedi. Bunların çoğu `std::ops::Range<usize>` döndüren fonksiyonların gövdesini `Range::new()` gibi bu türde olmayan çağrılarla değiştiriyordu. cargo-mutants'ın kitabı, *Limitations* bölümünde aracın türleri tam olarak anlamadığını ve bu yüzden bazen derlenmeyen mutantlar ürettiğini söyler. Eşleştirme kodunun derlenen 20 mutantının hepsi yakalandı.
- **Yakalanmayan tek mutant bizce eşdeğerdi.** BREX değer kodunda, bir değeri öneki ve rakamları diye bölen yardımcıdaki `&&` işaretinin `||` yapılması hiçbir testi kırmadı. Bizce bu değişiklik dışarıdan görülebilen hiçbir davranışı değiştirmiyordu. Yardımcıyı çağıran iki fonksiyon, öneki ve uzunluğu karşılaştırdıktan sonra rakam kısmını sayıya çeviriyor ve çevrilemeyen değeri zaten reddediyordu. Önekte `~` bulunduğu durum da çağıranlarda ya hiç oluşmuyor ya da önek karşılaştırmasında eleniyordu. Bizce burada mutasyon testi eksik bir testi değil, gereğinden fazla bir koşulu göstermişti.

## Belgelerin söylemedikleri

- Rust Book (2 Kasım 2026 tarihli sürüm) kapsamdan yalnızca geçerken söz eder. *Writing Automated Tests* bölümündeki *Test Organization* başlığının *Integration Tests* kısmı, tek başına doğru çalışan parçaların birlikte sorun çıkarabileceğini, bu yüzden birlikte çalışan kodun kapsamının da önemli olduğunu söyler. *An I/O Project* bölümündeki *Adding Functionality with Test Driven Development* başlığı da önce test yazmanın kapsamı yüksek tutmaya yardım ettiğini söyler. Kitabın tamamında kapsamın nasıl ölçüleceği anlatılmaz, mutasyon testi de geçmez.
- cargo-mutants'ın kitabı iki yerde kendisiyle çelişiyor. *Limitations* bölümü nextest desteğinin ileride eklenebileceğini söylerken *Using nextest* bölümü nasıl kullanılacağını anlatıyor. *Skipping functions with an attribute* bölümü `#[mutants::skip]`'in bir dosyanın başına `#![mutants::skip]` olarak da yazılabileceğini söylerken *Filtering files* bölümü Rust'ın şimdilik modül düzeyinde böyle bir özniteliğe izin vermediğini, bir modülü atlamanın tek yolunun dosya filtresi olduğunu söylüyor. Denemede `#[mutants::skip]` fonksiyonda ve satır içi `mod` bloğunda çalıştı, dosya başındaki `#![mutants::skip]` ise kararlı Rust'ta "inner macro attributes are unstable" hatasıyla derlenmedi.
