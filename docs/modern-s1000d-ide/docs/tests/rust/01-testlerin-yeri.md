---
title: "Testlerin yeri"
ikon: 🗂️
kapak: blue
tarih: 2026-11-07
etiketler: [rust, testler]
ozet: Rust'ta bir testin yerini, o testin neyi görmesi gerektiği belirler. Birim testi sınadığı kodla aynı dosyada durur ve özel öğeleri görür; tests/ klasöründeki test yalnızca kütüphanenin dışa açık API'sini görür; doc test yalnızca kütüphanede çalışır; örneklerin testleri ayrıca açılmadıkça düz cargo test ile çalışmaz. Bizce uç durum ve hata testleri sınadıkları davranışın yanına girer.
---

:::callout 🗂️ blue
**Rust'ta bir testin yerini, o testin neyi görmesi gerektiği belirler.** Birim testi sınadığı kodla aynı dosyadaki `#[cfg(test)]` modülünde durur ve modülün özel öğelerine erişebilir. `tests/` klasöründeki entegrasyon testi ayrı bir crate olarak derlenir ve yalnızca kütüphanenin dışa açık API'sini görür. Programın kodunu `use` ile sınamak için bir `lib.rs` gerekir. Doc test yalnızca kütüphane hedefinde çalışır. Bizce uç durum ve canlıda çıkan hata testleri ayrı bir klasöre değil, sınadıkları davranışın testlerinin yanına girer.
:::

Bu not Rust serisinin ilki. [Clean Code serisi](clean_code/01-neden-temiz-test.html) testin nasıl düşünüleceğini anlattı. Bu seri aynı ilkelerin Rust'ta ve Cargo'da nasıl uygulandığını anlatıyor. İlk soru en temel olanı: test hangi dosyaya yazılır?

Notta geçen birkaç terim:

- **Crate:** Derleyicinin tek seferde derlediği birim. Bir kütüphane ya da çalıştırılabilir bir program olabilir.
- **Hedef (target):** Bir Cargo paketinin ürettiği her kütüphane, program, test dosyası ve örnek.
- **Dışa açık API:** Bir kütüphanenin `pub` ile dışarıya açtığı türler ve fonksiyonlar.
- **Doc test:** Belge yorumundaki kod bloğunun derlenip çalıştırılmasıyla oluşan test.

Notun kod örnekleri 7 Kasım 2026'da Rust 1.98.1 ve onunla gelen Cargo 1.98.1 ile denendi.

## Dört yer

Rust Book'un *Writing Automated Tests* bölümündeki *Test Organization* başlığı, Rust topluluğunun testleri iki ana gruba ayırdığını söyler: birim testleri ve entegrasyon testleri. Cargo bunlara iki yer daha ekler: belge yorumlarındaki örnekler ve `examples/` klasöründeki programlar. Dört yeri birbirinden ayıran şey, testin neyi görebildiği ve `cargo test` çalışınca başına ne geldiğidir.

| Yer | Neyi görür | `cargo test` çalışınca |
|---|---|---|
| Aynı dosyada `#[cfg(test)] mod tests` | Modüldeki her şeyi, özel öğeler dahil | Çalışır |
| `tests/` altındaki her dosya | Yalnızca kütüphanenin dışa açık API'sini | Her dosya ayrı bir program olarak çalışır |
| Belge yorumundaki kod bloğu | Kütüphanenin dışa açık API'sini | Yalnızca kütüphane hedefinde çalışır |
| `examples/` altındaki program | Kütüphanenin dışa açık API'sini | Derlenir, içindeki testler `test = true` olmadan düz `cargo test` ile çalışmaz |

## Birim testi: kodla aynı dosyada

Rust Book birim testlerini, sınadıkları kodla aynı dosyada `tests` adlı bir modüle koymayı bir gelenek olarak anlatır. Modülün üstündeki `#[cfg(test)]`, içindekilerin yalnızca `cargo test` çalışırken derlenmesini sağlar. Testler ve onlara yardım eden fonksiyonlar normal derlemeye girmez.

```rust
pub fn normalize_title(raw: &str) -> String {
    words(raw).collect::<Vec<_>>().join(" ")
}

fn words(raw: &str) -> impl Iterator<Item = &str> {
    raw.split_whitespace()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapses_inner_whitespace() {
        assert_eq!(normalize_title("  Yakıt   filtresi "), "Yakıt filtresi");
    }

    #[test]
    fn treats_tabs_and_newlines_as_spaces() {
        assert_eq!(normalize_title("Sökme\tprosedürü\n"), "Sökme prosedürü");
    }
}
```

`tests` sıradan bir alt modül olduğu için `use super::*` ile üst modüldeki özel `words` fonksiyonuna da ulaşabilir. Rust Book, özel fonksiyonların doğrudan test edilmesinin test topluluğunda tartışmalı olduğunu kabul eder. Kararı okura bırakır: Rust buna izin verir, ama kimseyi zorlamaz.

Google'ın iki kaynağı bu konuda farklı yerlere bakar:

- **Comprehensive Rust** kursunun *Testing* bölümündeki *Unit Tests* sayfası, iç içe `tests` modülünün özel yardımcıları da test etmeye izin verdiğini not eder.
- **Software Engineering at Google** kitabının *Unit Testing* bölümünü Erik Kuefler yazdı. Bölümün *Test via Public APIs* başlığında Kuefler, testlerin sistemi kullanıcılarının çağırdığı biçimde çağırmasını ister. Buradaki "public API" sözünü dilin görünürlük kuralıyla değil, birimin kendi ekibi dışındakilere sunduğu arayüzle tanımlar. Bir birimin sınırını çizmenin bilimden çok sanata yakın olduğunu kabul eder ve birkaç pratik kural verir. Bunlardan birine göre, yalnızca bir iki başka sınıfa hizmet eden bir yardımcı büyük olasılıkla ayrı bir birim sayılmamalı, hizmet ettiği sınıflar üzerinden sınanmalıdır.

Bizce Rust'ta bunun karşılığı şu: `mod tests` özel öğeleri görebilse de birim testi, modülün kendisini çağıranlara sunduğu davranışı sınamalıdır. Yukarıdaki örnekte `words` için ayrı bir test yok, sekme ve satır sonu davranışı `normalize_title` üzerinden sınanıyor. `words` yarın başka bir yolla yazılsa testler değişmez. Özel bir yardımcıya doğrudan test yazmak ancak yardımcı kendi başına bir kural taşıdığında ve o kurala dışarıdan ulaşmak zor olduğunda akla gelir.

## Entegrasyon testi: `tests/` klasöründe

Doğrudan `tests/` altındaki her `.rs` dosyası ayrı bir crate olarak derlenir. Test dosyası kütüphaneyi bir dış kullanıcı gibi `use` ile içeri alır ve yalnızca `pub` öğelere erişir. Yukarıdaki özel `words` fonksiyonunu buradan çağıran bir test derlenmez: derleyici fonksiyonun özel olduğunu bildirir.

```rust
// tests/clean_titles.rs
mod common;

use titles::normalize_title;

#[test]
fn leaves_a_clean_title_alone() {
    assert_eq!(normalize_title(common::CLEAN_TITLE), common::CLEAN_TITLE);
}
```

```rust
// tests/common/mod.rs
pub const CLEAN_TITLE: &str = "Sökme prosedürü";
```

Ortak yardımcının dosya adı önemlidir. Rust Book'un aynı başlığındaki *Submodules in Integration Tests* kısmına göre yardımcı `tests/common.rs` olarak yazılırsa, Cargo onu da bir test dosyası sayar ve çıktıda hiç testi olmayan ayrı bir bölüm belirir. `tests/common/mod.rs` gibi bir alt klasördeki dosya ise test crate'i sayılmaz, yalnızca onu `mod common;` ile bildiren dosyalara katılır.

**`lib.rs` neden gerekir?** Başka crate'lere fonksiyon açabilen tek hedef kütüphanedir. Rust Book'un *Integration Tests for Binary Crates* kısmı bu yüzden, yalnızca `src/main.rs` içeren bir programın fonksiyonlarının `tests/` altından `use` ile alınamayacağını söyler. Kitap aynı ayrımı *An I/O Project* bölümündeki *Separating Concerns in Binary Projects* başlığında başka bir gerekçeyle de önerir: `main` fonksiyonu doğrudan test edilemez. Programın mantığı `lib.rs`'e taşınınca bütün mantık test edilebilir hale gelir. `main`'de argümanları okumak, ayarları kurmak, kütüphaneyi çağırmak ve dönen hatayı ele almak kalır. Kitaba göre bu kadar küçük bir kodun doğruluğu okunarak görülebilir.

Bunun bir istisnası var. Cargo Book'un *Cargo Targets* sayfasındaki *Integration tests* başlığına göre entegrasyon testi olan bir pakette programlar da derlenir. Test, programın yolunu `CARGO_BIN_EXE_<ad>` ortam değişkeninden alıp onu ayrı bir süreç olarak çalıştırabilir. Kütüphanesi olmayan bir komut satırı aracı böylece dışarıdan sınanabilir:

```rust
// tests/cli.rs (yalnızca src/main.rs içeren "stamp" paketinde)
use std::process::Command;

#[test]
fn prints_the_first_page_label() {
    let output = Command::new(env!("CARGO_BIN_EXE_stamp")).output().unwrap();
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "1/2\n");
}
```

Bizce bu yol, çıktısı metin olan araçlara uyar. Penceresi olan bir masaüstü uygulamasını süreç olarak başlatıp çıktısını okumak pratik değil. Böyle bir uygulamada `tests/` ancak bir `lib.rs` ile işe yarar.

*Integration tests* başlığı bir maliyete de değinir. Cargo her test dosyasını ayrı bir program olarak derler ve bu programları sırayla çalıştırır. Bazı durumlarda bu, derlemeyi uzatabilir ve testlerin bütün işlemci çekirdeklerine yayılmasını engelleyebilir. Entegrasyon testi çok olan bir pakete Cargo Book'un tavsiyesi, testleri tek bir test dosyasında toplayıp modüllere bölmeyi düşünmektir. Tek bir programın içindeki testler zaten paralel çalışır.

## Doc test: yalnızca kütüphanede

Belge yorumundaki (`///`) bir kod bloğu, `cargo test` sırasında ayrı bir küçük program olarak derlenip çalıştırılır. Okura API'nin nasıl çağrıldığını gösteren örnek, böylece kodla birlikte eskimez:

````rust
/// Başlıktaki fazla boşlukları teke indirir.
///
/// ```
/// assert_eq!(titles::normalize_title("  Yakıt   filtresi "), "Yakıt filtresi");
/// ```
pub fn normalize_title(raw: &str) -> String {
    words(raw).collect::<Vec<_>>().join(" ")
}
````

Cargo Book'un `cargo test` sayfasındaki *Documentation tests* başlığı, örneklerin kütüphane hedefinin belge yorumlarından çıkarıldığını söyler. Aynı başlık, her bloğun ayrı bir program olarak çalıştırılması biçiminin garanti olmadığını ve ileride değişebileceğini de not eder. *Cargo Targets* sayfasındaki *The doctest field* başlığı da bu ayarın yalnızca kütüphaneler için anlamlı olduğunu belirtir. Yani yalnızca çalıştırılabilir bir crate'te belge yorumundaki örnek hiç çalışmaz.

Bunu denedik. Kütüphanesi olmayan bir programda bir fonksiyonun belge yorumuna, çalışsa kesinlikle başarısız olacak `assert!(false)` içeren bir blok koyduk. `cargo test` hata vermedi, çıktıda doc test bölümü hiç yoktu. `cargo test --doc` ise paketin kütüphane hedefi olmadığını söyleyip durdu.

Bizce doc test, belgedeki örneğin doğru kalmasını sağlar, davranış testinin yerini tutmaz. Uç durumlar ve hata yolları birim ya da entegrasyon testlerinde kalır.

## Örnekler: derlenir, testleri kendiliğinden çalışmaz

`examples/` klasöründeki programlar, kütüphanenin nasıl kullanılacağını gösteren ya da geliştirmeye yardım eden küçük araçlardır. Cargo Book'un *Cargo Targets* sayfasındaki *The test field* başlığına göre `cargo test` örnekleri yalnızca derlenmeye devam ettiklerinden emin olmak için derler. Bir örneğin içindeki `#[test]` fonksiyonları, düz `cargo test` ile ancak `Cargo.toml`'da o örnek için `test = true` yazılmışsa çalışır:

```toml
[[example]]
name = "tool"
test = true
```

Bu ayar örneği bir test programı olarak derler ve `main` fonksiyonunun yerine Rust'ın test çalıştırıcısını koyar. Ayar yokken düz `cargo test` örneğin testlerini sessizce atlar: ne başarısız olurlar ne de çıktıda görünürler. `cargo test` sayfasının *Target Selection* başlığına göre bir örneği adıyla seçen `cargo test --example <ad>` ise bu ayara bakmaz ve örneği her durumda test eder.

Denemede ayar eklenince çıktıda örnek programın kendi test bölümü belirdi. Ayar olmayan başka bir pakette, `--example` ile adı verilen örneğin testleri de çalıştı.

## Bir hedef kırılınca ötekiler

`cargo test`, paketteki her test hedefini ayrı bir program olarak sırayla çalıştırır. Cargo Book'un `cargo test` sayfasındaki `--no-fail-fast` açıklamasına göre bir program başarısız olunca Cargo durur ve sonraki hedefleri çalıştırmaz. Aynı programın içindeki testlerin hepsi yine de sonuna kadar koşar. Rust Book da birim testlerinden biri başarısız olursa entegrasyon ve doc test bölümlerinin çıktıda hiç görünmeyeceğini hatırlatır.

Denemede kütüphanenin bir birim testini bilerek bozduk. `cargo test` yalnızca kütüphanenin birim testlerini çalıştırıp durdu. Entegrasyon testi, örnek programın testi ve doc test hiç çalışmadı. `cargo test --no-fail-fast` ise bütün hedefleri çalıştırdı ve kırılan hedefleri sonda tek tek listeledi.

Bizce bunun yer sorusuyla bir ilgisi var: testler ne kadar çok hedefe dağılırsa, ilk kırılan hedef ötekilerin sonucunu o kadar çok gizler. Sürekli entegrasyonda `--no-fail-fast`, hangi testlerin gerçekten kırıldığını tek seferde gösterir.

## Test yardımcıları nerede durur?

Birden fazla modülün birim testleri ortak bir yardımcı kullanıyorsa, yardımcı `#[cfg(test)]` ile işaretlenmiş bir modüle konabilir. Böylece yalnızca test derlemesinde var olur.

Bu yardımcı `tests/` klasöründen görünmez. Entegrasyon testi kütüphaneyi normal derlenmiş haliyle kullanır ve o derlemede `cfg(test)` açık değildir. Denemede `tests/` altından böyle bir yardımcıyı çağırdık. Derleyici yardımcıyı bulamadığını söyledi ve aynı adda bir öğenin derlemeden yapılandırma yüzünden çıkarıldığını not etti.

Entegrasyon testlerinin yardımcısı bu yüzden ya `tests/common/mod.rs` içinde durur ya da kütüphane onu bir Cargo özelliğinin (feature) arkasında dışa açar:

```toml
[features]
test-support = []
```

```rust
#[cfg(any(test, feature = "test-support"))]
pub mod test_support {
    pub const MESSY_TITLE: &str = "  Yakıt   filtresi ";
}
```

Bu yardımcıyı kullanan entegrasyon testi, `cargo test --features test-support` ile çalışır. Testin kendisini `Cargo.toml`'daki `[[test]]` girdisinde `required-features` ile bu özelliğe bağladık. Özellik açılmadan çalıştırılan `cargo test` bu testi uyarı vermeden atladı.

GPUI de kendi test araçlarını aynı biçimde, `test-support` özelliğinin arkasında sunar. Ayrıntılar serinin GPUI testleri notunda.

## Uç durum ve canlı hata testleri nereye girer?

Clean Code serisinin [Kapsam ve uç durumlar](clean_code/05-kapsam-ve-uc-durumlar.html) notu bu soruyu dile bağlı olmadan yanıtlamıştı. Testler nereden geldiklerine göre değil, neye dokunduklarına ve ne zaman çalıştırıldıklarına göre ayrılır. Hata testi, sınadığı davranışın testlerinin yanına girer ve adı hatanın numarasını değil davranışı anlatır.

Rust'ta bunun bir anlamı daha var: klasör değiştirmek görünürlüğü de değiştirir. Uç durumu sınayan test bir modülün içindeki bir kurala dayanıyorsa, onu `tests/` klasörüne taşımak o kuralı testten saklar.

Rust Book da Cargo Book da hata testlerinin yerinden söz etmez. Aşağıdaki yerleşim bizim çıkarımımız:

| Test | Yeri |
|---|---|
| Bir modülün davranışı, uç durumları ve o davranışta çıkan hatanın testi | Aynı dosyada `#[cfg(test)] mod tests` |
| Kütüphanenin dışarıya sunduğu, birkaç modülü birlikte kullanan bir akış | `tests/<özellik>.rs`, kütüphane hedefi varsa |
| Üçüncü taraf bir kütüphanenin davranışını sabitleyen learning test | Serinin learning test notunun konusu |
| Yavaş ya da belirli bir ortam isteyen test | Serinin her ortamda çalışma notunun konusu |

## Clean Code ile bağ

- **Hata testi davranışın yanında.** Clean Code serisinin *Kapsam ve uç durumlar* notu, Martin'in övdüğü JUnit test paketinde hata numarası taşıyan testin sınıfın öteki testleriyle aynı yerde durduğunu anlatıyordu. Rust'ın `mod tests` geleneği buna benzer ve bir adım ileri gider: test, sınadığı kodla aynı dosyada durur.
- **Bütün testler tek komutla.** Clean Code'un *Kokular ve Sezgisel Kurallar* bölümündeki *Testler Birden Fazla Adım Gerektirir* başlığı, bütün birim testlerinin tek bir komutla çalıştırılabilmesini temel bir gereklilik sayar. Rust'ta tek bir `cargo test` dört yerin hepsini kapsayabilir. Bizce bunun için üç koşul gerekir: örneklerin testleri `test = true` ile açılmıştır, doc testler bir kütüphanede durur ve kırılan ilk hedef ötekileri gizlemesin diye `--no-fail-fast` kullanılır.
- **Testin dayandığı arayüz.** Clean Code serisinin [Soyutlamanın arkasından test](clean_code/07-soyutlamanin-arkasindan-test.html) notu, sınanan kodun bağımlılığını bir soyutlamanın arkasına alıp testte onun yerine sabit bir uygulama koymayı anlatıyordu. Bizce o not da bu not da aynı yöne bakar: test, kodun iç ayrıntılarına değil bir arayüze dayandığında iç yapı değişse de ayakta kalır. `mod tests` özel ayrıntılara uzandıkça bu güvence zayıflar.

## Bu projede yaşadığımız

- **Kütüphane hedefi olmadan başladık.** 6 Kasım 2026'daki ilk sürümde proje yalnızca `src/main.rs` üzerinden derlenen bir programdı. `lib.rs` ve `tests/` klasörü yoktu. 7 Kasım 2026'da, PDF çıktısı ana dala birleştikten sonra da durum aynıydı. O gün projedeki 126 testin hepsi kaynak dosyaların içindeki `#[cfg(test)]` modüllerindeydi: 86 sıradan Rust testi, ekranda görünmeyen pencerelerle çalışan 36 GPUI testi ve bellek ölçme aracının 4 testi.
- **Ortak test yardımcısı kök modüldeydi.** Her teste ayrı bir geçici klasör veren yardımcı, programın kök modülünde `#[cfg(test)]` ile tanımlanmıştı. Önizleme paneli, blok düzenleyici, CSDB paneli, dosya izleme ve PDF kaydetme gibi pek çok modülün testi onu kullanıyordu. Bizce proje bir gün `lib.rs` açıp `tests/` klasörüne geçerse bu yardımcı oradan görünmez. `tests/common` altına taşınması ya da bir özelliğin arkasına alınması gerekir.
- **Hata testleri kendi dosyalarına girdi.** 6 ve 7 Kasım 2026'daki üç hata düzeltmesinin testleri, düzeltilen davranışın dosyasındaki test modülüne eklendi ya da oradaki bir testi genişletti: belge modeli, gereksinim tabloları ve PDF kaydetme. Ayrıntılar Clean Code serisinin *Kapsam ve uç durumlar* notunda.
- **Learning test, sınadığı kodla birlikte taşındı.** Projenin XML okuyucusunun satır içi DTD'yi kabul ettiğini gösteren test, 6 Kasım 2026'da başlık okuma modülünün test modülündeydi. roxmltree kütüphanesinin böyle bir DTD'yi varsayılan olarak reddettiği o gün yalnızca testin belgeleme yorumunda yazıyordu. 7 Kasım 2026'da XML okuma kodu ayrı bir modüle ayrıldığında test de onunla birlikte o modülün test modülüne geçti. Kütüphanenin reddettiğini denetleyen satır da o sırada eklendi.
- **Bellek ölçme aracı bir örnek oldu.** 7 Kasım 2026'da önizlemenin bellek kullanımını ölçen betik, `examples/` altında bir Rust örneğine dönüştürüldü. Araç, çalışan IDE'nin bellek kullanımını dışarıdan, ayrı bir süreç olarak ölçüyordu. Testleri kendi dosyasındaydı. Aynı değişiklik `Cargo.toml`'a o örnek için `test = true` ekledi. Yanındaki yorum, örneklerin `cargo test` ile yalnızca derlendiğini ve bu örneğin çalıştırılacak testleri olduğunu hatırlatıyordu.
- **Doc test yoktu.** 7 Kasım 2026'da projenin belge yorumlarında hiç kod bloğu yoktu. Olsaydı da kütüphane hedefi olmadığı için çalışmayacaktı.

## Tuzaklar

- **Ortak yardımcı test dosyası sanılır.** `tests/common.rs` çıktıda boş bir test bölümü olarak görünür. `tests/common/mod.rs` kullanılır.
- **`#[cfg(test)]` yardımcısı `tests/` klasöründen görünmez.** Entegrasyon testleri için yardımcı ya `tests/common` altında durur ya da bir özelliğin arkasında dışa açılır.
- **Bazı testler sessizce atlanır.** Düz `cargo test`, `test = true` olmayan örneklerin testlerini, kütüphanesi olmayan bir crate'teki doc testleri ve `required-features` özelliği açılmamış entegrasyon testlerini çalıştırmaz. Bunun için uyarı da vermez.
- **İlk kırılan hedef ötekileri durdurur.** Bütün sonucu görmek için `cargo test --no-fail-fast` kullanılır.
- **Her entegrasyon test dosyası ayrı bir programdır.** Dosyalar çoğaldıkça derleme bazı durumlarda uzayabilir. Cargo Book, çok sayıda entegrasyon testini tek dosyada modüllere bölmeyi düşünmeyi önerir.

## Belgelerin söylemedikleri

- Rust Book da Cargo Book da uç durum ve hata testlerinin hangi modülde ya da klasörde duracağını söylemez.
- Rust Book, özel fonksiyonların test edilip edilmeyeceğini tartışmalı bırakır ve bir öneri vermez.

Projenin yapısı tek dosyadan workspace'e doğru büyürken testlerin nereye taşındığı, serinin mimari evrim notunun konusu olacak.
