---
title: "Mimari evrim ve testler"
ikon: 🧱
kapak: brown
tarih: 2026-11-08
etiketler: [rust, testler, mimari]
ozet: Rust'ta kod modülden kütüphaneye, oradan ayrı crate'lere taşındıkça testin görebildiği şey değişir. Aynı dosyadaki test modülü kodla birlikte taşınır, ama üst modüllerdeki test yardımcılarına dayanıyorsa kırılır. Başka yerden yazılmış testler yalnızca açık bırakılanı görür. Kök paketi olan bir workspace'te, default-members yoksa, kökte çalıştırılan düz cargo test yalnızca kök paketi sınar. Sınanan crate'in türlerini döndüren ayrı bir test yardımcısı crate'i, o crate'in birim testlerinde tür uyuşmazlığına yol açar. Özel yardımcıları değil, dışa açık arayüzü sınayan testler yeniden düzenlemelerde değişmeden kalır.
---

:::callout 🧱 brown
**Rust'ta mimari büyüdükçe sınırlar testin görebildiğini değiştirir.** Aynı dosyadaki `mod tests` kodla birlikte taşınır ve modülün kendi özel öğelerini görmeye devam eder, ama üst modüllerdeki `#[cfg(test)]` yardımcılarına dayanıyorsa kırılır. Başka bir modülden ya da crate'ten yazılmış testler yalnızca açık bırakılanı görür: crate sınırını `pub(crate)` geçemez. Kök paketi olan bir workspace'te düz `cargo test` üye crate'lerin testlerini çalıştırmaz, `--workspace` ya da `default-members` gerekir. Sınanan crate'in türlerini döndüren ayrı bir test yardımcısı crate'i, o crate'in birim testlerinde tür uyuşmazlığına yol açar. Yeniden düzenlemelerde değişmeden kalan testler, özel yardımcıları değil dışa açık arayüzü sınayanlardır.
:::

Clean Code serisinin [Mimariyi testle sürmek](clean_code/08-mimariyi-testle-surmek.html) notu kitabın görüşünü anlattı: mimari baştan büyük tasarlanmaz, kaygılar ayrıldığı ve her adımı otomatik testler koruduğu sürece küçük adımlarla büyütülür. Bu not serinin son parçası ve aynı büyümenin Rust'ta testlere ne yaptığını anlatıyor: kod tek bir programın modüllerinden kütüphaneye, oradan bir workspace'in ayrı crate'lerine taşınırken hangi testler yerinde kalır, hangileri değişir, hangileri sessizce çalışmaz.

Notun deneyleri 8 Kasım 2026'da Rust 1.98.1 ile yapıldı. Aktarılan çıktılar bu deneylerden alındı. Rust Book atıfları kitabın 2 Kasım 2026 tarihli sürümüne dayanıyor.

## Testin gördüğü yer sınırla değişir

Rust Reference'ın *Visibility and Privacy* bölümü temel kuralı koyar: özel bir öğeye, tanımlandığı modül ve o modülün alt modülleri erişebilir. Aynı bölüm görünürlüğü daraltan biçimleri de tanımlar: `pub(super)` öğeyi üst modüle, `pub(crate)` ise bulunduğu crate'in tamamına açar. Testler için bunun sonucu şu tabloda:

| Kodun yeri | Aynı yerdeki `mod tests` | Başka yerden yazılan test |
|---|---|---|
| Bir programın modülü | Modülün ve üst modüllerinin özel öğelerini görür | Crate'in öteki modülleri öğe `pub(crate)` ya da `pub` ise, ona giden yoldaki modüller de görünür durumdaysa görür; `pub(super)` yalnızca üst modüle ve onun alt modüllerine açıktır |
| Kütüphane hedefi (`lib.rs`) | Aynı | `tests/` klasöründeki test yalnızca `pub` öğeleri görür |
| Workspace'te ayrı bir crate | Kendi crate'inin özel öğelerini görür; taşınmadan önce dayandığı üst modül yardımcıları artık başka crate'tedir | Öteki crate'ler yalnızca `pub` öğeleri görür |

Bizce tablonun söylediği şu: `mod tests` en dar sınırın içinde durduğu için kod nereye taşınırsa taşınsın onunla gider ve modülün kendi özel öğelerini görmeye devam eder. Ama bir test, üst modüllerdeki ya da kök modüldeki `#[cfg(test)]` yardımcılarına dayanıyorsa, modül başka bir crate'e taşındığında o yardımcıları göremez ve kırılır. Kodun dışından yazılmış testler için de sınır daralır. Serinin [Testlerin yeri](01-testlerin-yeri.html) notu, `#[cfg(test)]` ile işaretli yardımcıların `tests/` klasöründen görünmediğini göstermişti. Workspace'teki öteki crate'ler için de durum aynı: denemede üye crate'e `#[cfg(test)]` ile eklenen bir yardımcıyı kök paketin testi çağıramadı, derleyici `E0425` ile fonksiyonu bulamadığını söyledi.

Denemede bir fonksiyonu `pub(crate)` yapıp aynı workspace'teki başka bir crate'ten çağırdık. Hem programın kodu hem testi şu hatayla derlenmedi:

```text
error[E0603]: function `panel_title` is private
```

## Workspace'te `cargo test` neyi çalıştırır?

Rust Book'un *More About Cargo and Crates.io* bölümündeki *Cargo Workspaces* başlığı, *Adding a Test to a Workspace* kısmında kendi örneğindeki gibi kurulmuş bir workspace'in kökünde `cargo test` çalıştırınca bütün crate'lerin testlerinin koştuğunu, tek bir crate için de `-p` kullanılabileceğini gösterir. Kitabın örneğindeki kök `Cargo.toml` yalnızca bir `[workspace]` bölümünden oluşur, yani bir paket tanımlamaz.

Bu ayrıntı önemlidir, çünkü workspace kökünde paket seçilmeden çalışan bir komutun hangi crate'leri kapsadığı ona bağlıdır. Cargo Book'un *Workspaces* sayfası iki biçimi tanımlar: `[workspace]` bölümü bir paketin kendi `Cargo.toml` dosyasındaysa o paket kök pakettir, paket tanımlamayan bir kök dosya ise sanal bir manifesttir ve böyle bir workspace sanal (virtual) workspace sayılır. Sayfanın *Package selection* başlığına göre seçim yapılmadığında komut, bulunduğu klasördeki paketi, workspace kökündeyse `default-members` listesini kullanır. Bu liste yazılmamışsa ne olacağını *The default-members field* başlığındaki bir not söyler: kök paketi olan workspace'te yalnızca kök paket, sanal workspace'te bütün üyeler.

Denemede kök paketi olan küçük bir workspace kurduk: program kökte, belge modülü mantığı ayrı bir üye crate'te:

```toml
[package]
name = "ide"
version = "0.1.0"
edition = "2024"
publish = false

[workspace]
members = ["crates/dm"]

[dependencies]
dm = { path = "crates/dm" }
```

Düz `cargo test` yalnızca programın testini çalıştırdı, üye crate'in testinden hiç söz etmedi:

```text
     Running unittests src/main.rs (target/debug/deps/ide-9d0ef6f3271070cc)
test tests::the_panel_shows_the_title ... ok
```

Üye crate'in testi ancak `cargo test --workspace` ile çalıştı. `[workspace]` bölümüne `default-members = [".", "crates/dm"]` eklenince düz `cargo test` de ikisini çalıştırdı.

Langr, Clean Code'un *Ortaya Çıkış* bölümünde Kent Beck'in basit tasarım kurallarını aktarır. İlkinin anlatıldığı *Basit Tasarım Kuralı 1: Tüm Testleri Çalıştırır* başlığına göre, kapsamlı biçimde sınanan ve bütün testlerini her zaman geçen bir sistem test edilebilirdir. Langr testlerin sürekli çalıştırılmasını da ister. Clean Code serisinin [Kapsam ve uç durumlar](clean_code/05-kapsam-ve-uc-durumlar.html) notu da bütün testlerin tek komutla çalışması gerektiğini anlatmıştı. Bizce kök paketi olan bir workspace'te bu kuralın ön koşulu, o tek komutun bütün crate'leri kapsamasıdır: ya `default-members` yazılır ya da komut `cargo test --workspace` olur. Yoksa üye crate'lerin testleri hiç kırılmadan, hiç çalışmadan kalabilir.

## Test yardımcısı için ayrı bir crate

Testlerde tekrar eden kurulum kodu büyüyünce onu ayrı bir crate'e almak akla gelir: örneğin örnek belge modülleri üreten bir test yardımcısı crate'i. Bu crate, sınadığı crate'e bağımlıdır. Sınanan crate de onu `[dev-dependencies]` altında kullanmak ister.

Cargo böyle bir döngüye izin verir. Sorun, Cargo Book'un *Dependency Resolution* sayfasındaki *dev-dependency cycles* başlığına göre, birim testlerinde çıkar. O testler derlenirken test programına kütüphane iki kez girer: yardımcı crate'in kullandığı normal kopya ve birim testlerini taşıyan kopya. Bu iki kopyanın türleri birbirinin yerine geçmez. Sayfa bu yüzden, yardımcı crate sınanan crate'in türlerini dışarıya verdiğinde dikkatli olunmasını ve mümkünse paketlerin döngü kalmayacak biçimde yeniden düzenlenmesini önerir.

Denemede bunu kurduk. Yardımcı crate tek bir fonksiyon sunuyordu:

```rust
/// A module with a title, for tests in any crate.
pub fn titled(title: &str) -> dm::Module {
    dm::Module { title: title.into() }
}
```

Belge crate'i onu `[dev-dependencies]` altına ekledi ve kendi birim testinde kullandı:

```rust
#[test]
fn a_title_from_the_testkit_is_shown() {
    let module = dm_testkit::titled("Yakıt filtresi");
    assert_eq!(panel_title(&module, "DMC-1.xml"), "Yakıt filtresi");
}
```

Test derlenmedi. Derleyici aynı adlı iki türü ayırdı ve sorunun yerini doğru gösterdi:

```text
error[E0308]: mismatched types
   |                    ----------- ^^^^^^^ expected `Module`, found `dm::Module`
note: there are multiple different versions of crate `dm` in the dependency graph
```

Aynı test belge crate'inin `tests/` klasörüne taşınınca geçti. Bizce bunun nedeni, entegrasyon testinin kütüphanenin yalnızca normal derlenmiş kopyasını görmesi ve yardımcı crate'in de aynı kopyaya bağlı olması:

```rust
#[test]
fn a_title_from_the_testkit_is_shown() {
    let module = dm_testkit::titled("Yakıt filtresi");
    assert_eq!(dm::panel_title(&module, "DMC-1.xml"), "Yakıt filtresi");
}
```

Bizce bundan çıkan yerleşim şu: sınanan crate'in türlerini döndüren bir test yardımcısı crate'i, öteki crate'lerin testlerine ve sınanan crate'in `tests/` klasörüne hizmet eder. Sınanan crate'in kendi birim testleri ise kendi içindeki `#[cfg(test)]` yardımcılarını ya da Testlerin yeri notunda anlatılan `test-support` özelliğini kullanır.

## Hangi testler değişmeden kalır?

*Software Engineering at Google* kitabının *Unit Testing* bölümünü Erik Kuefler yazdı. Bölümün *Strive for Unchanging Tests* başlığı, üretim kodundaki değişiklikleri dört türe ayırır: saf yeniden düzenleme, yeni özellik, hata düzeltmesi ve davranış değişikliği. Kuefler'e göre mevcut testlerin değişmesi gereken tek tür davranış değişikliğidir. Bir yeniden düzenleme testleri değiştiriyorsa ya değişiklik sistemin davranışını etkiliyordur, yani saf bir düzenleme değildir, ya da testler uygun bir soyutlama düzeyinde yazılmamıştır.

Aynı bölümün *Test via Public APIs* başlığı bunun en önemli yolunu, testlerin sistemi kullanıcılarının çağırdığı gibi çağırması olarak gösterir. Kuefler burada "dışa açık API"yi dilin görünürlük kurallarıyla değil, birimin sahibi olan ekibin dışındakilere sunulan arayüzle tanımlar ve ikisinin her zaman örtüşmediğini belirtir. Birimin sınırını çizmek için verdiği ölçülerin özü şu: yalnızca bir iki yere hizmet eden yardımcılar büyük olasılıkla onları kullananlar üzerinden sınanır. Geniş kullanıma açık ya da genel bir iş gören kod ise doğrudan sınanır, bu bir miktar tekrar yaratsa bile.

Bizce `mod tests`'in özel öğeleri görebilmesi, Rust'ta Kuefler'in bu ölçüsünü çiğnemeyi çok kolaylaştırır: özel bir yardımcıyı doğrudan çağıran bir test yazmak bir satırlık iştir. Böyle bir test, yardımcının imzası her değiştiğinde onunla birlikte değişir. Modülün `pub` arayüzü üzerinden yazılmış bir test ise iç düzenlemelerde değişmeden kalır. Modül ayrı bir crate'e taşındığında da çoğu zaman yalnızca `use` yolu değişir.

## Clean Code ile bağ

- **Sistem mimarisini test güdümlü geliştirin.** Wampler'ın *Sistemler* bölümündeki bu öneri, kaygıları ayrılmış bir mimarinin testlerle büyütülebileceğini söyler. Bizce Rust'ta crate sınırı bu ayrımı derleyiciye denetletir: bir crate yalnızca açıkça `pub` yapılanı görür.
- **Tüm testleri çalıştırır.** Kent Beck'in, Langr'ın aktardığı ilk basit tasarım kuralı. Bizce workspace'te bu kuralın ön koşulu, `cargo test --workspace` ya da `default-members` ile bütün crate'lerin testlerinin tek komutla çalışmasıdır.
- **Kademecilik üzerine.** Martin'in *Ardışık İyileştirme* bölümündeki bu başlık, yapının tek seferde değil küçük adımlarla, her adımda testler geçerken değiştirilmesini ister. Bizce Rust'ta bunun sırası modül, kütüphane hedefi ve ayrı crate'tir. Her adımda önce testlerin hâlâ çalıştığı görülür.

## Bu projede yaşadığımız

- **Proje büyüdü, tek crate olarak kaldı.** 6 Kasım 2026'daki ilk sürümde `src` altında 12 Rust dosyası ve yaklaşık 1 300 satır vardı. 7 Kasım'ın sonunda 28 dosya ve yaklaşık 11 500 satır, 8 Kasım 2026 sabahı (10.57'deki commit) 43 dosya ve yaklaşık 19 800 satır oldu. Üç anda da proje tek bir program crate'iydi: `lib.rs`, `tests/` klasörü ve workspace yoktu. Clean Code serisinin 08 notu bunu 7 Kasım için "tek paket, modüller" diye anlatmıştı. Durum 8 Kasım sabahı da aynıydı.
- **Modüller birbirine üst modül üzerinden açılıyordu.** Aynı üç anda `pub(super)` sırasıyla 3, 57 ve 65 satırda geçiyordu, `pub(crate)` ise 0, 0 ve 3 satırda. O gün `pub(super)` satırlarının hepsi üst düzey modüllerin, örneğin blok editörünün, belge modelinin ya da CSDB okuyucusunun içindeydi. Bizce böyle bir modül bütünüyle ayrı bir crate'e taşınırsa bu sınırlar yeni crate'in içinde kalır. `pub` kararına dönüşecek olanlar crate sınırını geçecek öğelerdir, örneğin o gün `pub(crate)` olan üç satır. Modüllerin kendi `mod tests` blokları onlarla birlikte gider, ama kök modüle bağlı geçici klasör yardımcısını kullanan testler ona yeni bir yoldan ulaşmak zorunda kalır.
- **Uçtan uca testler crate'in içinde kaldı.** 8 Kasım 2026'da gece saat üçte, BREX denetimine gerçek XPath ve XSD motoruyla çalışan uçtan uca beklentiler eklendi. Bunlar `tests/` klasörüne değil, BREX modülünün içinde yalnızca testte derlenen ayrı bir modüle yazıldı. Bizce bunun bir nedeni kütüphane hedefinin olmamasıydı. Bu modül bir entegrasyon testinin işini görüyordu, ama BREX modülünün özel öğelerini de görebiliyordu.
- **Bir yeniden düzenleme testleri değiştirdi.** 7 Kasım 2026 akşamı ACP bağlantı yoklamasında "refactor" diye işaretlenen bir commit, ajanın cevap verip vermediğine karar veren özel yardımcının imzasını değiştirdi: yardımcı tüm tur kaydı yerine yalnızca durma nedenini ve ajanın metnini almaya başladı. Aynı commit, modülün dışarıya verdiği tur kaydı türünün alanlarını da değiştirdi ve durma nedenini isteğe bağlı olmaktan çıkardı. Aynı dosyadaki testler bu özel yardımcıyı doğrudan çağırıyordu. Çağrıları yeniden yazıldı ve testlere kayıt kuran küçük bir yardımcı silindi. Beklenen sonuçlar ise değişmedi. Bizce Kuefler'in ölçüsüyle bu commit saf bir yeniden düzenleme sayılmazdı, çünkü modülün arayüzünü de değiştiriyordu. Testlerin değişmesinin asıl nedeni ise özel bir yardımcının düzeyinde yazılmış olmalarıydı. Değişmeyen beklentiler, ajanın cevabına verilen kararın korunduğunu gösteriyordu.

## Belgelerin söylemedikleri

- Rust Book'ta `pub(crate)` ve `pub(super)` hiç geçmez. Testin görebildiğini belirleyen bu biçimler Rust Reference'ta anlatılır.
- Rust Book'un workspace örneği sanal bir workspace'tir. Kitap, kök paketi olan bir workspace'in kökünde, `default-members` yazılmamışsa düz `cargo test`'in yalnızca kök paketi sınadığından söz etmez. Cargo Book bunu *The default-members field* başlığındaki bir notta söyler.
- Ne Rust Book ne Cargo Book, bir crate bölünürken testlerin nasıl taşınacağını anlatır.
