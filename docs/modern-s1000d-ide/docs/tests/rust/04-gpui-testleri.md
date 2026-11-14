---
title: "GPUI testleri"
ikon: 🪟
kapak: purple
tarih: 2026-11-08
etiketler: [rust, testler, gpui]
ozet: GPUI testleri pencereyi ekranda göstermeden, saati, görev sırasını ve kullanıcıyı taklit ederek çalışır. Zamanlayıcılar ancak test saati ilerletilince dolar; run_until_parked saati ilerletmez. Görevler tohumlu bir sıralayıcıyla belirlenimci çalışır. Platforma göre değişen kısayollar secondary- ile yazılır. GPUI'nin test makrosunun adı test olduğu için use super::* Rust'ın #[test] özniteliğini gölgeleyebilir.
---

:::callout 🪟 purple
**GPUI testleri pencereyi ekranda göstermeden; saati, görevlerin sırasını ve kullanıcıyı taklit ederek çalışır.** `#[gpui_kit::test]` teste bir `TestAppContext` verir. Arkasındaki test platformu pencereleri, dosya diyaloglarını, tuş vuruşlarını ve tıklamaları taklit eder. Zamanlayıcılar ancak test saati elle ilerletilince dolar. Bekleyen görevler tohumlu bir rastgele sayı üreteciyle belirlenimci bir sırada çalışır. Platforma göre değişen kısayollar `secondary-` önekiyle yazılır.
:::

7 Kasım 2026'nın sonunda projenin ana dalında 66 test `#[gpui_kit::test]` ile yazılmıştı. Bu, projedeki testlerin büyük bir kısmıydı. Bu not, Clean Code serisinin [F.I.R.S.T.](clean_code/03-first.html) notundaki zamanı testin eline verme ve farklı platformlarda çalıştırma fikirlerinin GPUI'de nasıl karşılandığını anlatıyor. Serinin [Her ortamda çalışma](03-her-ortamda-calisma.html) notu gerçek saatin ve paralel çalışmanın sorunlarını anlatmıştı. Bu not GPUI'nin bu sorunlara kendi cevabını anlatıyor.

Rust Book GPUI'yi anlatmaz. GPUI'nin README dosyası da bu API'leri öğrenmenin şimdilik en iyi yolunun Zed'in kaynak kodunu okumak olduğunu söyler. Bu yüzden notun kaynağı GPUI'nin kendi kaynak kodu ve kod içi belgeleridir. Proje GPUI'yi `gpui-pre` 0.3.11 paketiyle kullanıyordu. Bu paket, Zed'in GPUI'sinin Apache-2.0 lisanslı bir anlık görüntüsüdür. README'ye göre GPUI henüz 1.0 öncesindedir ve sürümler arasında sık sık kıran değişiklikler olur. Aşağıdaki ayrıntılar bu sürüme aittir.

Notun deneme örnekleri 8 Kasım 2026'da Rust 1.98.1 ile, projenin kullandığı `gpui-kit` 0.9.0 ve `gpui-pre` 0.3.11 sürümleriyle ayrı bir deneme crate'inde çalıştırıldı. `previewing` yardımcısı ve PDF kaydetme parçası ise projenin kendi test kodundan alındı.

## Test makrosu ve bağlam

GPUI'nin `#[gpui::test]` makrosunun belgesine göre makro, üzerinde durduğu fonksiyonu sıradan bir `#[test]` fonksiyonuna çevirir. Böylece test hem `cargo test` hem nextest ile çalışır. Makro senkron ve `async` testleri destekler, teste istediği sayıda `TestAppContext` verebilir. `gpui-kit` bu makroyu kendi adıyla yeniden dışa açar. Proje testleri bu yüzden `#[gpui_kit::test]` diye yazdı.

İki bağlam türü var. GPUI'nin belgesine göre `TestAppContext`, uygulamanın bağlamına testlerde işe yarayan yöntemler ekler. `VisualTestContext` ise bir pencerenin test karşılığıdır, pencereye özgü test kodunu çalıştırmayı sağlar. Projenin önizleme testleri, bir paneli görünmeyen bir pencerede kuran küçük bir yardımcıyla başladı:

```rust
/// A panel in a headless window that has just been told to preview `dm.xml` in `folder`.
fn previewing<'a>(
    folder: &TestFolder,
    cx: &'a mut TestAppContext,
) -> (Entity<PreviewPanel>, &'a mut VisualTestContext) {
    folder.write("template.typ", TEMPLATE);
    folder.write("dm.xml", "ilk");
    cx.update(gpui_kit::init);
    let (panel, cx) = cx.add_window_view(|_, _| PreviewPanel::default());
    let source = PreviewSource::new(folder.path().to_path_buf(), "/dm.xml");
    panel.update(cx, |panel, cx| panel.preview(source, cx));
    (panel, cx)
}
```

**Test araçları bir özelliğin arkasındadır.** GPUI'nin test yöntemleri `cfg(any(test, feature = "test-support"))` koşuluyla derlenir. Yani yalnızca GPUI'nin kendi testlerinde ya da `test-support` özelliği açıldığında vardır. Serinin [Testlerin yeri](01-testlerin-yeri.html) notu bu deseni anlatmıştı. Proje özelliği yalnızca testlerde açmak için GPUI'yi bir geliştirme bağımlılığı olarak ekledi ve sürümünü `gpui-kit`'in kullandığı GPUI sürümüne sabitledi.

## Makronun adı `test`

GPUI test makrosuna `test` adını verir, `gpui-kit` de GPUI'nin bütün adlarını `pub use ::gpui::*;` ile yeniden dışa açar. `gpui-kit`'in kaynak kodundaki bir yorum bunun sonucunu açıkça yazar: `test-support` açıkken bu toplu dışa açma GPUI'nin test makrosunu da içerir, bu yüzden test modülleri `gpui-kit` türlerini tek tek içeri almalıdır, yoksa Rust'ın `#[test]` özniteliği gölgelenir. Sınanan modül `use gpui_kit::*;` ile her şeyi içeri alıyorsa, test modülündeki `use super::*;` bu makroyu da içeri taşır. O modülde yazılan `#[test]` artık Rust'ın kendi özniteliğine değil, GPUI'nin makrosuna çözülür.

Denemede bunu bir modülde kurduk. Sıradan bir `#[test]` fonksiyonu derlenmedi, derleyici şu hatayı verdi:

```text
error: recursion limit reached while expanding `#[test]`
```

Bizce bunun nedeni şu: GPUI'nin makrosu ürettiği koda `#[test]` yazar, gölgelenmiş adla bu da yeniden GPUI'nin makrosuna gider. Projenin test modüllerinde 6 Kasım 2026'dan beri bu yüzden `use super::*` yerine tek tek içeri alma yazılıydı. Yanlarında da nedenini anlatan bir yorum vardı.

## Saat yalnızca test ilerletince ilerler

GPUI testlerinde yürütücünün (executor) zamanlayıcıları gerçek saate değil, test saatine bağlıdır. Bu yalnızca yürütücünün kendi zaman araçları için geçerlidir. Yürütücünün `now` yönteminin belgesi, sahte zamanlayıcıların testte işe yaraması için `std::time::Instant::now` yerine bu yöntemin kullanılmasını ister. Kod gerçek saati doğrudan okursa test saati onu etkilemez.

Yürütücünün belgesi iki yöntemi şöyle anlatır:

- `advance_clock` zamanı ileri alır. Hiçbir görevi çalıştırmaz, yalnızca süresi dolan zamanlayıcıları hazır hale getirir.
- `run_until_parked`, sıralayıcı duracak noktaya gelene kadar bekleyen görevleri çalıştırır. Çalışacak görev kalmayınca saati bir sonraki zamanlayıcıya ilerlettiğini de söyler.

İki belge de 0.3.11'deki kodla uyuşmuyor:

- `run_until_parked`, sıralayıcının saati ilerletmeyen `run` yöntemini çağırıyor. Saati ilerleten ayrı bir yöntem olan `run_with_clock_advancement`'ı çağırmıyor.
- `advance_clock` ise sıralayıcıda, saati her zamanlayıcıya ilerletirken araya `run` çağrıları koyan bir döngü yürütüyor. Yani uyanan görevleri de çalıştırıyor.

Denemede kod haklı çıktı. Saniyede bir sayaç artıran bir varlıkta `run_until_parked` sayacı artırmadı. Ayrı bir denemede saat bir saniye ilerletilince, ardından `run_until_parked` çağrılmadan da sayaç bir arttı. Aşağıdaki test iki adımı birlikte kullanıyor:

```rust
#[gpui_kit::test]
fn the_timer_fires_only_when_the_test_clock_moves(cx: &mut TestAppContext) {
    let ticker = cx.new(Ticker::new);

    cx.run_until_parked();
    assert_eq!(ticker.read_with(cx, |ticker, _| ticker.ticks), 0);

    cx.executor().advance_clock(Duration::from_secs(1));
    cx.run_until_parked();
    assert_eq!(ticker.read_with(cx, |ticker, _| ticker.ticks), 1);
}
```

`Ticker`, bir döngüde `cx.background_executor().timer(...)` bekleyip sayacı artıran küçük bir varlıktır.

Clean Code serisinin F.I.R.S.T. notu, Martin'in zamanlayıcı örneğini anlatmıştı: kod sistem saatinden yalıtılır, zamanı test adım adım ilerletir. GPUI bunu hazır sunar. Kod zamanı yürütücüden okuduğu sürece, test koda kendi sahte saatini yerleştirmek zorunda kalmaz, GPUI'nin test saatini ilerletir.

## Görev sırası tohumla belirlenir

Bir GPUI testinde bekleyen görevlerin hangi sırayla çalışacağını, tohumlanmış bir rastgele sayı üreteci belirler. Makronun belgesine göre bu üreteç `SEED` ortam değişkeniyle tohumlanır ve yürütücüler görevleri onunla belirlenimci biçimde çalıştırır. Makro birkaç seçenek alır:

| Yazım | Ne yapar |
|---|---|
| `#[gpui::test]` | Bir kez çalışır. Tohum 0'dır ya da `SEED` ortam değişkeninin değeridir |
| `#[gpui::test(seed = 10)]` | Bir kez, 10 tohumuyla çalışır |
| `#[gpui::test(iterations = 5)]` | Beş kez çalışır. Tohumlar 0'dan 4'e kadardır |
| `#[gpui::test(retries = 3)]` | Başarısız olursa geçmesi için en çok dört kez çalışır |

`ITERATIONS` ortam değişkeni de `iterations` değerini dışarıdan değiştirir. Belgeye göre `SEED`, ilk çalıştırmanın tohumunu belirler.

Denemede dört arka plan görevi bir listeye adını yazdı. Test `iterations = 6` ile 0'dan 5'e altı tohumda çalıştı ve her tohumda farklı bir sıra çıktı. Örneğin tohum 0'da `d, a, b, c`, tohum 3'te `b, d, c, a`. Aynı test `SEED=3` ile çalıştırılınca tohumlar 3'ten başladı ve ilk sıra yine `b, d, c, a` oldu. `ITERATIONS=1 SEED=3` ile yalnızca bu tohum çalıştı ve aynı sırayı verdi.

```rust
#[gpui_kit::test(iterations = 6)]
fn background_tasks_run_in_a_seeded_order(cx: &mut TestAppContext) {
    let log = Arc::new(Mutex::new(Vec::new()));
    for name in ["a", "b", "c", "d"] {
        let log = log.clone();
        cx.executor()
            .spawn(async move { log.lock().unwrap().push(name) })
            .detach();
    }
    cx.run_until_parked();
    println!("order: {:?}", log.lock().unwrap());
}
```

Clean Code serisinin F.I.R.S.T. notu, Schuchert'in eşzamanlılık ekindeki *Çok İş Parçacıklı Kodu Test Etme* başlığında yaptığı öneriyi anlatmıştı: nadir görülen sıralama hatalarını ortaya çıkarmak için testler, ayarları rastgele değiştirilerek sürekli çalıştırılır. Bizce GPUI'nin tohumları bunun tekrarlanabilir bir biçimidir. Bir sıralama hatası bir tohumda ortaya çıkarsa, aynı tohumla aynı sıra yeniden kurulabilir.

`retries` seçeneği için iki kaynağa bakılabilir. Schuchert, *Eşzamanlılık* bölümündeki *Sahte Başarısızlıkları Eşzamanlılık Sorunu Adayı Olarak Değerlendirin* alt başlığında tek seferlik hata olmadığını varsaymayı önerir. Bizce `retries` bu öneriye ters düşer. Google'ın *Software Engineering at Google* kitabının *Testing Overview* bölümünde Adam Bender daha ölçülüdür. *Flaky Tests Are Expensive* örnek olayına göre kırılan testi otomatik olarak yeniden çalıştırmak, kararsızlık oranı düşükken işlemci zamanını mühendis zamanıyla takas eden makul bir yoldur. Bender bunun kök nedeni yalnızca ertelediğini de ekler.

## Kullanıcıyı taklit etmek

`TestAppContext` ve `VisualTestContext`, kullanıcının yapacaklarını taklit eden yöntemler sunar. Kaynak kodda bunların her biri kısa bir belgeyle anlatılır:

- `simulate_input`, odaktaki alana metin yazar.
- `simulate_keystrokes`, boşlukla ayrılmış bir tuş dizisine basar. İkisi de ardından arka plan yürütücüsünü durana kadar çalıştırır.
- `simulate_click` verilen noktada birincil fare tıklamasını taklit eder. `simulate_mouse_move` fareyi bir noktaya götürür, `simulate_mouse_down` da fare düğmesine basar.
- `simulate_path_prompt_response`, "Aç" diyaloğuna verilecek yanıtı belirler. `did_prompt_for_paths`, yanıt bekleyen böyle bir diyalog olup olmadığını söyler. Yanıt verilince yeniden yanlış döner.
- `simulate_new_path_selection`, kaydedilecek dosyanın yolunu soran diyaloğa yanıt verir; `did_prompt_for_new_path` de yanıt bekleyen böyle bir diyalog olup olmadığını söyler. Kod içi belgesi bu yöntemi de "Aç" diyaloğu diye anlatıyor, ama test platformunda yöntem yeni yol isteyen diyalog kuyruğundan okuyor.
- `simulate_prompt_answer`, bir uyarı penceresinde bir düğmeye basar.

Bu yöntemler hatalı kullanımda hemen paniğe geçer. Test platformunun kodunda, açık bir diyalog yokken yanıt vermek ya da tek dosya seçilebilen bir diyaloğa birden çok dosya vermek paniğe yol açıyor.

7 Kasım 2026'da eklenen PDF kaydetme testi bunları birlikte kullandı. Test kaydet düğmesine tıklıyor, diyaloğun açıldığını denetliyor ve diyaloğa yolu kendisi veriyordu:

```rust
click_save(cx);
assert!(cx.did_prompt_for_new_path());
cx.simulate_new_path_selection(|_| Some(path.clone()));
cx.run_until_parked();
```

**Kısayollar platforma göre değişir.** Uygulamaların kısayolları işletim sistemine göre farklı tanımlanır; projenin kullandığı GPUI Kit de geri almayı macOS'ta `cmd-z`'ye, Linux ve Windows'ta `ctrl-z`'ye bağlar. GPUI'nin tuş ayrıştırıcısında işletim sistemine bağlı tek önek `secondary-`'dir: macOS'ta `cmd`'ye, öteki sistemlerde `ctrl`'ye dönüşür. Bir testte `ctrl-z` yazmak, testi Linux ve Windows'taki geri alma kısayoluna bağlar.

## Test platformu bir sahte uygulamadır

*Software Engineering at Google* kitabının *Test Doubles* bölümünü Andrew Trenk ile Dillon Bly yazdı. Yazarlar, testte gerçek uygulama kullanılamadığında başvurulacak yolu bölümün *Faking* başlığında tartışır. Onlara göre sahte uygulama (fake) çoğu zaman en iyi seçenektir, çünkü gerçeğe yakın davranır. İdeali, sınanan kodun gerçeğiyle mi sahtesiyle mi konuştuğunu fark edememesidir. *When Should Fakes Be Written?* alt başlığı sahteyi kimin yazacağını da söyler: gerçek uygulamanın sahibi olan ekip. *The Fidelity of Fakes* alt başlığında sadakat belki de en önemli kavram olarak öne çıkar. Sahte uygulama öncelikle gerçeğin API sözleşmesine sadık kalmalıdır. Desteklemediği bir yola girildiğinde de hemen hata vermelidir.

Bizce GPUI'nin test platformu bu tarife uyar. Pencereleri, saati, diyalogları ve klavyeyi taklit eden bir sahte uygulamadır. Onu GPUI'nin kendi ekibi yazar ve sürdürür. Diyalog yöntemlerinin hatalı kullanımda paniğe geçmesi de "hemen hata ver" önerisine uyar. Sadakati projede de görüldü: test platformu, testin çalıştığı işletim sistemini gerçeği gibi bildirdiği için GPUI Kit'in macOS kısayolları testte de geçerli oldu. Sadakatin sınırları da var. Görünmeyen pencere gerçek bir ekrana çizilmez, test saati de gerçek gecikmeleri ölçmez. Bu yüzden çizim hızı ya da gerçek zamanlı davranış bu testlerle sınanamaz.

## Bu projede yaşadığımız

- **İlk sürümden beri GPUI testleri.** 6 Kasım 2026'daki ilk sürümde önizleme panelinin testleri, paneli görünmeyen bir pencerede kuran `previewing` ve test saatini bir saniye ilerletip görevleri çalıştıran `notice_change` yardımcılarıyla yazıldı. Panelin klasör izleme döngüsü değişiklikleri 400 milisaniyede bir yokluyordu, bu yüzden testler saati elle ilerletmek zorundaydı. Bir test, sayfa yenilenince eski sayfa görüntüsünün GPUI'nin görüntü atlasından çıkarıldığını da denetledi.
- **Bağımlılığın sürümü elle eşlendi.** Aynı gün GPUI, `test-support` özelliğiyle bir geliştirme bağımlılığı olarak eklendi. `Cargo.toml`'daki yorum iki şeyi anlatıyordu. Sürüm `gpui-kit`'in kullandığı GPUI ile aynı kalmalıydı, yoksa testler derlenmezdi. `gpui-kit`'in kendi `test-support` özelliği ise seçilmemişti. Yoruma göre bu özellik kilit dosyasına `gpui-fast` paketini getiriyor, o paketin kesin sürüm sabitlemeleri de uygulamanın `taffy` ve `unicode-properties` bağımlılıklarını daha eski sürümlere çekecekti.
- **Kullanıcı taklitleri adım adım eklendi.** Metin yazmayı taklit eden `simulate_input` 6 Kasım 2026'da, tuş dizisi gönderen `simulate_keystrokes` 7 Kasım 2026'da kullanılmaya başladı. Aynı gün PDF kaydetme ve klasör seçme diyalogları da testlerde taklit edildi.
- **macOS'ta kırılan geri alma testi.** 7 Kasım 2026'da silinen bir atıf çipini geri getiren bir test macOS'ta kırıldı, Linux'ta geçti. Test geri almak için `ctrl-z` gönderiyordu. `secondary-z` ile denenince macOS'ta da geçti. 7 Kasım 2026'nın sonunda ana daldaki test hâlâ `ctrl-z` kullanıyordu.
- **Hep aynı tohum.** 7 Kasım 2026'nın sonunda projedeki 66 GPUI testinin hiçbiri `iterations`, `seed` ya da `retries` kullanmıyordu. `SEED` verilmediği sürece testlerin hepsi yalnızca 0 tohumuyla, yani tek bir görev sırasıyla çalışıyordu. Bizce görev sırasına bağlı bir hata, ancak 0 tohumunun sırasında ortaya çıkıyorsa yakalanabilirdi.

## Belgelerin söylemedikleri

- GPUI'nin belgeleri az. README, bu API'leri öğrenmek için Zed'in kaynak kodunu okumayı öneriyor. Bu notun bilgileri `gpui-pre` 0.3.11'in kaynak kodundan geliyor ve sonraki sürümlerde değişebilir.
- `run_until_parked` ve `advance_clock` yöntemlerinin belgeleri, kodun yaptığından farklı davranışlar anlatıyor. Denemede ikisinde de kodun davranışı geçerli oldu.
- Makronun belgesi, bir testin kaç tohumla çalıştırılmasının yeterli olacağını söylemiyor.
