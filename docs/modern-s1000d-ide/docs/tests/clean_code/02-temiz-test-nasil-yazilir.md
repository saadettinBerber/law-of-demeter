---
title: "Temiz test nasıl yazılır?"
ikon: ✍️
kapak: green
tarih: 2026-11-07
etiketler: [clean-code, testler]
ozet: Temiz test okunabilir testtir; kur-işlet-denetle yapısıyla yazılır, testlere özgü bir dil kullanır, her testte tek kavramı sınar ve hata yolunu önce test eder.
---

:::callout ✍️ green
**Temiz test, okunabilir testtir.** Testte neyin kurulduğu, ne yapıldığı ve ne beklendiği bir bakışta görünmelidir. Her test tek bir kavramı sınar, adı da o kavramı söyler. Testle ilgisi olmayan ayrıntılar yardımcı fonksiyonlara taşınır, ama sonucu belirleyen değer testin içinde kalır.
:::

Bu not serinin ikinci parçası. *Neden temiz test?* notu testlerin neden özen istediğini anlatıyordu. Bu not o özenin testin yazımında nasıl göründüğünü anlatıyor.

## Okunabilirlik, okunabilirlik, okunabilirlik

Kitabın *Birim Testleri* bölümündeki *Temiz Testler* başlığı, temiz testin ölçüsünü tek bir niteliğe bağlar ve onu üç kez tekrarlar: okunabilirlik. Martin'e göre okunabilirlik testlerde belki de üretim kodundakinden bile önemlidir. Okunabilir testin tarifi her okunabilir kodunkiyle aynıdır: açık, sade ve az sözle çok şey anlatan kod.

Kitap bunu FitNesse projesinden alınmış bir test grubuyla gösterir. Testler, amaçla hiç ilgisi olmayan ayrıntılarla doludur: metni yol nesnesine çeviren çağrılar, yanıtlayıcıyı kurup yanıtı alan ve türünü dönüştüren satırlar, istek adresini parça parça birleştiren kod. Okuyucu testin neyi sınadığını görebilmek için önce bu ayrıntıları çözmek zorunda kalır. Martin aynı testleri yeniden düzenler. Yeni hali aynı işi yapar, ama yalnızca testin gerçekten ihtiyaç duyduğu parçaları gösterir ve okuyan kişi testin ne yaptığını hemen anlar.

## Kur, işlet, denetle

Yeniden düzenlenen testlerde kitabın KUR-İŞLET-DENETLE (BUILD-OPERATE-CHECK) dediği desen göze çarpar. Her test üç bölüme ayrılır:

1. **Kur:** testin üzerinde çalışacağı veri hazırlanır.
2. **İşlet:** sınanan davranış tetiklenir.
3. **Denetle:** ortaya çıkan sonuç beklenenle karşılaştırılır.

Bu üç bölüm testte gözle ayrı görünmelidir. Google'ın davranışları ayırırken kullandığı given-when-then parçaları da bu üç adımla örtüşür.

## Testlere özgü bir dil

*Alana Özgü Test Dili* başlığı, yeniden düzenlenen testlerin arkasındaki tekniği adlandırır. Testler sistemi doğrudan, programcıların kullandığı API ile sürmez. O API'yi çağıran yardımcı fonksiyonlar yazılır. Kitaptaki örnekte bunlar "şu sayfaları oluştur" ya da "şu isteği gönder" diyen fonksiyonlardır. Yardımcılar biriktikçe testlerin kendi sözlüğü oluşur: testi yazan daha az ayrıntıyla uğraşır, sonradan okuyan da testi daha hızlı anlar.

Martin bu dilin önceden tasarlanmadığını vurgular. Ayrıntıyla dolmuş test kodu sürekli yeniden düzenlendikçe zamanla gelişir.

Martin'in *Çifte Standart* başlığında verdiği örnek de aynı tekniği kullanır. Bir ortam denetim sisteminin testi, "aşırı soğuk" durumunu tek bir yardımcıyla kurar. Isıtıcının, fanın, soğutucunun ve iki alarmın durumunu da tek bir dizgeyle denetler: büyük harf açık, küçük harf kapalı anlamına gelir. Martin bunun okuyucudan bir harf eşlemesini akılda tutmasını istediğini ve *Zihinsel Eşleştirmeden Kaçınmak* kuralını neredeyse çiğnediğini kabul eder. Yine de anlam bir kez öğrenilince testin çok daha kolay okunduğunu ve bu durumda yerinde olduğunu söyler. Aynı örnek, test kodunda verimlilikten ödün verilebileceğini de gösterir. Bu konu serinin *Nelerden ödün verilir* notunun konusu.

## Her testte tek kavram

*Test Başına Tek Assert* başlığı, her testte yalnızca bir assert olmasını savunan bir görüşü ele alır. Martin bu görüşü bir testi ikiye bölerek dener. Bölerken yardımcı fonksiyonların adlarını da given-when-then kalıbına çevirir, örneğin `givenPages` ve `whenRequestIsIssued`, ve bunun testleri daha da okunur yaptığını söyler. Ama bölmek tekrar doğurur. Bu tekrarı gidermek için gereken düzenek, örneğin ortak bir temel sınıf ya da ortak kurulum, böyle küçük bir sorun için fazla ağırdır. Martin sonunda birden çok assert'li testi tercih eder. Tek assert kuralını iyi bir kılavuz sayar ama katı bir kural olarak görmez. Ona göre yapılması gereken, assert sayısını en aza indirmektir.

*Test Başına Tek Kavram* başlığı daha iyi bir kural önerir: her test fonksiyonu tek bir kavramı sınamalıdır. Kitabın örneği, bir tarihe ay ekleyen fonksiyon için yazılmış `testAddMonths` adlı tek bir testtir. Üstündeki yorum onu "çeşitli testler" diye tanıtır. Test birbirinden bağımsız üç şeyi art arda sınar. Okuyan kişi her bölümün neden orada durduğunu ve neyi sınadığını kendisi çıkarmak zorundadır.

Martin bu testi üç ayrı given-when-then senaryosuna böler. Senaryolar yan yana yazılınca aralarında gizlenmiş genel kural ortaya çıkar: bir tarihe ay eklendiğinde gün, yeni ayın son gününü aşamaz. Bu kural eksik bir testi de gösterir: 28 Şubat'a bir ay eklenince sonuç 28 Mart olmalıdır. Karışık yazılmış testte bu eksiklik görünmüyordu.

Sonuç olarak sorun birden çok assert değil, birden çok kavramdır. Kural iki parçalıdır: her testte tek kavram sınanır ve o kavram için assert sayısı en aza indirilir.

## Hata yolunu önce test et

Kitabın Michael Feathers'ın yazdığı *Hata Yönetimi* bölümündeki *Önce Try-Catch-Finally İfadenizi Yazın* başlığı, test sırasıyla ilgili bir öneri yapar. İstisna (exception) fırlatabilecek bir kod yazılırken Feathers işe try-catch-finally yapısını kurarak başlamayı önerir. Gerekçesi şu: try bloğu bir işlem (transaction) gibidir. İçinde ne olursa olsun, catch bloğu programı tutarlı bir durumda bırakmalıdır. Yapıyı baştan kurmak, kodu kullanacak kişinin bir şey ters gittiğinde neyle karşılaşacağını en başta belirler.

Kitabın örneği, dosyadan kayıt okuyan bir fonksiyondur:

1. İlk test, dosya yoksa bir istisna fırlatılmasını bekler. Fonksiyon henüz boş olduğu için test kırmızıdır.
2. Fonksiyon geçersiz bir dosyayı açmaya çalışacak şekilde yazılır. İstisna yakalanır ve test geçer.
3. Yakalanan istisnanın türü, gerçekte fırlatılan türe daraltılır.
4. Hata kapsamı kurulduktan sonra, her şeyin yolunda gittiği akış TDD ile doldurulur. Bu kod artık hiçbir şey ters gitmeyecekmiş gibi yazılabilir.

Feathers'ın önerisi kısaca şudur: önce istisnayı zorlayan testleri yaz, sonra bu testleri geçirmek için hata yakalayıcıya (handler) davranış ekle. Böylece try bloğunun kapsamı en başta kurulur ve bu kapsamın bir işlem gibi davranması korunur.

## Google bu konuda

*Software Engineering at Google* kitabının *Unit Testing* bölümü aynı hedefe kendi kurallarıyla varır ve birkaç noktada daha katıdır.

- **Metodu değil davranışı test et.** Google'a göre davranış, sistemin belli bir durumda bir dizi girdiye nasıl yanıt vereceğine dair bir güvencedir. Her davranış için ayrı bir test yazılır. Metot başına tek test yazmak, metot büyüdükçe testi karmaşıklaştırır, çünkü bir metot çoğu zaman birkaç davranışı birden taşır. Her davranışa ayrı test yazma kuralı, Martin'in "her testte tek kavram" kuralıyla aynı yere varır.
- **Testin adı davranışı anlatsın.** İyi bir ad, sisteme yapılan eylemi ve beklenen sonucu özetler. Bir test kırıldığında raporda çoğu zaman ilk, bazen de tek görülen şey adıdır. Test adlarını kimse koddan çağırmadığı için uzun olmaları sorun değildir. Adında "ve" geçen bir test muhtemelen iki davranışı sınıyordur ve bölünmelidir.
- **Test eksiksiz ve öz olsun.** Test gövdesi, sonucu anlamak için gereken her şeyi içermeli, ilgisiz hiçbir şeyi içermemelidir. Notun başındaki "sonucu belirleyen değer testin içinde kalır" kuralı bu ilkeye dayanır.
- **Testte mantık olmasın.** Testte döngü, koşul ya da hesaplama bulunmaz, çünkü testlerin testi yoktur. Bir testin doğruluğu ona bakınca anlaşılmalıdır. Google'ın örneğinde beklenen adresi tek satırda birleştiren bir ifade, adreste çift eğik çizgi hatasını gizlemişti. Adres açıkça yazılınca hata görünür oldu.
- **DRY değil, DAMP.** Google testler için "açıklayıcı ve anlamlı ifadeler" (Descriptive And Meaningful Phrases) ister. Testi daha açık yapıyorsa biraz tekrar kabul edilir.
  - Ortak kurulum, sınanan nesneyi kurmak için kullanılabilir.
  - Ama bir test kurulumdaki belirli bir değere dayanıyorsa o değer testin içinde açıkça yazılmalıdır.
  - Her testin sonunda çağrılan genel bir "doğrula" yardımcısı kötü bir alışkanlıktır. Testi davranıştan uzaklaştırır, bir hata çıktığında da birçok test birden kırılır.
- **Hata mesajı açık olsun.** Kırılan bir testin mesajı beklenen sonucu, gerçekleşen sonucu ve ilgili parametreleri göstermelidir.
- **Değişmeyen testlere çabala.** Saf bir yeniden düzenleme mevcut testleri değiştirmemelidir. Yeni bir özellik yeni testler ekler. Bir hata düzeltmesi de eksik kalan test vakasını ekler. Mevcut testler yalnızca sistemin davranışı bilerek değiştirildiğinde değişir. Bunun için testler sistemi, kullanıcısının çağırdığı biçimde, dışa açık API'si üzerinden çağırmalıdır. İç ayrıntılara uzanan testler, kullanıcıyı hiç etkilemeyen bir değişiklikte bile kırılır.

**İki kaynağın ayrıldığı nokta:** Martin ayrıntıyı yardımcı fonksiyonlara taşımayı öne çıkarır. Google da yardımcılara izin verir, ama sonucu belirleyen değerin bir yardımcının ya da ortak kurulumun içinde saklanmasına karşıdır. İki kural birlikte okunabilir: yardımcı, testle ilgisi olmayan ayrıntıyı saklar, sonucu belirleyen değeri saklamaz.

## Bu projede yaşadığımız

**Önizleme testlerindeki küçük test dili.** 7 Kasım 2026'da önizleme panelinin testleri birkaç yardımcıyla yazılmıştı. `previewing` yardımcısı, bir klasördeki dosyayı önizleyen bir paneli kuruyordu. `notice_change` yardımcısı da test saatini ilerletip panelin dosya değişikliğini fark etmesini sağlıyordu. Böylece test gövdeleri pencere kurma ve bekleme ayrıntılarıyla değil, sınanan davranışla başlıyordu. Bu, kitabın anlattığı alana özgü test dilinin küçük bir örneğiydi.

**Testte görünmeyen değer.** Aynı gün ön koşul sırasıyla ilgili üç test kırıldı. Bu testler beklenen metni kendi içinde yazıyordu, ama o metnin neden birinci blokta olduğu testte değil, XML dosyasındaydı. Testi okuyan biri bunu görmek için dosyayı açmak zorundaydı. Dosyada sıra değişince testler de kırıldı. Google'ın ortak kurulum için yaptığı uyarı bu duruma da uyuyordu: test belirli bir değere dayanıyorsa o değer testin içinde görünmelidir.

**Adında "ve" geçen test.** 7 Kasım 2026'da Rust geliştirme rehberimizi incelerken `rejects_blank_text_and_preserves_valid_formatting` adlı bir test fark ettik. Adındaki "and", testin iki ayrı kavramı birlikte sınadığını gösteriyordu: boş metnin reddedilmesi ve geçerli metnin biçiminin korunması. Rehberin kendisi her testte tek bir davranışın sınanmasını öğütlüyordu. Bu test iki ayrı teste bölünmeye adaydı.

## Kitabın söylemedikleri

- *Birim Testleri* bölümünde testte döngü ya da koşul kullanmak hakkında açık bir kural yok. Bu kural Google'dan geliyor.
- Yardımcı fonksiyonların sonucu belirleyen değeri saklama riskinden söz edilmiyor. Bu uyarı da Google'dan geliyor.
- Test adları için bir adlandırma kalıbı ya da ad uzunluğu önerisi yok. Given-when-then yalnızca yardımcı fonksiyonların adlarında geçiyor.
- Bölüm, kırılan bir testin mesajının neleri içermesi gerektiğinden söz etmiyor.
