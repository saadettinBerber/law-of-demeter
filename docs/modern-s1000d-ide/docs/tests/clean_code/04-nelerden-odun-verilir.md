---
title: "Nelerden ödün verilir?"
ikon: ⚖️
kapak: yellow
tarih: 2026-11-07
etiketler: [clean-code, testler]
ozet: Test kodunda kaynak kullanımından ödün verilebilir, okunabilirlikten verilemez; kapatılan bir testin gerekçesi görünür olmalı, kırılan bir testi susturmak ise bir güvenlik önlemini devre dışı bırakmaktır.
---

:::callout ⚖️ yellow
**Test kodunda kaynak kullanımından ödün verilebilir, okunabilirlikten verilemez.** Bir testin işlemciyi ya da belleği idareli kullanması ikinci plandadır. Bir test kapatılacaksa neden kapalı olduğu görünür olmalıdır. Kitap kapatılmış bir testi, belirsiz bir gereksinim hakkında sorulmuş bir soru olarak da kullanır. Kırılan bir testi "sonra düzeltirim" diye susturmak ise ödün değil, bir güvenlik önlemini devre dışı bırakmaktır.
:::

Bu not serinin dördüncü parçası. Önceki notlar temiz testin nasıl yazıldığını ve hangi kurallara uyduğunu anlatıyordu. Bu not sınırları çiziyor: test kodu hangi konularda üretim kodundan daha gevşek olabilir, hangi konularda olamaz.

## Çifte standart: kaynak kullanımı gevşer, okunabilirlik gevşemez

Kitabın *Birim Testleri* bölümündeki *Çifte Standart* başlığı, ilk notta anlatılan kirli testli ekibe tek bir noktada hak verir: testlere hizmet eden kodun mühendislik ölçütleri gerçekten farklıdır. Martin'e göre bu kod da sade ve anlaşılır olmak zorundadır, yalnızca üretim kodu kadar verimli olması beklenmez. Çünkü testin çalıştığı ortamın ihtiyaçları üretim ortamınınkinden çok farklıdır.

Martin'in örneği, prototipini yazdığı bir ortam denetim sistemidir. Testlerin denetlediği durum dizgesi en basit yoldan, parçalar art arda eklenerek kurulur. Daha verimli ama daha hantal bir yol da vardır. Sistem gömülü ve gerçek zamanlı olduğu için üretimde işlemci ve belleğin kısıtlı olması muhtemeldir. Test makinesinde ise büyük olasılıkla böyle bir kısıt yoktur. Martin bu yüzden testte basit yolu seçer.

Aynı örnekte ikinci bir gevşeme de vardır. Durum dizgesi, okuyucudan büyük ve küçük harflerin "açık" ve "kapalı" anlamına geldiğini akılda tutmasını ister. Martin bunun zihinsel eşleştirmeden kaçınma kuralını neredeyse çiğnediğini kabul eder, ama testte yerinde bulur.

Martin'e göre testte gevşetilebilecek olan çoğunlukla kaynak kullanımıdır, okunabilirlik değil. Temizlikten verilen ödün hiçbir zaman bu kapsama girmez.

## Önemsiz görünen testi atlama

Kitabın *Kokular ve Sezgisel Kurallar* bölümü test kokularını *Testler* adlı ayrı bir grupta toplar. Bu gruptaki *Önemsiz Testleri Atlamayın* başlığına göre "çok basit" görünen bir testi yazmamak bir tasarruf değildir. Böyle bir test az emekle yazılır ve kodun nasıl davrandığını belgelediği için emeğinin karşılığını fazlasıyla verir.

## Kapatılan test bir sorudur

Aynı gruptaki *Yoksayılan Bir Test, Bir Belirsizlikle İlgili Bir Sorudur* başlığı, bir testi kapatmanın meşru bir yolunu anlatır. Gereksinim net değilse, belirsiz davranış hakkındaki soru kapatılmış bir test olarak yazılabilir. Test derleniyorsa çalıştırılmamak üzere işaretlenir, derlenmiyorsa yorum satırına alınır. Bizce böyle bir test, yanıt gelene kadar sorunun kendisini kodun içinde tutar.

Kitabın *SerialDate'i Yeniden Düzenleme* bölümündeki *Önce, Çalıştırın* başlığı bunun bir örneğini gösterir. Martin, yeniden düzenleyeceği tarih sınıfı için kendi test paketini yazar. Geçmeyen testlerin çoğunu yorum satırına alır. Bunlar sınıfın sahip olması gerektiğini düşündüğü davranışlardır ve yeniden düzenleme sırasında onları da geçirmeyi hedefler. Haftanın günlerinin "tues" ve "thurs" gibi kısaltmalarını sınayan iki testi ise kapalı bırakır, çünkü bu kısaltmaların desteklenmesi gerekip gerekmediğinden emin değildir. Bu ikisi yanıtı belli olmayan sorulardır.

Kitabın *Yorumlar* bölümündeki *Sonuçlara Dair Uyarı* başlığı, bir testin neden kapatıldığını açıklayan bir yorum örneği verir. Martin bugün böyle bir testin gerekçesini anlatan bir açıklamayla, örneğin "çalışması çok uzun sürüyor" notuyla yoksay olarak işaretleneceğini söyler. Bizce bu iki başlığın ortak noktası, kapatılan testin neden kapalı olduğunun görünür olmasıdır.

## Kırık testi susturmak bir ödün değildir

*Kokular ve Sezgisel Kurallar* bölümünün *Genel Kurallar* grubundaki *Güvenlik Önlemlerinin Geçersiz Kılınması* başlığı Çernobil'den söz eder. Santral yöneticisi, bir deneyi yürütmeyi zahmetli hale getiren güvenlik önlemlerini birer birer devre dışı bırakmıştı. Martin'e göre güvenlik önlemlerini aşmak her zaman risk taşır. Uyarıları kapatıp derlemeyi geçirmek uzun hata ayıklama günlerine yol açabilir. Kırılan bir testi kapatıp sonra düzelteceğini söylemeyi de borcu bedava para sanmaya benzetir.

Bu başlıkla *SerialDate* örneği ilk bakışta çelişir gibi görünür, çünkü Martin orada da geçmeyen testleri kapatır. Bizce fark şudur: SerialDate'te kapatılan testler hiç geçmemiş, istenen davranışı anlatan yeni testlerdi. Kınanan ise geçen bir testin kırıldığında susturulmasıdır. Belirsiz bir gereksinimi soran ya da henüz yazılmamış bir davranışı anlatan kapalı test meşrudur. Kırıldığı için, düzeltmeyi ertelemek amacıyla kapatılan test ise bir güvenlik önlemini devre dışı bırakmaktır.

## Yavaş test, çalıştırılmayan testtir

*Testler* grubundaki *Testler Hızlı Olmalıdır* başlığı ödün verilemeyecek bir şeyi daha söyler. Martin'e göre yavaş bir test sonunda hiç çalıştırılmaz: zaman daraldığında paketten çıkarılan testler yavaş olanlardır. Bu yüzden testleri hızlı tutmak için gereken her şey yapılmalıdır.

Bizce bu kural çifte standartla çelişmez. Tek bir testin içindeki küçük bir verimsizlik kabul edilebilir. Test paketinin bütününü yavaşlatan bir verimsizlik ise kabul edilemez, çünkü sonunda testin kendisinden vazgeçilmesine yol açar.

## Google bu konuda

*Software Engineering at Google* kitabının *Unit Testing* ve *Testing Overview* bölümleri, test kodunun üretim kodundan farklı olabileceği noktaları başka bir yönden ele alır.

- **Tekrardan ödün verilir.** Google testlerde DRY'ı tamamen bırakmaz, ama açıklık için DAMP'ı öne alır: test daha açık oluyorsa biraz tekrar kabul edilir. Martin'in açıkça saymadığı bir gevşemedir bu. Ayrıntıları *Temiz test nasıl yazılır?* notunda.
- **Mantıktan ödün verilmez.** Google test gövdesinde döngü, koşul ya da hesaplama istemez, çünkü testlerin testi yoktur. Döngü gerektiren dar kapsamlı doğrulama yardımcılarına ise izin verir. Bu konuda test kodu üretim kodundan daha katı bir kurala tabidir.
- **Ortak test altyapısı üretim kodu gibidir.** Birçok test paketinin paylaştığı test altyapısı, Google'a göre üretim koduna daha çok benzer ve her zaman kendi testlerine sahip olmalıdır. Bizce bu, gevşekliğin tek tek testlerin gövdesi için geçerli olduğunu, onların dayandığı ortak araçlar için geçerli olmadığını gösterir.
- **Her şey doğrudan test edilmez.** Yalnızca bir iki sınıfa hizmet eden bir yardımcı sınıf, büyük olasılıkla ayrı bir birim sayılmaz ve onu kullanan sınıflar üzerinden test edilir. Sahiplerine sormadan herkesin kullanabildiği her şey ise neredeyse kesinlikle doğrudan test edilmelidir.
- **Kararsız testleri yeniden çalıştırmak geçici bir takastır.** *Testing Overview* bölümüne göre kararsızlık oranı düşükken bu testleri otomatik olarak yeniden çalıştırmak, işlemci zamanını mühendis zamanıyla takas eden kabul edilebilir bir ödündür. Ama bu yalnızca kök nedeni çözmeyi erteler. Ayrıntıları *F.I.R.S.T.* notunda.

Bizce iki kaynak birlikte şunu söyler: test kodu üretim kodundan iki yönde ayrılır. Kaynak kullanımı ve tekrar konusunda daha gevşektir. Mantık ve sadelik konusunda daha katıdır.

## Bu projede yaşadığımız

**Kırık testleri kapatmadık.** 7 Kasım 2026'da `main` dalında dört test kırık çıktığında hiçbirini kapatmadık. Nedenlerini araştırıp *Kırık testler: ön koşul sırası ve macOS'ta geri alma* notuna yazdık. Üç testin kırılmasının nedeni, örnek modülde iki ön koşulun yer değiştirmesiydi. Bu araştırma bir soru doğurdu: ön koşullar hangi sırada kalmalıydı? Sırayı değiştiren commit nedenini söylemiyordu. Kitap belirsiz bir davranışı kapatılmış bir testle sormaya izin verir. Bizim sorumuz ise davranışla değil test verisiyle ilgiliydi, çünkü kod doğru çalışıyordu. Soruyu karar bekleyen bir madde olarak o nota yazdık ve testleri kırık bıraktık.

**Raporda görünmeyen atlamalar.** Aynı akşam `main` dalında üç test yalnızca Unix türevi sistemlerde derleniyordu. Windows'ta bu testler hiç derlenmeyeceği için test raporunda atlandıkları da görünmezdi. Neden yalnızca Unix'te çalıştıkları testlerin kodundan çıkarılabiliyordu, ama raporda hiçbir iz kalmazdı.

Bu testlerden biri, salt okunur bir klasöre PDF kaydetmenin başarısız olduğunu sınıyordu. Test yönetici (root) olarak çalıştırıldığında klasör izinleri işlemediği için sınama yapılamıyordu. Test bu durumda "atlandı" diye bir satır yazıp hiçbir şey denetlemeden bitiyor, raporda ise geçmiş görünüyordu. Kitabın istediği şey, atlanan bir testin gerekçesinin görünür olmasıydı. Burada gerekçe yazılıyordu, ama raporun söylediği sonuç gerçeği yansıtmıyordu.

**Silinemezse bırakılan geçici klasör.** Dosya yazan testlerin ortak yardımcısı, test bitince geçici klasörünü siliyordu. Silme başarısız olursa hata vermiyor, klasörü işletim sisteminin geçici dizininde bırakıyordu. Koddaki yorum gerekçesini açıklıyordu: zaten başarısız olmuş bir testin temizliği sırasında ikinci bir hata çıkarsa bütün test çalıştırması yarıda kesilirdi. Bizce bu, ortak bir test aracında bilinçli bir takastı: artık bir klasör bırakmak, çalıştırmanın çökmesinden ucuzdu. Bu yardımcı birçok test modülünün kullandığı ortak bir araçtı. Google'ın ölçütüne göre böyle bir aracın kendi testleri olmalıydı, o dönemde ise yoktu.

**Minik sayfa.** Önizleme panelinin testleri çok küçük bir sayfa şablonu kullanıyordu. Koddaki yorum bunun Typst'i hızlı tuttuğunu söylüyordu. Testlerin amacı sayfa düzeni değil, panelin davranışıydı. Bu, testleri hızlı tutmak için yapılmış küçük ama bilinçli bir tercihti.

## Kitabın söylemedikleri

- Çifte standardın açık gerekçesi verimlilik. Aynı örnekte zihinsel eşleştirmeye yaklaşan bir dizge de kabul ediliyor, ama tekrar gibi başka ölçütlerin testte gevşetilip gevşetilemeyeceği söylenmiyor. Bu konu Google'dan geliyor.
- Kapatılan bir testin ne kadar süre kapalı kalabileceği ve nasıl takip edileceği anlatılmıyor.
- Yalnızca belirli platformlarda çalışabilen testlerin nasıl atlanacağı anlatılmıyor. Eşzamanlılık bölümü yalnızca testlerin her hedef platformda çalıştırılmasını önerir.
- Kendi kodumuzda neyin test edilmeyeceğine dair bir kural yok. Kitap aksine önemsiz görünen testlerin bile atlanmamasını öğütlüyor. James Grenning'in yazdığı *Sınırlar* bölümü ise üçüncü taraf kodu test etmeyi bizim işimiz saymaz, ama o kodu öğrenmek için yazılan testleri yararlı bulur.
