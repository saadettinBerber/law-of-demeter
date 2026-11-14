---
title: "Kapsam ve uç durumlar"
ikon: 🎯
kapak: red
tarih: 2026-11-07
etiketler: [clean-code, testler]
ozet: Testler bozulabilecek her şeyi sınayana kadar yetersizdir; sınırlar sezgiye bırakılmaz, bir hata bulununca çevresi de taranır, bu testler sınadıkları davranışın yanına girer ve kapsam yüzdesi hedef değil sinyaldir.
---

:::callout 🎯 red
**Testler, bozulabilecek her şeyi sınayana kadar yetersizdir.** Sınırlar ve uç durumlar sezgiye bırakılmaz, tek tek test edilir. Bir hata bulunduğunda önce onu gösteren test yazılır, sonra hatanın çevresi de taranır, çünkü hatalar kümelenme eğilimindedir. Bu testler ayrı bir klasöre değil, sınadıkları davranışın diğer testlerinin yanına girer. Kapsam aracı nereye bakılmadığını gösterir. Yüzdesi bir hedef değil, bir sinyaldir.
:::

Bu not serinin beşinci parçası. Önceki notlar tek bir testin nasıl yazılacağını anlatıyordu. Bu not test paketinin bütününe bakıyor: ne kadar test yeterlidir, uç durumlar nasıl bulunur ve bir hata çıktığında testler nereye eklenir.

## Ne kadar test yeterli?

Kitabın *Kokular ve Sezgisel Kurallar* bölümündeki *Testler* grubu, *Yetersiz Testler* başlığıyla açılır. Martin, ne kadar test yazılacağı sorusunu "bu kadarı yeter gibi" duygusuna bırakmayı yetersiz bulur. Ona göre bir paket ancak bozulabilecek her şeyi sınadığında yeterlidir. Denenmemiş bir koşul ya da sonucu doğrulanmamış bir hesap kaldıkça eksiktir. Martin'in yalnızca çalıştırılmamış koşullardan değil, doğrulanmamış hesaplardan da söz ettiğine dikkat: bir satırın çalışması yetmez, ürettiği sonuç da denetlenmelidir.

## Kapsam aracı nereye bakılmadığını gösterir

Aynı gruptaki *Bir Kapsam Aracı Kullanın!* başlığına göre kapsam aracı, test stratejisindeki boşlukları raporlar ve az sınanmış modülleri ya da fonksiyonları bulmayı kolaylaştırır. Çoğu geliştirme ortamı satırları çalışıp çalışmadıklarına göre renklendirir. Hiç çalışmamış bir koşul gövdesi ya da hata yakalama bloğu böylece hemen görünür.

Kitapta bunun üç örneği var:

- **Yarım kalmış kapsam.** *SerialDate'i Yeniden Düzenleme* bölümündeki *Önce, Çalıştırın* başlığında Martin, var olan testlerin hepsi geçtiği halde sınıfın çalıştırılabilir ifadelerinin ancak yarısına uğradığını kapsam aracıyla görür. Sınıfı anlayıp yeniden düzenlemek için kendi testlerini yazar. Bir kısmı henüz kapalıyken bile kapsam yüzde 92'ye çıkar.
- **Düşen ama sorun olmayan kapsam.** Aynı bölümün *Sonra Doğrusunu Yapın* başlığının sonunda kapsam yüzde 84,9'a düşer. Martin bunun daha az şey sınandığı için değil, sınıf çok küçüldüğü için kapsanmayan birkaç satırın ağırlığı arttığı için olduğunu açıklar. Kalan satırları da test etmeye değmeyecek kadar önemsiz bulur. Martin de yüzdeyi bir hedef olarak değil, nedenine bakılacak bir sinyal olarak okur.
- **Tam kapsam.** *JUnit'in İç Yapısı* bölümünde Martin, incelediği modülün testlerinin kodun tamamını çalıştırdığını görür. Bunu hem koda güvenmek için hem de yazarlarına saygı duymak için bir neden sayar.

*Test Kapsamı Desenleri Aydınlatıcı Olabilir* başlığı aracın bir kullanımını daha gösterir: geçen testlerin hiç uğramadığı kod, kırılan testlerin neden kırıldığına dair ipucu verir. SerialDate'te hiçbir testin çalıştırmadığı bir satır, onu koruyan koşulun her zaman yanlış olduğunu ve algoritmanın baştan hatalı olduğunu ortaya çıkarmıştı.

## Sınırlar sezgiye bırakılmaz

*Sınır Koşullarını Test Edin* başlığının uyarısı tek cümlelik: hatalar çoğu zaman algoritmanın ortasında değil, kenarlarında saklanır.

Aynı bölümün *Genel Kurallar* grubundaki *Sınırlarda Yanlış Davranış* başlığı bu uyarının nedenini anlatır. Martin'e göre sınırlarda yanılmamızın nedeni, doğru davranışın ne kadar karmaşık olabileceğini küçümseyip köşe durumlarını kanıtlamak yerine sezgiye dayanmamızdır. Bunun bir kısayolu yoktur: sınır koşulları tek tek aranır ve her birine test yazılır.

SerialDate'te bir örnek vardır. Bir tarihten sonra gelen belirli bir haftanın gününü bulan fonksiyon, cumartesi olan 25 Aralık 2004'ten sonraki cumartesi sorulduğunda yine 25 Aralık'ı döndürüyordu. Bir karşılaştırmada eşitlik durumu atlanmıştı. Martin bunu sınır koşulu hatası diye adlandırır.

*Temiz test nasıl yazılır?* notundaki ay ekleme örneği de aynı yere varır. Senaryolar ayrı ayrı yazılınca ayın son gününe dair genel kural ve eksik kalan 28 Şubat testi ortaya çıkmıştı.

## Hatalar kümelenme eğilimindedir

*Hataların Yakınını Kapsamlı Biçimde Test Edin* başlığına göre hatalar bir araya toplanma eğilimindedir. Bu yüzden bir fonksiyonda hata bulunduğunda yalnızca o hatayı değil, fonksiyonun bütününü sınamak gerekir. Yanında başka hatalar çıkması olasıdır.

SerialDate bunu doğrular. Martin'in sınır hatası bulduğu fonksiyon, değişiklik geçmişine göre daha önce de onarılmıştı. Verilen haftanın gününe en yakın tarihi bulan bir başka fonksiyonun testi de başta kısaydı. İlk vakaların bir kısmı geçmediği için Martin bu testi yeni vakalarla genişletti.

*Başarısızlık Desenleri Aydınlatıcıdır* başlığı bu genişletmenin neden işe yaradığını açıklar. Hangi vakaların kırıldığına bakmak sorunun teşhisini kolaylaştırır. Martin'in örnekleri, yalnızca beş karakterden uzun girdilerin ya da ikinci argümanı negatif olan çağrıların kırıldığı durumlardır. Desenin görünmesi için vakaların olabildiğince eksiksiz ve mantıklı bir sırada olması gerekir. SerialDate'te kapalı bırakılan vakaların deseni, algoritmanın en yakın gün ileride olduğunda yanıldığını göstermişti.

## Bir hata bulunduğunda

Kitap hata düzeltmek için ayrı bir yöntem tarif etmez. Bizce önceki notlardaki kurallar birlikte şu sırayı verir:

1. **Hatayı gösteren testi yaz ve kırıldığını gör.** TDD'nin ilk yasasına göre başarısız bir test olmadan üretim kodu yazılmaz. Düzeltme de üretim kodudur.
2. **Testi geçirecek kadar düzelt.** Üçüncü yasa, kodun yalnızca kırmızı testi yeşile çevirecek kadar yazılmasını ister.
3. **Hatanın çevresini sına.** Hatalar kümelenme eğiliminde olduğu için aynı fonksiyonun sınırları ve benzer girdileri de test edilir.
4. **Testler yeşilken yeniden düzenle.** *Ortaya Çıkış* bölümüne göre yeniden düzenleme, testler bozulmadığını gösterdiği için güvenle yapılabilir.

## Uç durum testleri nerede durmalı?

Kitapta bu soruya doğrudan bir yanıt yok. Bizce dolaylı ipuçları aynı yöne bakıyor: testler nereden geldiklerine göre ayrılmaz.

- **Ayrı duran testler de tek çalıştırmaya bağlanmalıdır.** *Ardışık İyileştirme* bölümünde Martin'in birim testlerinden ayrı, FitNesse ile yazılmış bir kabul testi kümesi vardı. Bir davranışı yalnızca kabul testleri sınıyordu ve Martin o ana kadar yalnızca birim testlerini çalıştırıyordu. Bir dipnotta, böyle sürprizlerin tekrarlanmaması için kabul testlerinin hepsini çağıran bir birim testi eklediğini anlatır. Kabul testleri türleri gereği ayrıydı. Martin onları birleştirmedi, aynı çalıştırmaya bağladı.
- **Bütün testler tek komutla çalışmalıdır.** *Kokular ve Sezgisel Kurallar* bölümünün *Ortam* grubundaki *Testler Birden Fazla Adım Gerektirir* başlığına göre bütün birim testlerini tek bir komutla çalıştırabilmek temel bir gerekliliktir.
- **Hata testi, modülün öteki testlerinin arasında durur.** *JUnit'in İç Yapısı* bölümündeki test paketinde, adı bir hata numarası taşıyan bir test öteki testlerle aynı test sınıfındadır. Martin'in övdüğü paket de budur.
- **Yeni vakalar mevcut teste eklenir.** SerialDate'te Martin, hata bulduğu fonksiyonun testini yeni vakalarla genişletti.

Bizce bu ipuçlarından çıkan sonuç şu: ayrı bir klasör testin kökenine göre değil, neye dokunduğuna ve ne zaman çalıştırıldığına göre açılır. Uç durum ve hata testleri, sınadıkları davranışın testlerinin yanına girer ve adları hatanın numarasını değil davranışı anlatır. Ayrı bir yeri hak eden, farklı bir ortamda ya da farklı bir zamanda çalıştırılması gereken testlerdir. Böyle testler ayrı dursa bile tek bir komutla çalıştırılabilmelidir. Üçüncü taraf kodu öğrenmek için yazılan testler bunun örneğidir ve serinin *Sınırlar ve learning test* notunun konusudur.

## Google bu konuda

- **Beyoncé kuralı** (*Testing Overview* bölümü): Bozulmasını istemediğin her şeyi test et. Bir davranışın korunacağından emin olmanın tek yolu onu otomatik bir testle sabitlemektir. İlgisiz bir altyapı değişikliği testlerden geçip ürünü bozarsa, düzeltmek ve eksik testi eklemek ürünün sahibine düşer.
- **Hata düzeltmesi eksik vakayı ekler** (*Unit Testing* bölümü, *Strive for Unchanging Tests* başlığı): Google'a göre bir hata düzeltmesi, genellikle mevcut testleri değiştirmeden eksik kalan test vakasını ekler.
- **Test, düzeltmeyle aynı değişiklikte gelir** (Google'ın kod incelemesi rehberi, *Tests* başlığı): Testler genel olarak, acil durumlar dışında, üretim koduyla aynı değişiklikte eklenir. İnceleyen kişi testlerin, kod bozulduğunda gerçekten kırılıp kırılmayacağına da bakar.
- **Testler boyuta göre ayrılır** (*Testing Overview* bölümü, *Test Size* başlığı): Google testleri kullandıkları kaynaklara ve erişebildikleri şeylere göre küçük, orta ve büyük diye ayırır. Bizce bu, bir testin bir hatadan doğmuş olmasının bu ayrımda rol oynamadığı anlamına gelir. Ayrıntıları *F.I.R.S.T.* notunda.
- **Kapsam bir hedef değildir** (*Testing Overview* bölümü, *A Note on Code Coverage* başlığı): Kapsam yalnızca hangi satırların çalıştığını ölçer, sonucun denetlenip denetlenmediğini ölçmez. Yüzde 80 gibi bir eşik konduğunda bu eşik kısa sürede taban olmaktan çıkar, tavana dönüşür. Google'ın kitabı kapsamın yalnızca küçük testlerden ölçülmesini önerir. Asıl sorunun, kullanıcıların beklediği davranışların test edilip edilmediği olduğunu söyler.
- **Kapsam bir sinyaldir** (Google Testing Blog, *Code Coverage Best Practices*, 2020):
  - Kapsam kayıplı ve dolaylı bir ölçüdür, tek doğruluk kaynağı olarak kullanılmamalıdır. Değerinin büyük kısmı, neyin kapsanmadığını göstermesidir.
  - Her ürüne uyan ideal bir sayı yoktur. Google'ın genel kılavuzu yüzde 60'ı kabul edilebilir, yüzde 75'i takdire değer, yüzde 90'ı örnek düzey sayar, ama ekiplere yukarıdan bir sayı dayatmaz.
  - Proje genelinde yüzde 90'ın üstüne çıkmak büyük olasılıkla emeğe değmez. Her commit için yüzde 99 makul bir hedef, yüzde 90 iyi bir alt sınırdır.
  - Yüksek kapsam test kalitesini garanti etmez ve yanlış bir güven duygusu verebilir. Kapsanan satırların gerçekten doğrulanıp doğrulanmadığını ölçmenin daha iyi yolu mutasyon testidir: kod bilerek bozulur ve testlerin bunu yakalayıp yakalamadığına bakılır.
  - Düşük kapsam ise ürünün geniş bölümlerinin hiç test edilmediğini kesin olarak gösterir.
  - Kapsanmayan satırları kod incelemesinde tek tek konuşmak, bir sayıya takılmaktan daha değerlidir.
  - Bir kodun ne kadar test isteyeceği, işe etkisine, ne sıklıkla değiştiğine, ömrüne ve karmaşıklığına bağlıdır.

**İki kaynak nerede buluşur, nerede ayrılır:** İkisi de yüzdeyi bir sinyal olarak okur. Martin yüksek kapsamı güven kaynağı sayar, ama kapsam düştüğünde nedenine bakar ve önemsiz satırları test etmeye değer bulmaz. Google buna yüksek bir sayının yanlış güven verebileceği uyarısını ve kaba bir sayısal kılavuz ekler. Martin'in "doğrulanmamış hesap" vurgusu da Google'ın "kapsam doğrulamayı ölçmez" uyarısına denk düşer. Google'ın kendi içinde küçük bir vurgu farkı vardır: kitap kapsamın yalnızca küçük testlerden ölçülmesini önerir, blog yazısı ise entegrasyon testlerinin kapsamını da önemli sayar.

## Bu projede yaşadığımız

**Hata düzeltmeleri testleriyle geldi.** 6 ve 7 Kasım 2026'da yapılan üç hata düzeltmesinin üçü de testiyle birlikte commit edildi. Testlerin hiçbiri ayrı bir klasöre gitmedi, hepsi sınadıkları davranışın testlerinin yanına girdi:

- PDF kaydetme, kayıt sırasında kullandığı geçici dosya yüzünden hedefin yanındaki bir dosyayı ezebiliyordu. Düzeltmeyle gelen test, PDF kaydetme testlerinin arasına `a_file_beside_the_target_is_left_alone` adıyla eklendi.
- Önizleme şablonu, destek ekipmanı ya da yedek parça gerekmediğini söyleyen modülleri basmıyordu. Düzeltmeyle gelen test, gereksinim tablosu testlerinin arasına `lists_that_say_none_are_needed_still_print` adıyla eklendi.
- İç bir varlıkla başlayan paragraf yanlışlıkla düzenlenebilir sayılıyordu. Bu düzeltme yeni bir test yazmadı, reddedilen düzenleme komutlarını sınayan mevcut testi genişletti. Testin girdisine yeni bir paragraf türü eklendi ve bu yüzden testteki öteki vakaların sıra numaraları da kaydı. Google ise hata düzeltmesinin genellikle mevcut testlere dokunmadan eksik vakayı eklemesini önerir. Burada mevcut test değişmişti.

Test adlarının hiçbiri bir hata numarası taşımıyordu, hepsi davranışı anlatıyordu.

**Uç durumlar baştan yazıldı.** 6 Kasım 2026'daki ilk sürümde dosya izleme testleri; var olmayan klasörü, daha eski tarihli bir dosyanın eklenmesini ve ağacın yukarısına dönen bir bağlantıyı, olağan durumlarla aynı test modülünde sınıyordu. Bu uç durumlar bir hatadan sonra değil, kodla birlikte yazılmıştı.

**Clean Code deposunda.** 29 Eylül 2026'da clean-code-philosophy deposunda, bir ölçüm aracının demet anahtarlı bir indeks gördüğünde çöktüğü bir hata düzeltildi. Test, o ölçümün öteki testlerinin bulunduğu dosyaya eklendi ve adı davranışı anlatıyordu. Aynı gün biraz sonra açılan ayrı bir klasör ise yalnızca gerçek dış araçlara dokunan öğrenme testleri içindi.

## Kitabın söylemedikleri

- Uç durum ya da hata testlerinin nerede tutulacağı, ayrı bir klasöre ya da etikete konup konmayacağı söylenmiyor.
- Hata testlerinin hata numarasıyla mı davranışla mı adlandırılacağı söylenmiyor. Kitaptaki örneklerde numara da geçiyor.
- Hedef bir kapsam yüzdesi verilmiyor.
- Kapsanan satırın gerçekten doğrulanıp doğrulanmadığını ölçmekten, yani mutasyon testinden söz edilmiyor.
- Uç durumları otomatik olarak üreten yöntemlerden, örneğin rastgele girdiyle sınamaktan eşzamanlılık dışında söz edilmiyor. Eşzamanlılık bölümü ve eki, ayarları rastgele değiştirerek test çalıştırmayı önerir.
