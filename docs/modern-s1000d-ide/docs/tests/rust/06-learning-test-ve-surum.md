---
title: "Learning test ve sürüm"
ikon: 📌
kapak: purple
tarih: 2026-11-08
etiketler: [rust, testler]
ozet: Rust'ta öğrenme testi (learning test) sıradan bir testtir ve her cargo test çalıştırmasında koşar. Sürümü Cargo.lock tutar. Sürüm yalnızca cargo update ya da Cargo.toml değişince değişir, ardından gelen ilk cargo test öğrenme testlerini yeni sürüme karşı çalıştırır. Derleyici API'nin biçimindeki değişikliği yakalar, davranıştaki değişikliği öğrenme testi yakalar. Yükseltme cargo update -p ile küçük adımlarla yapılır. = ile sabitleme, aynı paketi isteyen başka gereksinimlerle çakışabilir ya da herkesi o sürüme çeker. Testler için eklenen bir paket de uygulamanın sürümlerini değiştirebilir.
---

:::callout 📌 purple
**Bir kütüphaneden beklediğin davranışı öğrenme testiyle yaz, sürümü `Cargo.lock` tutsun.** Rust'ta öğrenme testi sıradan bir testtir ve her `cargo test` çalıştırmasında koşar. Kilit dosyası sürümü, sen `cargo update` çalıştırana ya da `Cargo.toml`'u değiştirene kadar sabit tutar. Sürüm değişince ilk `cargo test` öğrenme testlerini yeni sürüme karşı çalıştırır. Yükseltmeyi `cargo update -p` ile paket paket yap. `=` ile sabitlemeyi istisna say, çünkü aynı paketi isteyen başka gereksinimlerle çakışabilir ya da herkesi o sürüme çeker.
:::

Clean Code serisinin [Sınırlar ve learning test](clean_code/06-sinirlar-ve-learning-test.html) notu, James Grenning'in önerisini anlattı: üçüncü taraf bir kütüphane, üretim kodunda denenerek değil, ayrı testlerle tanınır. Bu testler kütüphanenin yeni bir sürümü çıktığında yeniden çalıştırılır. Bu not aynı önerinin Rust'taki karşılığını anlatıyor: öğrenme testi nereye yazılır, sürümü ne sabitler, bir yükseltmede ne yakalanır.

Notun kod örnekleri ve aktarılan çıktılar 8 Kasım 2026'da Rust 1.98.1 ve onunla gelen Cargo ile denendi. Denemeler ağa çıkmadan, makinedeki crate önbelleğiyle yapıldı. Rust Book atıfları kitabın 2 Kasım 2026 tarihli sürümüne dayanıyor.

## Öğrenme testi sıradan bir testtir

Grenning, Clean Code'un *Sınırlar* bölümündeki *Sınırları Keşfetmek ve Öğrenmek* başlığında, kütüphaneyi uygulamada kullanacağımız biçimde çağıran küçük deneyler önerir. *Öğrenme Testleri Bedavadan da İyidir* başlığına göre bu testlerin asıl getirisi sonradan gelir: kütüphanenin yeni bir sürümü çıktığında yeniden çalıştırılırlar ve davranış farkını hemen gösterirler.

Rust'ta bunun için ayrı bir araç gerekmez. Öğrenme testi `#[test]` ile işaretlenmiş bir fonksiyondur, `cargo test` onu öteki testlerle birlikte derler ve çalıştırır. Aşağıdaki dosya XML kütüphanesi roxmltree hakkındaki iki varsayımı sabitliyor. İlki projede de vardı: S1000D dosyalarının satır içi DTD'si varsayılan ayarla reddedilir. İkincisi, `xlink:href` gibi önekli bir özniteliğin yalnızca yerel adıyla aranınca bulunmadığıdır. Dosya roxmltree 0.20 için yazıldı:

```rust
//! Learning tests: what we rely on in roxmltree, checked against the version in Cargo.lock.

use std::assert_matches;

use roxmltree::{Document, Error, ParsingOptions};

const XLINK: &str = "http://www.w3.org/1999/xlink";

/// S1000D files declare their illustrations as entities in an inline DTD.
const ILLUSTRATION_DTD: &str = "<!DOCTYPE dmodule [\
    <!NOTATION svg SYSTEM \"svg\">\
    <!ENTITY ICN-1 SYSTEM \"icn/ICN-1.svg\" NDATA svg>\
    ]><dmodule/>";

#[test]
fn an_inline_dtd_is_rejected_unless_allowed() {
    assert_matches!(Document::parse(ILLUSTRATION_DTD), Err(Error::DtdDetected));

    let options = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };
    assert!(Document::parse_with_options(ILLUSTRATION_DTD, options).is_ok());
}

#[test]
fn a_bare_name_does_not_find_a_prefixed_attribute() {
    let xml = format!(r#"<dmRef xmlns:xlink="{XLINK}" xlink:href="DMC-1"/>"#);
    let document = Document::parse(&xml).unwrap();
    let dm_ref = document.root_element();

    assert_eq!(dm_ref.attribute("href"), None);
    assert_eq!(dm_ref.attribute((XLINK, "href")), Some("DMC-1"));
}
```

**Nerede durur?** Serinin [Testlerin yeri](01-testlerin-yeri.html) notu, yalnızca `src/main.rs`'i olan bir programın fonksiyonlarının `tests/` klasöründen `use` ile alınamadığını anlatmıştı. Öğrenme testi ise bizim kodumuzu değil, bağımlılığı çağırır. Denemede yalnızca `src/main.rs`'i olan bir pakette bu dosya `tests/learning_roxmltree.rs` olarak durdu ve roxmltree'yi doğrudan kullanabildi. `cargo test --test learning_roxmltree` yalnızca bu dosyadaki testleri çalıştırdı.

Bizce iki yerleşimin de yeri var. Test yalnızca kütüphaneyi çağırıyorsa `tests/` altında, kütüphanenin adını taşıyan bir dosyada durabilir. Kütüphanenin davranışını bizim sınır kodumuzla birlikte sabitliyorsa, o kodun yanındaki `#[cfg(test)]` modülünde durmalıdır, çünkü sınır kodu çoğu zaman modülün dışına açık değildir.

## Sürümü `Cargo.lock` tutar

Rust Book'a göre bir bağımlılığın yeni sürümü projeye kendiliğinden girmez. Kitap bunu *Programming a Guessing Game* bölümünde, `Cargo.lock`'u tanıttığı *Ensuring Reproducible Builds* başlığında anlatır. Dosyaya ilk derlemede seçilen sürümler yazılır, sonraki her derleme de o sürümleri kullanır. Kitabın gerekçesi bir senaryodur: aynı serinin yeni bir yama sürümü önemli bir hatayı düzeltirken kodu bozan bir gerileme de getirebilir. Sürümü bilerek değiştirmenin yolu, aynı bölümün *Updating a Crate to Get a New Version* başlığındaki `cargo update` komutudur. Komut kilit dosyasını hesaba katmaz ve `Cargo.toml`'daki gereksinime uyan en yeni sürümleri seçer. Kitap, kilit dosyasının çoğu zaman kaynak kontrolüne eklendiğini de not eder.

Cargo Book'un *Dependency Resolution* sayfası, *Lock file* başlığında kilit dosyasındaki sürümlere en yüksek önceliğin verildiğini söyler. `Cargo.toml`'daki gereksinim değişir ve kilitli sürüm artık ona uymazsa, o kayıt yok sayılır ve uyan en yeni sürüm seçilir. Denemede `log` 0.4.27'ye kilitliyken gereksinimi `"0.4.34"` yaptık. Cargo 0.4.34'ü değil, önbellekteki en yeni uyumlu sürüm olan 0.4.35'i seçti.

Bizce öğrenme testleri için sonuç şu: Grenning'in öğrenme testlerini yeni sürümde yeniden çalıştırma önerisi Rust'ta hatırlamaya kalmaz. Testler her zaman kilit dosyasındaki sürüme karşı koşar. Sürüm ancak `cargo update` ile ya da `Cargo.toml` değiştirilince değişir. İki durumda da değişiklik `Cargo.lock`'ta bir fark olarak görünür ve ardından gelen ilk `cargo test` öğrenme testlerini yeni sürüme karşı çalıştırır. Clean Code serisinin 06 notundaki PMD örneğinde ise yeniden çalıştırmayı tetikleyen bir düzenek yoktu, iş hatırlamaya kalmıştı.

Kilit dosyasının kendiliğinden değişmemesi gereken yerler için `--locked` seçeneği var. Aynı sayfanın *Dependency updates* başlığına göre `--locked` ve `--frozen`, gereksinim değiştiğinde kilit dosyasını güncellemek yerine hata verdirir. Denemede gereksinim değiştirilmişken `cargo test --locked` şu hatayla durdu:

```text
error: cannot update the lock file .../Cargo.lock because --locked was passed to prevent this
```

Cargo Book'un sık sorulan sorular sayfasındaki *Why have Cargo.lock in version control?* yanıtı, kilit dosyasını kaynak kontrolüne eklemenin paketin ihtiyacına bağlı olduğunu söyler, ama getirilerini de sayar. Aynı sürümlerle yapılan derlemeler `git bisect` ile hatanın kaynağını aramayı kolaylaştırır. Sürekli entegrasyonun (CI) da dış etkenlerle değil, yalnızca yeni commit'ler yüzünden kırılmasını sağlar. Yanıt bir uyarı da ekler: kilit dosyası paketi kullananları etkilemez, onlar için yalnızca `Cargo.toml` geçerlidir. Yanıtın ilk örneği `cargo install`'dur: `--locked` verilmezse kurulum kilit dosyasındaki sürümleri değil, en yeni sürümleri kullanır. Bizce `cargo install` ile kurulmayan ve başka bir paketin bağımlılığı olmayan bir uygulamada bu uyarının karşılığı azdır.

## Bir yükseltmede ne yakalanır?

Denemede `Cargo.toml`'daki gereksinimi `"0.20"`'den `"0.21"`'e çektik. Cargo Book'un *Specifying Dependencies* sayfasındaki *Default requirements* başlığına göre Cargo, soldaki ilk sıfır olmayan basamağı aynı olan sürümleri uyumlu sayar. 0.20 ile 0.21 bu yüzden uyumsuzdur ve `cargo update` bu geçişi kendiliğinden yapmaz, gereksinimi elle değiştirmek gerekir. Ardından çalışan `cargo test` kilit dosyasını güncelledi (`Updating roxmltree v0.20.0 -> v0.21.2`). Kod derlendi, ama öğrenme testlerinden biri kırıldı:

```text
---- a_bare_name_does_not_find_a_prefixed_attribute stdout ----

thread 'a_bare_name_does_not_find_a_prefixed_attribute' (24610027) panicked at tests/learning_roxmltree.rs:32:5:
assertion `left == right` failed
  left: Some("DMC-1")
 right: None
```

roxmltree'nin değişiklik günlüğü (CHANGELOG) 0.21.0'ın değişiklikleri arasında bunu sayıyordu: `Node::has_attribute`, `Node::attribute` ve `Node::attribute_node` artık, `Node::has_tag_name` gibi, yerel adla eşleşir. Derleyici bir şey söylemedi, çünkü iki sürümde de bu fonksiyonların imzası aynıydı. Değişen yalnızca davranıştı. Test yeni davranışı anlatacak biçimde yeniden yazıldı ve geçti:

```rust
/// Since 0.21.0 a bare name matches the local name, as `has_tag_name` does.
#[test]
fn a_bare_name_finds_a_prefixed_attribute() {
    let xml = format!(r#"<dmRef xmlns:xlink="{XLINK}" xlink:href="DMC-1"/>"#);
    let document = Document::parse(&xml).unwrap();
    let dm_ref = document.root_element();

    assert_eq!(dm_ref.attribute("href"), Some("DMC-1"));
    assert_eq!(dm_ref.attribute((XLINK, "href")), Some("DMC-1"));
}
```

Cargo Book'un *SemVer Compatibility* bölümü, girişinde kapsamını açıkça sınırlar. Bölüm çoğunlukla, daha önce derlenen kodun artık derlenmemesine yol açan değişiklikleri sınıflandırır. Girişe göre neredeyse her değişiklik, çalışma anındaki davranışı bir ölçüde olumsuz etkileme riski taşır. Böyle bir değişikliğin uyumsuz sayılıp sayılmayacağı ise çoğu zaman bakımcının yargısına kalır. Bizce öğrenme testleri tam bu boşluk içindir: davranıştaki değişiklik sürüm numarasına yansısa da yansımasa da testte görünür.

*Software Engineering at Google* kitabının *Dependency Management* bölümünü Titus Winters yazdı. Bölümün *The Limitations of SemVer* başlığına göre sürüm numarası bir kanıt değil, bakımcının uyumluluk hakkındaki tahminidir. *SemVer Might Overpromise* başlığı bu tahminin neyi kaçırabileceğine örnekler verir: günlük çıktısının biçimini ya da sırasız bir çıktının sırasını değiştirmek, sözleşmenin parçası olmasa da kullanıcıları bozabilir. Clean Code serisinin 06 notu bu bölümü Hyrum Yasası ile birlikte anlatmıştı.

Bizce Rust'ta iş bölümü şöyle: API'nin biçimi değişirse derleyici yakalar, davranışı değişirse öğrenme testi yakalar. Bu örnekteki geçiş, Cargo'nun da uyumsuz saydığı bir geçişti. Rust Book'un senaryosundaki gibi bir gerileme ise bir yama sürümüyle de gelebilir. O zaman `cargo update` onu kendiliğinden getirir ve fark ancak testlerde görünür.

## Yükseltmeyi küçük adımlarla yap

Winters, *Minimum Version Selection* başlığında Russ Cox'un 2018'de Go için anlattığı bir yaklaşımı aktarır: bir bağımlılık ancak gerektiği kadar ileri taşınır, ardından testlerin çalıştırılması beklenir. Yazara göre bağımlılıkları güncellerken küçük adımlar daha güvenlidir. Cargo varsayılan olarak bunun tersini yapar. *Dependency Resolution* sayfasının *Version numbers* başlığına göre genellikle mevcut en yüksek sürümü tercih eder. `cargo update` sayfasına göre de argümansız `cargo update` kilit dosyasındaki bütün paketleri günceller.

Bizce adımı küçük tutmak Cargo'da da mümkün. `cargo update` sayfasındaki seçenekler bunun için yeterli:

- `cargo update --dry-run` neyin değişeceğini gösterir, kilit dosyasına yazmaz.
- `cargo update -p roxmltree` yalnızca o paketi günceller. Sayfaya göre paketin bağımlılıklarına ancak paket başka türlü güncellenemiyorsa dokunulur, öteki paketler kilitli sürümlerinde kalır.
- `cargo update -p roxmltree --precise 0.21.1` paketi belirli bir sürüme götürür. Daha eski bir sürüme dönmek için de kullanılabilir.

Denemedeki çıktılar şöyleydi:

```text
$ cargo update -p log --precise 0.4.27
 Downgrading log v0.4.35 -> v0.4.27

$ cargo update --dry-run
     Locking 1 package to latest Rust 1.98.1 compatible version
    Updating log v0.4.27 -> v0.4.35
warning: not updating lockfile due to dry run
```

Her adımın ardından `cargo test` öğrenme testlerini yeni sürüme karşı çalıştırır. Bizce kilit dosyasındaki fark, testleri geçen kodla aynı commit'e girmelidir. Böylece `git bisect` hangi sürüm değişikliğinin neyi bozduğunu gösterebilir.

Bir yama sürümü yanlışlıkla uyumsuz bir değişiklik getirirse ne yapılacağını *Dependency Resolution* sayfasının *Troubleshooting* altındaki *SemVer-breaking patch release breaks the build* başlığı anlatır. Sayfa önce, sürümü yayımlayan projenin onu geri çekmesini (yank) önerir. Geri çekilene kadar ise çalıştırılabilir bir program gibi en sonda duran bir ürün için önerisi, sorunlu paketi kilit dosyasında güncellememektir. Bunun aracı da `cargo update`'in `--precise` seçeneğidir.

Cargo'nun *Continuous Integration* rehberindeki *Verifying Latest Dependencies* başlığı, en yeni sürümleri denemenin yollarını sayar: kilit dosyasını hiç eklememek, hatası işi durdurmayan ayrı bir CI işi, zamanlanmış bir iş ya da Dependabot gibi araçlarla düzenli güncelleme istekleri. Rehberin örnek işi kararlı Rust'ı güncelledikten sonra `cargo update` çalıştırır, ardından derler ve testleri çalıştırır. Bizce öğrenme testlerinin en çok işe yarayacağı yer böyle bir iştir.

## `=` ne zaman?

Cargo Book `=`'i bir istisna olarak ele alır. *Specifying Dependencies* sayfasının sürüm gereksinimleri kısmındaki öneri kutusu, kararsız kalındığında varsayılan gereksinimi önerir. Dar bir üst sınırın bedelini çözümleme üzerinden anlatır: üst sınırı bir sonraki uyumsuz sürümden önce kesen bir paket, ağaçta daha yeni bir sürüm isteyen başka bir paketle çözülemez bir çakışmaya düşebilir. Kutuya göre sürümü denetlemek için `Cargo.lock` çoğu zaman daha uygun bir yer olabilir. Kutu bu bedelin önemsiz kalabileceği ya da getirinin ağır basabileceği durumları da sayar. Bu projeye uyan durum, pakete başka hiçbir paketin bağımlı olmamasıdır, örneğin paketin yalnızca bir ikili hedefi varsa. Öteki iki durumda tam sürüm (`=`) yerinde olabilir. Biri bir ön sürüme (pre-release) bağlanmaktır. Öteki, bir kütüphanenin yeniden dışa açtığı yordamsal makronun (proc-macro) o kütüphaneyi çağıran kod üretmesidir.

*Dependency Resolution* sayfasının *Recommendations* başlığı da `=`'e küçük bir yer bırakır. İki paket sıkı bağlıysa, örneğin bir kütüphane ile onun yordamsal makro kütüphanesi, `=` ikisinin birlikte hareket etmesini sağlayabilir. Sayfa genel ölçüyü de verir: gereksinimler daraldıkça çözümleyicinin başarısız olma olasılığı artar, gevşedikçe de yeni bir sürümün derlemeyi bozma olasılığı.

Aynı sayfanın *Troubleshooting* altındaki *SemVer-breaking patch release breaks the build* başlığı, geçici bir `=` için de yer açar. crates.io'da yayımlanan bir program, sorunlu sürüm geri çekilene kadar bağımlılığı sorunsuz bilinen bir sürüme `=` ile zorlayabilir. Sayfa aynı programlar için öteki yol olarak, kullanıcılara `cargo install --locked` önermeyi sayar.

Yine *Dependency Resolution* sayfasının *SemVer compatibility* başlığındaki bir örnek, aynı crate için iki farklı `=` gereksiniminin hataya yol açtığını gösterir. Denemede bunu bir dev-dependency üzerinden kurduk. Dev-dependency, `Cargo.toml`'un `[dev-dependencies]` bölümüne yazılan ve yalnızca testler, örnekler ve benchmark'lar derlenirken kullanılan bağımlılıktır. Yardımcı paket `log = "=0.4.27"`, uygulama `log = "=0.4.34"` isteyince Cargo kilit dosyasını üretemedi:

```text
error: failed to select a version for `log`.
...
all possible versions conflict with previously selected packages
```

Bizce yalnızca ikili hedefi olan bir uygulamada `=` başkasına zarar vermez, ama kilit dosyasının zaten yaptığı işe de pek bir şey eklemez. Kattığı şey bir bilgidir: `Cargo.toml`'u okuyan, bu sürümün bilerek seçildiğini görür. Bu bilgi, yanında nedeni yazıyorsa işe yarar.

## Tek çözümleme, tek kilit dosyası

*Dependency Resolution* sayfasının *Dependency kinds* başlığına göre çözümleyici normal bağımlılıklara, derleme betiği bağımlılıklarına ve dev-dependency'lere çoğunlukla aynı biçimde davranır. Saydığı farklardan biri, çalışma alanının (workspace) üyesi olmayan paketlerin dev-dependency'lerinin hesaba hiç katılmamasıdır. Rust Book'un *More About Cargo and Crates.io* bölümündeki *Cargo Workspaces* başlığının *Depending on an External Package* kısmı da bir çalışma alanında tek bir `Cargo.lock` bulunduğunu anlatır. Kitaba göre çalışma alanındaki crate'ler bir bağımlılığın uyumlu sürümlerini istediği sürece hepsi tek bir sürümü paylaşır. Uyumsuz sürümler istendiğinde ise Cargo yine olabildiğince az sürüme çözmeye çalışır.

Bizce bunun test açısından kolay gözden kaçan bir sonucu var: yalnızca testler için eklenen bir paket, uygulamanın kendisiyle derlenen sürümü de değiştirebilir. Denemede uygulama `log = "0.4"` istedi, yalnızca dev-dependency olan yardımcı paket ise `log = "=0.4.27"`. Cargo kilit dosyasına 0.4.27'yi yazdı ve bunu çıktısında da belirtti (`Adding log v0.4.27 (available: v0.4.35)`). Yalnızca normal bağımlılıkları gösteren `cargo tree -e normal` de uygulamanın kendi derlemesinin 0.4.27 ile yapıldığını gösterdi.

Cargo Book'un *Feature resolver version 2* başlığına göre dev-dependency'lerde açılan özellikler (feature), test ya da örnek derlenmedikçe normal derlemeye karışmaz. Sayfa sürümler için böyle bir ayrım anlatmıyor, denemede de görmedik.

## Aynı crate'in iki sürümü

Uyumsuz iki sürüm birleştirilemiyorsa Cargo ikisini de derler (*SemVer compatibility* başlığı). Bunun ne zaman sorun olduğunu *Version-incompatibility hazards* başlığı anlatır: o kütüphaneyi kullanan crate'ler onun türlerini dışarıya açtığında, adı aynı olan iki tür derleyici için ayrı türlerdir. Sayfaya göre bu çoğunlukla derlemede yakalanır. Yakalanmadığı bir durum da verir: bir sürümle oluşturulmuş bir nesneyi öteki sürümün türüne `downcast_ref` ile çevirmek çalışma anında başarısız olur. Yinelenen sürümleri bulmak için önerdiği araç `cargo tree -d`'dir.

Denemede aynı adlı iki yerel crate kurduk: `shape` 1.0.0 ve 2.0.0. İkisinde de aynı `Square` yapısı vardı. Bir ara crate 1.0.0'ı, uygulama 2.0.0'ı kullandı ve uygulama kendi `Square` değerini ara crate'e vermeye çalıştı:

```text
error[E0308]: mismatched types
  expected `shape::Square`, found `Square`
note: there are multiple different versions of crate `shape` in the dependency graph
```

Bizce bir çatının türlerini paylaşmak için `=` gerekmez. Uyumlu bir gereksinim yeter, çünkü çözümleyici uyumlu sürümleri tek bir sürümde birleştirir. Asıl tehlike, çatı uyumsuz bir sürüme geçtiğinde bizim gereksinimimizin eskide kalmasıdır. O zaman iki kopya oluşur ve hata, yukarıdaki gibi aynı adlı iki tür arasında çıkar.

## Clean Code ile bağ

- **Öğrenme testleri.** Grenning'in öğrenme testleri Rust'ta sıradan testlerdir. Bizce kilit dosyası, onları yeni sürümde yeniden çalıştırma önerisini kendiliğinden işletir.
- **Giden sınır testleri.** Grenning, öğrenme testlerine ihtiyaç olsun olmasın, sınırın üretim kodunun kullandığı yoldan sınanmasını ister. Ona göre bu testler olmazsa yeni sürüme geçişi kolaylaştıracak bir dayanak kalmaz ve eski sürümde gereğinden uzun kalmaya özenebiliriz. Bizce sınır kodumuzu kütüphaneyle birlikte sınayan testler bu türdendir ve o kodun modülünde durur.
- **Kontrol edebildiğine dayan.** Grenning *Temiz Sınırlar* başlığında, kontrol edemediğimiz bir şeye değil, edebildiğimiz bir şeye dayanmayı önerir. Bunu üçüncü taraf kodu sarmalamak ve ona adaptörle bağlanmak için söyler. Bizce ilke sürüme de uzatılabilir: hangi sürümle çalıştığımızı kütüphanenin yeni yayınları değil, bizim `Cargo.lock` dosyamız belirler.

## Bu projede yaşadığımız

- **Öğrenilen bilgi önce yorumdaydı, sonra teste girdi.** 6 Kasım 2026'daki ilk sürümde CSDB'nin başlık okuyucusunda, belgeleme yorumu "Learning test:" diye başlayan bir test vardı. Yorum, roxmltree'nin satır içi DTD'yi izin verilmedikçe reddettiğini söylüyordu. Test ise yalnızca bizim okuyucumuzun böyle bir dosyadan başlığı okuyabildiğini denetliyordu. 7 Kasım 2026'nın ilk dakikalarında CSDB'nin XML yardımcıları ayrı bir dosyaya taşınırken teste, roxmltree'nin varsayılan ayarla hata verdiğini denetleyen bir satır eklendi. Bizce test, kütüphanenin varsayılanı değişince ancak bu satırla kırılabilir hâle geldi, çünkü bizim okuyucumuz DTD'ye zaten açıkça izin veriyordu. 7 Kasım 2026'nın sonunda CSDB'nin XML yardımcılarındaki ayrıştırma fonksiyonu `pub(super)` ile yalnızca CSDB modülüne açıktı. Test bu yüzden onun yanındaki `#[cfg(test)]` modülünde duruyordu.
- **Sürümler ilk günden sabitlendi, nedenler yorumlardaydı.** 6 Kasım 2026'daki ilk `Cargo.toml`'da GPUI Kit ve Typst crate'leri `=` ile sabitlenmişti. Yanlarında bir neden yazmıyordu. roxmltree ise `"0.21"` ile eklenmişti. Yorumu, Typst'in zaten derlediği sürümün kullanıldığını söylüyordu. 7 Kasım 2026'da eklenen `tempfile`, `async-io` ve `chrono` da kilit dosyasında başka paketler yüzünden zaten bulunan sürümlerle eklendi, yorumları bunu belirtiyordu.
- **Test bağımlılığının uygulamaya etkisi yorumda yazılıydı.** İlk `Cargo.toml`'daki bir yorum, GPUI testlerinin GPUI Kit'in `test-support` özelliğiyle değil, GPUI'nin kendisiyle açıldığını ve nedenini anlatıyordu. Yoruma göre o özellik kilit dosyasına `gpui-fast`'i getirecek, onun tam sabitlemeleri de `taffy` ile `unicode-properties`'in sürümünü uygulama için de geriye çekecekti. Bu not için o durumu ayrıca denemedik. Aynı yorum, testler için kullanılan GPUI sürümünün GPUI Kit'in sabitlediği sürümle aynı tutulmasını, yoksa testlerin derlenmeyeceğini söylüyordu. GPUI Kit 0.9.0 GPUI'yi `=0.3.11` ile istiyordu, projenin dev-dependency'si de `=0.3.11` idi. Bizce bu uyarıyı gerekli kılan biraz da bizim `=` işaretimizdi. Uyumlu bir gereksinim de aynı sürüme çözülürdü. GPUI Kit sabitlemesini 0.3 serisinde değiştirseydi onu izlerdi. `=0.3.11` ise o durumda kilit dosyasının üretilmesini engelleyecekti.
- **Bir sabitleme gereğinden dardı.** 7 Kasım 2026'nın ilk dakikalarında yazı tamamlama denemesiyle birlikte `lsp-types = "=0.97.0"` eklendi. Yorumu, GPUI Kit bu sürümü sabitlediği için bizimkinin de ona uyması gerektiğini söylüyordu. GPUI Kit'in bileşen crate'i gpui-component 0.9.0 ise `lsp-types`'ı `=` olmadan, `"0.97.0"` ile istiyordu. Bizce türleri paylaşmak için uyumlu bir gereksinim yeterdi. Sabitleme aynı gün öğlen, deneme ve onun öğrenme testleriyle birlikte kaldırıldı.
- **Öğrenme testlerinin yerini sınır testleri aldı.** 7 Kasım 2026 akşamı blok alanına projenin kendi öneri listesini bağlamadan önce, GPUI Kit 0.9.0'ın metin alanının (`TextareaState`) buna izin verip vermediğini yoklayan dört öğrenme testi yazıldı. Testler yalnızca testte derlenen ayrı bir modüldeydi ve modülün başındaki yorum GPUI Kit'in sürümünü adıyla anıyordu. Örneğin imlecin metindeki konumunun bayt olarak verildiğini ve yazılan kelimeyi değiştirmenin tek bir geri alma adımı olduğunu sabitliyorlardı. Yaklaşık 19 dakika sonra özellik eklenince bu testler silindi. Commit iletisine göre gerçek blok alanının testleri onları kapsıyordu. Özelliğin kendisiyle birlikte eklenen testler arasında bir seçimin tek adımda geri alındığını ve aşağı ile yukarı oklarının seçimi taşıdığını sınayanlar vardı. Bizce bu, Grenning'in giden sınır testi önerisine uyuyordu. Kaybedilen, testlerin hangi GPUI Kit sürümüne karşı yazıldığı bilgisiydi. O bilgi artık yalnızca git geçmişindeydi.
- **Denenen sürüm `=` ile sabitlendi.** 7 Kasım 2026 akşamı ACP bağlantı denemesi için `agent-client-protocol = "=5.0.0"` eklendi. Yorumu, sınanan sürüm tam olarak bu olduğu için sabitlendiğini söylüyordu. Bizce bu, `=` işaretinin nedenini yanında taşıyan bir örnekti.
- **Öğrenme testleri hiçbir yükseltmeyle karşılaşmadı.** `Cargo.lock` ilk sürümden beri git'teydi. 7 Kasım 2026'nın sonuna kadar kilit dosyasını değiştiren commit'lerin hiçbirinde daha önce kilitlenmiş bir paketin sürümü değişmedi. Bu commit'ler yalnızca yeni paketler ekledi ya da projenin bağımlılık listesini değiştirdi. Projede bir CI yapılandırması da yoktu. Öğrenme testlerinin yeni bir sürüme karşı yeniden çalıştırılması bu projede henüz hiç gerekmemişti.

## Belgelerin söylemedikleri

- Rust Book'ta da Cargo Book'un bu not için okunan sayfalarında da öğrenme testi (learning test) adı geçmiyor. Bu testlerin nereye yazılacağı ve nasıl adlandırılacağı için bir gelenek yok.
- Cargo bir yükseltmede hangi paketin değiştiğini söylüyor (`Updating roxmltree v0.20.0 -> v0.21.2`), ama o pakete hangi testlerin bağlı olduğunu bilmiyor. Bizce öğrenme testi dosyasını kütüphanenin adıyla adlandırmak, bu bağı kurmanın en kısa yolu.
