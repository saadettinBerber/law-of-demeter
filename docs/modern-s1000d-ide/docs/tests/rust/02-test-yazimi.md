---
title: "Test yazımı"
ikon: ✍️
kapak: green
tarih: 2026-11-07
etiketler: [rust, testler]
ozet: Rust'ta iyi bir test, kırıldığında ne olduğunu kendisi söyler. Adı davranışı anlatır ve cargo test filtresinde de iş görür. Döngüyle yazılan tablo testi ilk başarısız vakada durur; assert!(matches!(...)) gelen değeri göstermez, assert_matches! gösterir; ? ile kırılan test satır numarası vermez; #[track_caller] yardımcıdaki hatayı testin satırına bağlar.
---

:::callout ✍️ green
**Rust'ta iyi bir test, kırıldığında ne olduğunu kendisi söyler.** Adı davranışı anlatır. Kırıldığında, beklenen değerle gelen değer çıktıda yan yana görünür. Bunun için doğru makroyu seçmek gerekir: `assert_eq!` ve `assert_matches!` gelen değeri basar, `assert!(matches!(...))` basmaz. Döngüyle yazılan tablo testi ilk başarısız vakada durur. `?` ile kırılan test satır numarası vermez. Test yardımcısına eklenen `#[track_caller]`, hatayı yardımcının değil testin satırına bağlar.
:::

Clean Code serisinin [Temiz test nasıl yazılır?](clean_code/02-temiz-test-nasil-yazilir.html) notu temiz testin ne olduğunu anlattı: okunabilirlik, kur-işlet-denetle, testlere özgü bir dil ve her testte tek kavram. Bu not aynı kuralların Rust'ın test araçlarıyla nasıl uygulandığını anlatıyor. Testlerin hangi dosyaya yazılacağı serinin [Testlerin yeri](01-testlerin-yeri.html) notunun konusuydu.

Notun kod örnekleri ve aktarılan hata çıktıları 7 Kasım 2026'da Rust 1.98.1 ile denendi. Çıktılardan iş parçacığı numaraları gibi konuyla ilgisiz ayrıntılar çıkarıldı.

## Kur, işlet, denetle

Rust Book'un *Writing Automated Tests* bölümündeki *How to Write Tests* başlığına göre tipik bir test önce ihtiyaç duyduğu durumu kurar, ardından sınanan kodu çağırır, en sonda da sonucu bekleneniyle karşılaştırır. Bizce bu üç adım, Clean Code'daki kur-işlet-denetle deseniyle ve Kuefler'in given-when-then parçalarıyla örtüşür. Rust bunun için ayrı bir sözdizimi sunmaz. Kuefler'in *Structure tests to emphasize behaviors* başlığındaki önerisi, böyle dillerde bölümleri boşluk ve gerekirse yorumla belirgin kılmaktır:

```rust
#[test]
fn a_comma_is_not_a_decimal_point() {
    let text = "1,5";

    let result = parse_quantity(text);

    assert_eq!(result, Err(QuantityError::NotANumber));
}
```

Bu kadar kısa bir testte üç bölüm tek satıra da sığabilirdi. Bizce bölümleri ayırmak, test uzadığında neyin kurulduğunu ve neyin denetlendiğini bulmayı kolaylaştırır.

## Ad davranışı anlatır

*Software Engineering at Google* kitabının *Unit Testing* bölümünü Erik Kuefler yazdı. Bölümün *Name tests after the behavior being tested* başlığına göre iyi bir test adı, sisteme yapılan eylemi ve beklenen sonucu özetler. Kuefler adın önemini şöyle gerekçelendirir: bir test kırıldığında raporda çoğu zaman ilk ya da tek görülen şey adıdır. Adında "ve" geçen bir test büyük olasılıkla birden fazla davranışı sınıyordur.

Rust'ta ad bir iş daha görür. Rust Book'un aynı bölümündeki *Controlling How Tests Are Run* başlığının *Running a Subset of Tests by Name* kısmına göre `cargo test`'e bir adın parçası verilince yalnızca adında o parçayı taşıyan testler çalışır. Testin bulunduğu modül de adın parçasıdır. `a_comma_is_not_a_decimal_point` testi çıktıda `tests::a_comma_is_not_a_decimal_point` olarak görünür, `cargo test decimal` ile de seçilir.

Bizce bundan iki sonuç çıkar:

- Aynı davranışın testlerinde aynı sözcükleri kullanmak, onları tek bir filtreyle çalıştırmayı sağlar. Modül adı da test adının parçası olduğu için, miktar kuralları `quantity` adlı bir modülde dursa `cargo test quantity` o modülün bütün testlerini bulur. Sözcüğün biçimine dikkat etmek gerekir: `quantities` içinde `quantity` geçmez.
- Ada `test_` öneki eklemek bir şey katmaz. Fonksiyonu test yapan `#[test]` özniteliğidir, modül adı da çıktıda zaten görünür. Google'ın Comprehensive Rust kursunun *Testing* bölümündeki *Unit Tests* sayfasının sade örneklerinde `test_empty` gibi adlar geçer. Bunlar kursun kısa örnekleri, Kuefler'in anlattığı davranış adları değil.

## Tek kavram ve tablo testleri

Clean Code serisinin *Temiz test nasıl yazılır?* notu her testin tek kavramı sınamasını anlatmıştı. Rust'ta bu kural en çok tablo testlerinde zorlanır: aynı kuralı birçok girdiyle denemek isteyen biri, girdileri bir diziye koyup döngüyle dolaşır.

Rust'ın test çalıştırıcısında parametreli testler için yerleşik bir araç yok. Döngüyle yazılan bir tabloda ilk başarısız `assert` testi bitirir, sonraki vakalar hiç denenmez. Denemede `"1,5"`, `"-1"` ve `"1,0"` girdilerini dolaşan bir testte hem birinci hem üçüncü girdi yanlışlıkla kabul ediliyordu. Çıktı yalnızca birincisini gösterdi:

```text
assertion `left == right` failed: "1,5"
  left: Ok(1.5)
 right: Err(NotANumber)
```

Çıktıdaki `"1,5"`, assert'e eklenen `"{text:?}"` iletisinden geliyor. Bu ileti olmasa hangi vakanın kırıldığı da görünmezdi.

Kuefler aynı bölümün *Don't Put Logic in Tests* başlığında daha genel bir uyarı yapar. Ona göre açık bir test, ona bakınca doğruluğu anlaşılan testtir. Döngüler ve koşullar testi okuyanı kafasında hesap yapmaya zorlar, testlerin de kendi testleri yoktur.

Bizce Rust'ta üç yol var ve seçim vakaların ne kadar benzediğine bağlı:

| Yol | Ne zaman | Bedeli |
|---|---|---|
| Her vakaya ayrı test | Vakalar farklı davranışlar ya da farklı nedenler taşıyorsa | Daha çok satır. Kuefler'in *DAMP, Not DRY* başlığı, testi açık kıldığı sürece biraz tekrarı kabul edilebilir sayar |
| Döngü ve iletide vakanın adı | Vakalar tek bir kuralın örnekleriyse ve kısa bir liste yeterliyse | İlk kırılan vaka ötekileri gizler |
| Kırılan vakaları toplayıp tek assert | Vakalar tek bir kuralın örnekleriyse ve hepsini birden görmek isteniyorsa | Döngü yerine bir süzgeç ifadesi okunur |

Üçüncü yolda test, kuralı çiğneyen bütün girdileri bir listede toplar ve listenin boş olmasını bekler:

```rust
#[test]
fn signs_and_exponents_are_not_numbers() {
    let accepted: Vec<&str> = ["-1", "+1", "1e3", "1.", ".5"]
        .into_iter()
        .filter(|text| parse_quantity(text).is_ok())
        .collect();

    assert_eq!(accepted, Vec::<&str>::new());
}
```

Denemede `"1,5"`, `"-1"` ve `"1,0"` girdilerini aynı hatalı kodla bu biçimde de sınadık. Çıktı, yanlışlıkla kabul edilen iki girdiyi tek seferde gösterdi:

```text
assertion `left == right` failed
  left: ["1,5", "1,0"]
 right: []
```

## Kırılan test ne olduğunu söylemeli

Kuefler'in *Write Clear Failure Messages* başlığına göre iyi bir hata iletisi beklenen sonucu, gerçekleşen sonucu ve ilgili parametreleri gösterir. Öyle ki test kodunu açmadan sorun anlaşılabilsin. Kuefler'in örneği bu yüzden assert'in neyi aldığına bakar: yalnızca bir `bool` alan assert ancak genel bir ileti verebilir, sınanan değerin kendisini alan assert ise ne beklendiğini ve ne geldiğini söyleyebilir. Böyle bir araç yoksa önemli bilgi iletiye elle yazılır. Rust'ta bunun büyük kısmını assert makrosunun seçimi belirler.

**`assert_eq!` iki değeri de basar.** Rust Book'un *Testing Equality with assert_eq! and assert_ne!* başlığına göre `assert!(a == b)` yalnızca ifadenin yanlış çıktığını söyler. `assert_eq!` ise iki değeri de çıktıya yazar. Bunun için değerlerin türü `PartialEq` ve `Debug` trait'lerini taşımalıdır. Kendi türlerimizde bu çoğu zaman `#[derive(PartialEq, Debug)]` eklemek kadar kolaydır. Kitap, Rust'ta iki argümanın `left` ve `right` diye adlandırıldığını ve sıralarının önemli olmadığını da hatırlatır.

**Ek ileti bağlam katar.** Aynı bölümün *Adding Custom Failure Messages* başlığına göre `assert!`, `assert_eq!` ve `assert_ne!` zorunlu argümanlardan sonra bir biçim dizgesi alabilir. Tablo testindeki `"{text:?}"` iletisi buna bir örnektir.

**`assert!(matches!(...))` gelen değeri göstermez.** Bir sonucun yalnızca biçimini denetlemek, örneğin bir `Err` olduğunu bilmek yeterliyse akla ilk gelen `matches!` makrosudur. Ama `assert!` yalnızca bir `bool` görür. Denemede kırılan test şu kadarını söyledi:

```text
assertion failed: matches!(result, Err(_))
```

Rust 1.96.0'dan beri kararlı olan `assert_matches!` bu eksiği kapatır. Standart kütüphane belgesi bu makroyu, gelen değerin `Debug` gösterimini basabildiği için `assert!(matches!(...))` yerine genellikle tercih edilir sayar. Desenden sonra bir `if` koşulu da alabilir. Belgedeki örnek makroyu `use std::assert_matches;` satırıyla içeri alır. Denemede bu satır olmadan derleyici makroyu bulamadı:

```rust
use std::assert_matches;

#[test]
fn a_name_without_the_pdf_extension_is_refused() {
    assert_matches!(
        pdf_target("rapor.txt"),
        Err(message) if message.contains(".pdf uzantılı")
    );
}
```

Ayrı bir denemede `Ok` beklenen bir yerde `Err` geldi. Ek iletisiyle birlikte kırılan `assert_matches!` hem gelen değeri hem beklenen deseni gösterdi:

```text
assertion `left matches right` failed: beklenen: kaydedildi
  left: Err("x.pdf")
 right: Ok(_)
```

Bizce kural şu: sonucun tamamı biliniyorsa `assert_eq!`, yalnızca biçimi biliniyorsa `assert_matches!` kullanılır. `assert!` ise yalnızca gerçekten bir `bool` sınanıyorsa, o da gelen değeri gösteren bir iletiyle birlikte kullanılır. `assert_matches!` da değerin `Debug` taşımasını ister. Taşımayan bir türde `assert!(matches!(...))` en kısa seçenek olarak kalır. Gelen değeri görmek isteyen, türe `Debug` türetir.

## `Result` döndüren testler

Rust Book'un *Using Result<T, E> in Tests* başlığına göre bir test `Result<T, E>` döndürebilir. Böylece gövdede `?` işleci kullanılabilir ve `Err` dönen herhangi bir adım testi başarısız yapar:

```rust
#[test]
fn a_fraction_after_a_point_is_kept() -> Result<(), QuantityError> {
    let quantity = parse_quantity("0.5")?;

    assert_eq!(quantity, 0.5);
    Ok(())
}
```

Kitap iki sınır koyar. Böyle bir teste `#[should_panic]` eklenemez. Bir işlemin `Err` döndüğünü sınamak için de o değer üzerinde `?` kullanılmaz; kitap bunun yerine `assert!(value.is_err())` önerir.

Denemede bir bedel daha gördük. `?` ile kırılan test yalnızca hatanın `Debug` gösterimini bastı:

```text
Error: Empty
```

Satır numarası yoktu. Testte birkaç `?` olsaydı hangisinin kırıldığı görünmezdi. Aynı adımı `.expect("miktar okunmalı")` ile yazınca çıktı hem iletiyi hem satırı gösterdi.

Bizce bu yüzden `?`, kırılmaması beklenen kurulum adımlarına uyar. Sınanan sonucun kendisi `assert_eq!` ya da `assert_matches!` ile denetlenir. Kitabın önerdiği `assert!(value.is_err())` de gelen değeri göstermediği için, `Debug` taşıyan bir türde `assert_matches!(value, Err(_))` daha çok bilgi verir.

## `#[should_panic]`

`#[should_panic]` ile işaretli bir testte başarı ölçütü tersine döner: panik beklenen sonuçtur, panik olmazsa test başarısız sayılır. Rust Book'un *Checking for Panics with should_panic* başlığı bu ölçütün bir zayıflığına dikkat çeker. Kodun başka bir yerindeki ilgisiz bir panik de testi geçirir. Kitabın çaresi, özniteliğe `expected` ile panik iletisinden bir parça vermektir. Test ancak gelen ileti bu parçayı içeriyorsa geçer.

```rust
#[test]
#[should_panic(expected = "yüzde 100'ü aşamaz")]
fn a_percent_above_one_hundred_is_a_bug() {
    Percent::new(101);
}
```

Denemede iletisi beklenen metni içermeyen bir panikte test başarısız sayıldı. Çıktı gelen iletiyi ve beklenen metni alt alta gösterdi.

Bizce `#[should_panic]` yalnızca panik, kodun sözleşmesinin bir parçasıysa kullanılır. Örnekteki `Percent::new`, 100'ü aşan bir değeri çağıranın hatası sayıyor ve bu yüzden panik yapıyor. Hatayı `Result` ile bildiren kodda test `Err` değerini denetler, panik beklemez.

## Test yardımcıları

Clean Code serisinin *Temiz test nasıl yazılır?* notu, testlere özgü bir dil kuran yardımcıları ve Kuefler'in DAMP ilkesini anlatmıştı: yardımcı ilgisiz ayrıntıyı saklar, sonucu belirleyen değeri saklamaz. Kuefler'in *Shared Helpers and Validation* başlığı, denetim yapan yardımcıları iki türe ayırır. Odaklı olanı girdileri hakkında tek bir kavramsal gerçeği denetler. Kavram basit ama denetimi döngü ya da koşul gerektiriyorsa, bu mantığı test gövdesinden alıp yardımcıya taşımak okunabilirliği korur. Genel olanı ise her testin sonunda çağrılıp sabit bir denetim listesini yürütür. Kuefler bunun kötü bir alışkanlığa dönüşebileceğini söyler: testin neyi amaçladığı kaybolur, bir hata da birçok testi birden kırar.

Rust'ta denetim yapan yardımcının bir tuzağı var. Yardımcının içindeki `assert_eq!` kırılınca çıktı yardımcının satırını gösterir, onu çağıran testin satırını göstermez. Aynı testte yardımcı birkaç kez çağrılıyorsa hangi çağrının kırıldığı ancak iletiden çıkarılabilir.

Rust Reference'ın *Code generation attributes* sayfasındaki *The track_caller attribute* başlığı işe yarayan bir mekanizma anlatır. Bu öznitelikle işaretli bir fonksiyonun içindeki kod, fonksiyonu çağıran yerin konumuna dair bir ipucu alır. Reference bunun bir ipucu olduğunu ve derleyicinin korumak zorunda olmadığını da yazar. Fonksiyon, fonksiyon işaretçisine dönüştürülünce de gerçek çağıran bilgisi kaybolur. Bizce test yardımcısının sorununu bu mekanizma çözer, çünkü yardımcılar testten doğrudan çağrılır.

```rust
#[track_caller]
fn assert_refused(text: &str) {
    assert_eq!(parse_quantity(text), Err(QuantityError::NotANumber), "{text:?}");
}

#[test]
fn spaces_around_a_number_are_refused() {
    assert_refused(" 1");
    assert_refused("1 ");
}
```

Denemede biri öznitelikli, öteki özniteliksiz iki eş yardımcıyı iki ayrı testte kullandık. Özniteliksiz yardımcıda çıktı yardımcının içindeki `assert_eq!` satırını gösterdi. `#[track_caller]` ekli yardımcıda ise kırılan çağrının test gövdesindeki satırını gösterdi.

## Clean Code ile bağ

- **Kur, işlet, denetle.** Bizce Rust Book'un bir test gövdesine verdiği üç iş, Clean Code'un *Birim Testleri* bölümündeki desenle örtüşür.
- **Her testte tek kavram.** Rust'ta bu kural tablo testlerinde sınanır. Vakalar tek kuralın örnekleriyse bir tabloda kalabilir, farklı davranışlarsa ayrı testlere bölünür.
- **Testlere özgü dil.** Rust'ta bu dilin yardımcıları `#[track_caller]` ile hatayı testin satırına bağlayabilir.
- **Çifte standart.** Clean Code serisinin [Nelerden ödün verilir?](clean_code/04-nelerden-odun-verilir.html) notu, test kodunda kaynak kullanımından ödün verilebileceğini, okunabilirlikten verilemeyeceğini anlatmıştı. Bizce Rust'ta bunun karşılığı şu: testte `.clone()` ve kurulum adımlarında `.unwrap()` ya da `.expect()` serbesttir, çünkü test zaten panikle kırılır. Gelen değeri göstermeyen bir assert ise okunabilirlikten verilmiş bir ödündür.

## Bu projede yaşadığımız

- **Tablo testleri döngüyle yazıldı.** 7 Kasım 2026'da destek ekipmanı ve yedek parça satırları belge modeline eklenirken yazılan miktar testleri, geçerli ve geçersiz miktarları dizilerde topladı ve döngüyle dolaştı. Döngüdeki her assert'e `"{text:?}"` iletisi eklendi, böylece kırılan vaka adıyla görünecekti. Ama döngü yüzünden ilk kırılan vaka ötekileri gizleyecekti. Aynı testlerden birinin sonunda, parça numarası sütununun sayı olmayan bir metni kabul ettiğini denetleyen ayrı bir assert de vardı. Üstündeki yorum adları da anıyordu. Bizce bu, miktar kuralından ayrı bir kavramdı.
- **Bütün taşımaları dolaşan bir test.** Aynı gün blok taşıma komutları eklenirken yazılan bir test, örnek veri modülünde ve testlerin içinde yazılmış iki küçük modülde her blok için her taşıma komutunu denedi. Kabul edilen her taşımadan sonra modülün hâlâ geçerli olduğunu bir doğrulama yardımcısı denetledi. Yardımcı her iletisine o anki komutu ekliyordu. Bu test Kuefler'in "testte mantık olmasın" kuralına aykırıydı. Bizce yine de yerindeydi: testin amacı tek tek vakalar değil, bütün taşımaların ortak bir özelliği koruduğunu göstermekti. Döngü bu amacın kendisiydi, bir kısayol değildi. Kırılan vaka da iletideki komuttan anlaşılacaktı.
- **Önizleme testleri `matches!` ile denetlendi.** 6 Kasım 2026'daki ilk sürümden beri önizleme panelinin testleri panelin durumunu `assert!(matches!(...))` ile denetledi. 7 Kasım 2026'nın sonunda durum türü `Debug` türetmiyordu. Bu yüzden böyle bir test kırılsaydı gelen durum çıktıda görünmeyecekti. `assert_matches!` da `Debug` istediği için tek başına çözüm olmayacaktı. Aynı testlerde kaydedilen PDF'in sonucu ise `Debug` taşıyan bir türdü. Bizce onu denetleyen assert'ler `assert_matches!` ile gelen sonucu gösterebilirdi.
- **Bazı araçlar hiç kullanılmadı.** 7 Kasım 2026'nın sonunda projenin testlerinde `#[should_panic]`, `Result` döndüren test ve `#[track_caller]` yoktu. Hiçbir test adı `test_` önekiyle başlamıyordu. Belge modelindeki komutlar gibi hata bildiren kodlar `Result` döndürüyor, testleri de `Err` değerlerini denetliyordu.

## Belgelerin söylemedikleri

- Rust Book tablo testlerinden ya da parametreli testlerden söz etmez. Döngüyle yazılan bir testin ilk kırılan vakada durduğu, bu notun kendi denemesinden geliyor.
- Rust Book `Result` döndüren testin kırıldığında satır numarası vermediğini söylemez.
- Rust Book'un test bölümleri `assert_matches!` makrosundan ve `#[track_caller]` özniteliğinden söz etmez. Bu ikisi standart kütüphane belgesinden ve Rust Reference'tan geliyor.
