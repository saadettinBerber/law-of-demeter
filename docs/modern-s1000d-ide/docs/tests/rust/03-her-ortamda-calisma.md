---
title: "Her ortamda çalışma"
ikon: 🌍
kapak: orange
tarih: 2026-11-07
etiketler: [rust, testler]
ozet: cargo test aynı test programındaki testleri tek bir süreçte, paralel iş parçacıklarında çalıştırır. Ortak dosya, ortam değişkeni ve değişebilir static testleri birbirine bağlar. --test-threads=1 aynı süreçteki çakışmayı önler, başka süreçlerle çakışmayı önlemez. Ortam değişkeni sınırda bir kez okunur, testlere değer olarak verilir. Ortama bağlı test #[ignore = "neden"] ile görünür biçimde ayrılır; kendini sessizce atlayan test "ok" görünür.
---

:::callout 🌍 orange
**Bir test hangi makinede, hangi sırayla ve hangi testlerle birlikte çalışırsa çalışsın aynı sonucu vermelidir.** `cargo test` aynı test programındaki testleri tek bir süreçte, paralel iş parçacıklarında çalıştırır. Ortak bir dosya, bir ortam değişkeni ya da değişebilir bir `static` testleri birbirine bağlar. `--test-threads=1` aynı süreçteki çakışmayı önler, ama aynı dosyayı kullanan başka bir süreçle çakışmayı önlemez. Bizce asıl çözüm paylaşımı kaldırmaktır: her test kendi klasörünü kurar, ortam değişkeni programın sınırında bir kez okunur. Belirli bir ortam isteyen test `#[ignore = "neden"]` ile görünür biçimde ayrılır.
:::

Clean Code serisinin [F.I.R.S.T.](clean_code/03-first.html) notu, testlerin birbirinden bağımsız ve her ortamda tekrarlanabilir olmasını anlattı. Projeden dört örnek verdi: macOS'ta kırılan geri alma testi, gerçek saat yerine test saati, her teste ayrı geçici klasör ve makineye göre değişebilecek fontlar. Bu not, Rust'ta bu iki kuralı neyin bozduğunu ve hangi araçların koruduğunu anlatıyor.

Notun kod örnekleri ve aktarılan çıktılar 7 Kasım 2026'da Rust 1.98.1 ile denendi. Rust Book atıfları kitabın 2 Kasım 2026 tarihli sürümüne dayanıyor.

## Testler aynı süreçte, paralel çalışır

Paralel çalışmanın bedeli ortak durumdur. Rust Book'un *Writing Automated Tests* bölümündeki *Controlling How Tests Are Run* başlığının *Running Tests in Parallel or Consecutively* kısmı bunu bir dosya örneğiyle anlatır. Aynı adlı dosyaya yazıp sonra onu okuyan iki test düşünelim: biri yazdıktan sonra, okumadan önce öteki aynı dosyanın üzerine yazabilir. Kod doğru olduğu halde test kırılır. `cargo test` testleri varsayılan olarak iş parçacıklarında aynı anda çalıştırdığı için kitap testlerin birbirine ya da ortak bir duruma dayanmamasını ister. Ortak durum yalnızca dosyalar değildir; kitap çalışma dizinini ve ortam değişkenlerini de bu kapsamda sayar. Önerdiği iki çareden biri her teste ayrı bir dosya vermek, öteki testleri tek tek çalıştırmaktır.

rustc kitabının *Tests* bölümü, *CLI arguments* başlığı altındaki *Execution options* kısmında üç ayrıntı ekler:

- İş parçacığı sayısı varsayılan olarak donanımın sunduğu eşzamanlılık kadardır.
- Testler varsayılan olarak alfabetik sıraya dizilir, ama paralel çalışırken hangisinin ne zaman çalışacağı belirlenimci değildir.
- Testleri rastgele sırayla çalıştıran `--shuffle` seçeneği 7 Kasım 2026'da kararlı değildi ve `-Z unstable-options` istiyordu.

Bu not boyunca "aynı süreç", tek bir test programının içi demek. Serinin [Testlerin yeri](01-testlerin-yeri.html) notunda anlatıldığı gibi `tests/` altındaki her dosya ayrı bir program olarak derlenir ve ayrı bir süreçte çalışır.

Denemede iki test aynı geçici dosyaya kendi harfini yazıp geri okudu. Çakışma olasılığını artırmak için her test bunu 2000 kez yaptı. Paralel çalıştırmaların 20'de 20'si kırıldı. Çıktıda `"b"` bekleyen test `"a"` okumuştu:

```text
assertion `left == right` failed
  left: "a"
 right: "b"
```

Aynı testler `cargo test -- --test-threads=1` ile 20 çalıştırmanın hiçbirinde kırılmadı.

## `--test-threads=1` neyi çözer, neyi çözmez?

Rust Book tek iş parçacığını meşru bir seçenek olarak sunar: testler daha uzun sürer, ama ortak bir durumu paylaşsalar da birbirine karışmazlar.

Bu, yalnızca tek bir süreç için doğrudur. Denemede aynı test programını iki ayrı süreçte, ikisi de tek iş parçacığıyla, aynı anda çalıştırdık. On turun onunda da testler kırıldı. İki süreç testleri aynı alfabetik sırayla başlattığı için çakışma, başlama anlarının ne kadar kaydığına bağlıydı; bu denemede her turda çakıştılar. Bizce geçici klasör en azından aynı kullanıcının süreçleri arasında ortaktır. İki çalışma ağacında aynı anda `cargo test` çalıştırmak ya da aynı makinede iki sürekli entegrasyon işi koşturmak bu durumu yaratır.

Bizce `--test-threads=1` paylaşımı kaldırmaz, yalnızca aynı süreç içinde görünmez kılar. Üstelik bütün test takımını yavaşlatır ve F.I.R.S.T.'in hız kuralına ters düşer. Asıl çözüm, Rust Book'un ilk önerdiği yoldur: her test kendi dosyasını kullanır.

## Her teste kendi klasörü

Testin kendi klasörünü nerede ve nasıl adlandıracağı da önemlidir. Standart kütüphanenin `std::env::temp_dir` belgesi bu konuda bir uyarı yapar ve bir öneride bulunur. Uyarıya göre geçici klasörü başka kullanıcılar ya da farklı yetkilerle çalışan süreçler de kullanıyor olabilir. Bu yüzden orada sabit ya da önceden kestirilebilen adla açılan bir dosya güvenlik açığına dönüşebilir. Öneri ise benzersiz adları güvenli biçimde üreten bir crate kullanmaktır.

Projede böyle bir crate yerine elle yazılmış bir yardımcı kullanıldı. Klasör adı testin verdiği addan ve sürecin numarasından oluşuyordu. Bu ad kestirilebilir olduğu için belgenin uyardığı türdendi. Aşağıdaki, o yardımcının sadeleştirilmiş halidir:

```rust
/// A folder of its own for each test; removed when the test ends.
struct TestFolder(PathBuf);

impl TestFolder {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("ide-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestFolder {
    fn drop(&mut self) {
        // A panic here while a failed test unwinds would abort the whole run.
        let _ = fs::remove_dir_all(&self.0);
    }
}
```

Süreç numarası iki ayrı `cargo test` sürecini ayırır, testin verdiği ad da aynı süreçteki testleri. Bizce bu düzenin zayıf yanı, adların benzersizliğinin elle sağlanmasıdır. Rastgele ad üreten bir crate bu yükü kaldırır.

**Temizlik panik yapmamalı.** `Drop` içindeki silme hatası bilerek görmezden geliniyor. Standart kütüphanenin `Drop` belgesi, *Panics* başlığında bunun nedenini anlatır: bir panik sırasında yığın çözülürken (unwinding) çağrılan `drop` da panik yaparsa, bu çifte panik programı büyük olasılıkla durdurur. Denemede kırılan bir testin temizliği panik yaptı. Test programı `SIGABRT` ile kapandı, aynı anda çalışan ilgisiz bir testin sonucu da hiç raporlanmadı.

## Ortam değişkenleri

Rust Book ortam değişkenlerini testler arasındaki ortak durumun bir parçası sayar. Standart kütüphane bundan daha ileri gider. `std::env::set_var` belgesinin *Safety* başlığına göre Windows dışındaki sistemlerde, çok iş parçacıklı bir programda bu fonksiyonu güvenle kullanmanın tek yolu onu hiç kullanmamaktır, çünkü bu sistemler ortamı iş parçacıklarına karşı güvenli biçimde okumayı sağlamaz. Rust'ın edition rehberindeki *Rust 2024* bölümünün *Unsafe functions* başlığı, `set_var` ve `remove_var` fonksiyonlarının 2024 edition'ından beri `unsafe` sayıldığını anlatır. Denemede 2024 edition'lı bir testte `set_var`'ı `unsafe` blok olmadan çağırınca derleyici hata verdi.

Bizce bundan çıkan sonuç şu: test çalıştırıcısı testleri iş parçacıklarında çalıştırdığı için, testte ortam değişkeni değiştirmekten kaçınılır. Ortamı okuyan kod ise testin ulaşamayacağı bir yere itilir.

Rust Book'un *An I/O Project* bölümündeki *Working with Environment Variables* başlığı buna uyan bir düzen kurar, ama bunu test için yaptığını söylemez. Programın yapılandırmasını kuran fonksiyon `IGNORE_CASE` değişkenini bir kez okur ve sonucu `ignore_case` alanına yazar. Programı çalıştıran fonksiyon bu alana bakıp iki arama fonksiyonundan birini seçer. Testler, ortamı hiç okumayan bu arama fonksiyonlarını doğrudan çağırır.

Aşağıdaki örnek aynı fikrin bizim uyarlamamız. Burada ayar, arama fonksiyonuna değer olarak veriliyor:

```rust
/// Reads the environment once, at the program's edge.
pub fn options_from_env() -> Options {
    Options {
        ignore_case: std::env::var("IGNORE_CASE").is_ok(),
    }
}

/// Takes the setting as a value, so tests never touch the environment.
pub fn matching_titles<'a>(query: &str, titles: &[&'a str], options: &Options) -> Vec<&'a str> {
    // ...
}

#[test]
fn ignoring_case_finds_lowercase_titles() {
    let titles = ["Yakıt filtresi", "yakıt pompası", "Kapak"];
    let options = Options { ignore_case: true };

    let found = matching_titles("Yakıt", &titles, &options);

    assert_eq!(found, ["Yakıt filtresi", "yakıt pompası"]);
}
```

Testin bir alt süreç başlatması gerekiyorsa, `set_var` belgesinin önerdiği yol `Command::env`'dir. Değişken yalnızca o alt sürecin ortamına girer.

## Değişebilir `static`

Aynı test programındaki testler tek bir süreçte çalıştığı için bir `static` değer de onların ortak durumudur. Bizce burada iki durum ayrılır:

- **Yalnızca okunan bir `static`**, örneğin bir kez kurulan bir font deposu, testleri birbirine bağlamaz. Ama içeriği makineden okunuyorsa sonucu makineye bağlar. Bu bağımsızlık değil, tekrarlanabilirlik sorunudur.
- **Değişebilir bir `static`**, örneğin bir `Mutex` ya da sayaç, testleri birbirine bağlar. Bir testin bıraktığı değer ötekinin başlangıcı olur ve sonuç testlerin çalışma sırasına göre değişir.

## Makineye ve sisteme bağlı testler

**Platforma özgü testler.** `#[cfg(unix)]` gibi bir koşulla işaretlenen test, koşulu karşılamayan sistemde hiç derlenmez. Bizce bunun bir yan etkisi var: Windows'ta bu test çıktıda "atlandı" diye de görünmez, test sayısı sessizce azalır.

**`#[ignore = "neden"]`.** Rust Reference'ın *Testing attributes* sayfasındaki *The ignore attribute* başlığı, özniteliğin bir neden metni de alabileceğini söyler. rustc kitabının *Tests* bölümüne göre kapatılan test yine derlenir, yalnızca varsayılan olarak çalıştırılmaz. Bizce `#[cfg(unix)]` ile farkı budur: kapatılan test her sistemde derlenmeye devam eder ve çıktıda "ignored" diye görünür. Rust Book'un *Ignoring Tests Unless Specifically Requested* kısmı bu testlerin nasıl çalıştırılacağını anlatır: `cargo test -- --ignored` yalnızca kapatılmış testleri, `cargo test -- --include-ignored` hepsini çalıştırır. Kitabın örneği çok uzun süren bir testtir. Bizce belirli bir ortam isteyen testler de aynı yolla ayrılabilir:

```rust
#[test]
#[ignore = "gerçek bir ACP bağdaştırıcısı ister"]
fn talks_to_the_real_adapter() {}
```

Denemede çıktı nedeni de gösterdi:

```text
test tests::talks_to_the_real_adapter ... ignored, gerçek bir ACP bağdaştırıcısı ister
```

**Kendini atlayan test "ok" görünür.** Belgelerde, kararlı Rust'ta bir testin çalışırken kendini "atlandı" diye işaretlemesinin bir yolunu bulamadık. Rust Reference'ın *The test attribute* başlığına göre `()` döndüren bir test, panik yapmadan sona erdiği sürece geçmiş sayılır. Uygun ortamı bulamayıp erkenden dönen test de bu yüzden "ok" görünür. Geçen testlerin yazdıkları da çıktıda gösterilmez. Rust Book'un *Showing Function Output* kısmı bunu standart çıktı için anlatır. rustc kitabının *Output options* kısmına göre `--no-capture` ve `--show-output` seçenekleri standart hata çıktısını da kapsar. Denemede erken dönen testin `eprintln!` ile yazdığı "skipped" iletisi düz `cargo test` çıktısında hiç görünmedi. İleti ancak `-- --show-output` ya da `-- --no-capture` ile göründü. Bizce böyle bir test, özellikle bir sürekli entegrasyon makinesinde, asıl denetimini yapmadığı halde geçmiş gibi görünebilir.

**Göreli yollar.** 6 Kasım 2026'daki ilk sürümden beri projenin testleri örnek dosyalara `env!("CARGO_MANIFEST_DIR")` ile ulaştı. Bu makro, paketin klasörünü derleme anında koda yazar. Böylece yol, testin hangi dizinden çalıştırıldığına bağlı kalmaz. Cargo Book'un `cargo test` sayfasındaki *Working directory of tests* başlığına göre birim ve entegrasyon testlerinin çalışma dizini zaten testin ait olduğu paketin kök dizinidir.

## Kararsız testler

*Software Engineering at Google* kitabının *Testing Overview* bölümünü Adam Bender yazdı. Bölümün *Test Size* başlığına göre Google'ın küçük testleri tek bir süreçte, birçok dilde tek bir iş parçacığında çalışır. Uyumaz, diske ve ağa erişmez, başka bloke eden çağrılar yapmaz. Bender'e göre bu kısıtların amacı, küçük testleri yavaşlığın ve belirlenimsizliğin başlıca kaynaklarından uzak tutmaktır. Belirlenimci olmayan biçimde kırılan, yani kararsız (flaky) birkaç test bile takımın vaktini yer.

*Flaky Tests Are Expensive* başlıklı örnek olay sayılarla konuşur. Her testin, kırılmaması gerekirken kırılma olasılığı binde bir olsa, günde on bin test çalıştıran bir ekip her gün on kırılmayı araştırır. Kırılan testi otomatik olarak yeniden çalıştırmak düşük oranlarda mantıklı bir takastır, ama kök nedeni yalnızca erteler. Asıl tehlike güvenin kaybolmasıdır: ekip birkaç kararsız kırılmayı araştırdıktan sonra test takımına güvenmez olur ve kırılan testlere tepki vermeyi bırakır. Bender'e göre kırılma oranı yüzde bire yaklaştıkça testler değerini yitirmeye başlar. Google'ın oranı yaklaşık binde bir buçuktur. Bender kararsızlığın kaynakları arasında saati, iş parçacıklarının sıralanmasını ve ağ gecikmesini sayar.

Aynı bölümün *Properties common to all test sizes* başlığı bütün testlerin hermetik olmaya çalışmasını ister: test kendi ortamını kurmak, çalıştırmak ve temizlemek için gereken her şeyi içerir. Testlerin çalışma sırası gibi dış koşullar hakkında olabildiğince az varsayımda bulunur.

## Süreç başına test: nextest

nextest, `cargo test` yerine kullanılabilen bir test çalıştırıcısıdır. Belgesinin *Why process-per-test?* sayfasına göre her testi ayrı bir süreçte çalıştırır. Gerekçesi, test yazarlarının ve kütüphanelerin ortak global durumu kendi aralarında düzenlemek zorunda kalmamasıdır. Belge bu modelin bedelini de not eder: süreç başlatmak Windows'ta yavaştır, macOS'ta da kötü amaçlı yazılım korumaları araya girerse yavaşlayabilir.

Bizce bu, bir sürecin belleğinde kalan paylaşımı kaldırır: `static` değerler ve süreç içinde değiştirilen ortam değişkenleri. Diskteki paylaşımı kaldırmaz. Yukarıdaki iki süreçli deneme, ayrı süreçlerin aynı dosyada yine çakıştığını gösterdi. nextest'i bu not için kurup denemedik. Bu kısım yalnızca belgesine dayanıyor.

## Clean Code ile bağ

- **Bağımsız (Independent).** Rust'ta bu kuralı en çok paralel çalışma sınar. Ortak dosya, ortam değişkeni ve değişebilir `static` testleri birbirine bağlar.
- **Tekrarlanabilir (Repeatable).** Makineden okunan fontlar, platforma özgü kısayollar ve gerçek saat, sonucu makineye bağlar. Bizce bunlar paralellikten bağımsız sorunlardır: tek iş parçacığıyla da sürerler.
- **Hızlı (Fast).** `--test-threads=1` ile çakışmayı gizlemek, hız kuralından ödün vermektir.
- **Kendini doğrulayan (Self-Validating).** Kendini sessizce atlayan test, geçmediği halde geçti diye raporlanır. Bizce bu, testin sonucunu kendisinin söylemesi kuralını bozar.

## Bu projede yaşadığımız

- **Elle benzersiz klasör.** 6 Kasım 2026'daki ilk sürümden beri dosya yazan testler, testin adı ve sürecin numarasıyla adlandırılan kendi geçici klasörlerini kurdu. Yardımcı, önceden kalmış aynı adlı bir klasörü baştan siliyordu. Kodundaki yoruma göre bu, yarıda kesilmiş bir çalıştırmanın aynı süreç numarasıyla klasör bırakmış olabileceği içindi. Silme hatası ise `Drop` içinde bilerek görmezden geliniyordu, yorumu da çifte panik riskini anlatıyordu.
- **Rastgele adlı klasör.** 7 Kasım 2026'da PDF kaydetme, hedefin yanındaki dosyaları ezmesin diye benzersiz adlı geçici dosya kullanmaya başladı ve projeye `tempfile` crate'i eklendi. Aynı gün eklenen ACP bağlantı denemesinin testleri geçici klasörlerini bu crate ile kurdu. Böylece projede iki yöntem yan yana durdu.
- **Ortam değişkeni sınırda okundu.** ACP bağlantı denemesi, bağdaştırıcının komutunu değiştirmek için bir ortam değişkeni okuyordu. Değişkeni yalnızca denemeyi başlatan `run` fonksiyonu okuyordu. Bu fonksiyonu program `--acp-probe` bayrağıyla açılınca ve denemenin örnek programı çağırıyordu. Asıl işi yapan fonksiyon komutu ve zaman aşımını parametre olarak alıyordu, testler de onu doğrudan çağırıyordu. 7 Kasım 2026'nın sonunda projenin testlerinde hiç `set_var` yoktu.
- **Gerçek saatle bir test.** Aynı denemenin testlerinden biri, yanıt vermeyen bir alt sürecin zaman aşımıyla kapatıldığını sınadı. Test gerçek saati kullandı: sürenin zaman aşımından en fazla iki saniye uzun olduğunu denetledi ve alt sürecin kapandığını 100 milisaniyelik aralıklarla yoklayarak bekledi. Bender'in tanımına göre bu, alt süreç başlatan ve bekleyen bir orta boy testti. Bizce iki saniyelik pay kırılma olasılığını azaltıyordu, ama yükü çok yüksek bir makinede tümüyle ortadan kaldırmıyordu.
- **Kendini atlayan bir test.** 7 Kasım 2026'da eklenen bir PDF kaydetme testi, salt okunur bir klasöre yazmanın başarısız olmasını bekliyordu. `root` kullanıcısı klasör izinlerini yok saydığı için test, klasör yazılabilir kaldığında bir "skipped" iletisi yazıp erkenden dönüyordu. Bu durumda çıktıda "ok" görünecekti, ileti de ancak `--show-output` ya da `--no-capture` ile okunabilecekti.
- **Kapatılan test yoktu.** 7 Kasım 2026'nın sonunda projede `#[ignore]` ile kapatılmış bir test yoktu.

## Belgelerin söylemedikleri

- Rust Book, `--test-threads=1`'in başka süreçlerle çakışmayı önlemediğini söylemez.
- Rust Book'un test bölümü, erken dönen bir testin "ok" görüneceğinden söz etmez.
- Rust Book'un test bölümleri, `set_var`'ın 2024 edition'ında `unsafe` olduğundan söz etmez. Bu bilgi standart kütüphane belgesinden ve edition rehberinden geliyor.
