---
title: "F.I.R.S.T.: temiz testin beş kuralı"
ikon: 🏁
kapak: orange
tarih: 2026-11-07
etiketler: [clean-code, testler]
ozet: Temiz testler hızlı, bağımsız, tekrarlanabilir, kendini doğrulayan ve zamanında yazılmış olmalıdır; bu projede en çok ders çıkardığımız kural testin her ortamda aynı sonucu vermesi oldu.
---

:::callout 🏁 orange
**Temiz testler beş kurala uyar: hızlıdır, birbirinden bağımsızdır, her ortamda tekrarlanabilir, sonucunu kendisi söyler ve zamanında yazılır.** Kuralların İngilizce baş harfleri F.I.R.S.T. kısaltmasını oluşturur. Bu projede en çok ders çıkardığımız kural tekrarlanabilirlik oldu: testin sonucu, çalıştığı makineye, işletim sistemine ya da saate bağlı olmamalıdır.
:::

Bu not serinin üçüncü parçası. İlk iki not testlerin neden özen istediğini ve nasıl okunur yazılacağını anlatıyordu. Bu not, kitabın *Birim Testleri* bölümündeki *F.I.R.S.T.* başlığında sıralanan beş kuralı ele alıyor. Tekrarlanabilirlik için kitabın eşzamanlılık bölümüne de başvuruyor, çünkü testleri farklı ortamlarda çalıştırmak en ayrıntılı orada anlatılıyor. O bölümü ve kitabın eşzamanlılık ekini Martin değil, Brett L. Schuchert yazmıştır.

## Hızlı (Fast)

Yavaş bir test paketi seyrek çalıştırılır. Seyrek çalıştırılan testler bir sorunu ancak düzeltmesi pahalılaştıktan sonra gösterir. Kodu temizlemek de riskli görünmeye başlar ve Martin'e göre kod sonunda çürür.

## Bağımsız (Independent)

Hiçbir test, kendinden önce çalışan bir testin bıraktığı duruma güvenmemelidir. Her test tek başına da, başka testlerle birlikte herhangi bir sırada da çalıştırılabilmelidir.

Testler birbirine bağlıysa ilk kırılan test arkasından bir kırılma zinciri getirir. Asıl sorun bu zincirin içinde kaybolur, sonraki testlerdeki gerçek kusurlar da görünmez olur.

## Tekrarlanabilir (Repeatable)

Bir test, ister üretim ortamında ister test ortamında, ister ağa bağlı olmayan bir dizüstü bilgisayarda çalışsın, aynı sonucu vermelidir. Martin ortama bağlı testlerin iki bedelini sayar: kırıldıklarında ortam her zaman hazır bir bahane olur, ortam kurulu değilse de testler hiç çalıştırılamaz.

### Saati testin eline vermek

*Birim Testleri* bölümünün açılışında Martin, doksanlarda yazdığı bir zamanlayıcıyı bugün nasıl test edeceğini anlatır. Kodu işletim sisteminin saatinden yalıtır, zaman fonksiyonlarını testte sahte bir saatle değiştirir ve zamanı testin içinden adım adım ilerletirdi. Böylece zaman üzerindeki denetim teste geçer. Bizce sonucun makinenin hızından bağımsız olmasını sağlayan da budur.

### Farklı platformlarda çalıştırmak

Kitabın *Eşzamanlılık* bölümündeki *İş Parçacıklı Kodu Test Etme* başlığı, tekrarlanabilirliğin öbür yüzünü gösterir. Bölümün yazarı Schuchert ve ekibi 2007 ortasında bir eşzamanlı programlama kursu hazırlarken bunu yaşamıştı. Kurs OS X'te hazırlanmış, sanal makinede çalışan Windows XP'de sunulmuştu. Hata koşullarını göstermek için yazılan testler XP'de OS X'tekinden daha seyrek kırılıyordu. Oysa kodun her durumda hatalı olduğu biliniyordu. Farkı, işletim sistemlerinin iş parçacıklarını farklı yönetmesi yaratıyordu.

Buradan çıkan öneri *Farklı Platformlarda Çalıştırın* alt başlığında yer alır: testler, programın çalışabileceği her ortamda çalıştırılmalıdır. Öneri özellikle iş parçacıklı kodun hedeflenen her platformda erken ve sık denenmesini ister.

Bizce bu öneri tekrarlanabilirlik kuralıyla çelişmez, onu tamamlar. Bir testin vereceği karar ortama bağlı olmamalıdır. Testi farklı ortamlarda çalıştırmak da kodun ya da testin ortama gizlice nerede bağlandığını ortaya çıkarır.

### "Tek seferlik hata" yoktur

*Sahte Başarısızlıkları Eşzamanlılık Sorunu Adayı Olarak Değerlendirin* alt başlığında Schuchert, iş parçacıklı kodda ara sıra görülen bir kırılmanın büyük olasılıkla bir eşzamanlılık hatası olduğunu söyler. Böyle bir hata binlerce, hatta milyonlarca çalıştırmada bir görülebilir. Bu yüzden kırılmayı kozmik ışına ya da donanım aksaklığına bağlayıp "tek seferlik" saymak kolaydır. Schuchert tek seferlik hatanın var olmadığını varsaymayı önerir. Görmezden gelinen her kırılma, yanlış olabilecek bir yaklaşımın üstüne yeni kod yığılmasına izin verir.

*İş Parçacıklı Kodu Test Etme* başlığının girişi de aynı uyarıyı yapar: bir test kırıldıysa, sonraki çalıştırmada geçmesi o kırılmayı görmezden gelmek için gerekçe değildir. Kırılmanın izi sürülmelidir.

*Önce İş Parçacıklı Olmayan Kodunuzu Çalışır Hale Getirin* alt başlığı bir öneri daha yapar. İş parçacıklarından habersiz düz nesneler, iş parçacığı ortamının dışında sınanabilir. Sistemin ne kadar büyük kısmı böyle nesnelerde durursa o kadar iyidir. Böylece iş parçacığı hatalarıyla öteki hatalar aynı anda kovalanmaz.

### Nadir hatayı sıklaştırmak

Kitabın eşzamanlılık ekindeki *Çok İş Parçacıklı Kodu Test Etme* başlığı konuyu daha da somutlaştırır. Bir makinede hatayı güvenilir biçimde gösterecek şekilde ayarlanan bir test, başka bir makinede, işletim sisteminde ya da Java sanal makinesi sürümünde yeniden ayarlanmak zorunda kalabilir. Schuchert'in önerisi bu yüzden ayarlanabilir testler yazmak ve bunları bir test sunucusunda, ayarları rastgele değiştirerek sürekli çalıştırmaktır. Test kırıldığında hangi koşullarda kırıldığı kaydedilir ve bu çalışma hedeflenen her platformda tekrarlanır.

Schuchert bunun da yetmeyebileceğini söyler. *Başarısızlığı Zorlamaya Çalışarak Kodunuzu Enstrümanlayın* alt başlığı, koda çalışma sırasını değiştiren çağrılar ekleyerek nadir hataları daha sık ortaya çıkarmayı önerir. Ekteki ConTest aracı örneğinde böyle bir enstrümanlama, hatanın görülme sıklığını yaklaşık on milyon denemede birden otuz denemede bire çıkarmıştı.

## Kendini Doğrulayan (Self-Validating)

Bir test ya geçer ya kalır, sonucu başka bir yoruma yer bırakmaz. Testin geçip geçmediğini anlamak için log okumak ya da iki çıktıyı yan yana koyup elle karşılaştırmak gerekiyorsa, bu karar kişiden kişiye değişir. Testleri çalıştırmak da uzun bir elle inceleme işine dönüşür.

## Zamanında (Timely)

Test, onu geçirecek kod yazılmadan hemen önce yazılır. Kod önce yazılırsa test edilmesi zor çıkabilir. Bazı kodlar sonradan test edilemez sayılıp testsiz bırakılabilir, ya da kod baştan test edilebilir biçimde tasarlanmaz. Bu kural, *Neden temiz test?* notunda anlatılan TDD'nin üç yasasıyla örtüşür.

## Google bu konuda

*Software Engineering at Google* kitabının *Testing Overview* bölümü bu kurallara somut ölçütler ekler.

- **Test boyutları.** Google testleri kullandıkları kaynaklara göre üç boyuta ayırır:
  - *Küçük test* tek bir süreçte, birçok dilde de tek bir iş parçacığında çalışır. Uyuyamaz (sleep), diske ya da ağa erişemez. Bağımlılıkları aynı süreçte çalışan test ikizleriyle değiştirilir.
  - *Orta test* birden çok süreç kullanabilir, ama ağda yalnızca kendi makinesine (localhost) bağlanabilir.
  - *Büyük test* birden çok makineye yayılabilir.

  Bu sınırlar hız ve belirlenimcilik (determinism) için konmuştur. Google bazı dillerde bu sınırları araçla zorlar: ağ bağlantısı açmaya çalışan küçük bir test kırılır. Bu ölçütler Martin'in "hızlı" ve "tekrarlanabilir" kurallarını ölçülebilir hale getirir.
- **Hermetik test.** Her test, ortamını kurmak, çalıştırmak ve kaldırmak için gereken her şeyi kendi içinde taşır. Dış ortam ve testlerin çalışma sırası hakkında olabildiğince az varsayımda bulunur, örneğin paylaşılan bir veritabanına dayanmaz. Bu, Martin'in "bağımsız" ve "tekrarlanabilir" kurallarının birleşimidir.
- **Kararsız testler.** Aynı kodla bazen geçip bazen kırılan testlere kararsız (flaky) test denir.
  - Kararsızlık oranı yüzde 1'e yaklaştığında testler değerini yitirmeye başlar. Google'da bu oran binde 1,5 civarındadır ve Google bu testleri düzeltmeye etkin biçimde mühendis zamanı ayırır.
  - Asıl tehlike güven kaybıdır: ekip testlere güvenmezse kırılmalara tepki vermeyi bırakır.
  - Kararsızlığın kaynağı çoğunlukla testin kendi belirlenimci olmayan davranışıdır: saat, iş parçacıklarının sırası, ağ gecikmesi.
- **Ayrıldığı nokta.** Schuchert, sonraki çalıştırmada geçti diye bir kırılmanın görmezden gelinmemesini ister. Google ise düşük oranlarda testleri otomatik olarak yeniden çalıştırmayı, işlemci zamanını mühendis zamanıyla takas eden kabul edilebilir bir çözüm sayar. Ama bunun kök nedeni çözmeyi yalnızca ertelediğini de açıkça söyler. İki kaynak, kararsız testin sonunda düzeltilmesi gerektiğinde birleşir.

## Bu projede yaşadığımız

**Yalnızca macOS'ta kırılan test.** 7 Kasım 2026'da testleri hem macOS'ta hem de Docker içinde Linux'ta çalıştırdık. Silinen bir atıf çipini geri alma testi Linux'ta geçti, macOS'ta kırıldı. Test geri almak için `ctrl-z` tuşlarına basıyordu. Arayüz kütüphanesi GPUI Kit ise geri almayı macOS'ta `cmd-z`'ye, Linux ve Windows'ta `ctrl-z`'ye bağlıyordu. Yani test, Linux ve Windows'taki geri alma kısayoluna bağlı kalmıştı. GPUI Kit'in kendi testlerinde kullanılan ve her sistemde doğru tuşa çevrilen `secondary-z` kısayoluyla denendiğinde test macOS'ta da geçti.

Nedene götüren ipucu, testlerin iki platformda çalıştırılmasıydı. Yalnızca macOS'ta çalışan biri kırılmanın nedenini uygulama kodunda arayabilirdi. Bu, *Farklı Platformlarda Çalıştırın* önerisinin küçük ama gerçek bir örneğiydi.

**Gerçek saat yerine test saati.** Aynı dönemde önizleme paneli, dosya değişikliklerini belli aralıklarla yoklayarak fark ediyordu. Panelin testleri gerçek zamanı beklemek yerine test saatini elle bir saniye ilerletiyordu. Bu, Martin'in zamanlayıcı örneğindeki yaklaşımla aynıydı: zamanı test denetliyordu, sonuç makinenin hızına bağlı değildi.

**Ayrı klasörler ve ortak bir dosya.** Dosya yazan testlerin her biri kendi geçici klasörünü kuruyor ve test bitince siliyordu. Klasör adı testin verdiği addan ve çalışan sürecin numarasından oluşuyordu. Böylece aynı anda çalışan testler birbirinin dosyalarına dokunmuyordu. Bu düzenin zayıf yanı, adların benzersizliğinin elle sağlanmasıydı: iki test aynı adı verseydi aynı klasörü paylaşacaktı.

Belge ve dizgi testleri ise IDE'nin de açtığı ortak örnek klasöründeki dosyaları okuyordu. 7 Kasım'da bu dosyalardan birinde iki ön koşulun yeri değişince üç test birden kırıldı. Testler birbirine değil, paylaştıkları bir dosyaya bağlıydı. Bağımsızlık kuralı açısından bu da bir bağdı.

**Makineye göre değişebilecek fontlar.** 7 Kasım 2026'da bu notları hazırlarken önizlemenin Typst'e verdiği fontlara baktık. Önce makinedeki sistem fontları taranıyor, Typst'in programla birlikte gelen fontları yalnızca yedek olarak ekleniyordu. Örnek şablon Noto Sans fontunu istiyordu ve bu font Typst'in kendi fontları arasında yoktu. Örnek modülün iki sayfa tuttuğunu denetleyen test, Noto Sans'ın makinede kurulu olup olmamasına göre farklı sonuç verebilirdi. O güne kadar bu yüzden kırılan bir test görmemiştik. Tekrarlanabilirlik açısından bu, kırılmadan önce fark edilmiş bir riskti.

## Kitabın söylemedikleri

- Tekrarlanabilirlik maddesi ağ bağlantısı olmayan ortamı anıyor. Ama kuralın nasıl sağlanacağı yalnızca saat ve iş parçacıkları üzerinden anlatılıyor. Font, dosya yolu, satır sonu karakteri ya da klavye kısayolu gibi başka ortam farklarından söz edilmiyor.
- "Hızlı" için bir süre ölçütü yok. Google'ın test boyutları daha somut bir ölçüt veriyor.
- Kararsız testler için karantinaya almak ya da yeniden çalıştırmak gibi bir politika yok. Kitap kırılmanın izinin sürülmesini ve kodun enstrümanlanarak hatanın sıklaştırılmasını öneriyor.
- Testlerin aynı anda, paralel çalıştırılmasından söz edilmiyor. Bağımsızlık kuralı bu durumu dolaylı olarak kapsıyor.
