---
title: "Neden temiz test?"
ikon: 🧪
kapak: blue
tarih: 2026-11-07
etiketler: [clean-code, testler]
ozet: Kirli testler zamanla terk edilir, testsiz kalan kod da değişmekten korkulan ve çürüyen bir yapıya döner; testler bu yüzden üretim koduyla aynı özeni hak eder.
---

:::callout 🧪 blue
**Testler, üretim koduyla aynı özeni hak eder.** Testler kodla birlikte değişmek zorundadır. Kirli testler zor değişir, zamanla yük sayılır ve terk edilir. Test kalmayınca değişiklik yapmaktan korkulur ve kod çürür. Testleri temiz tutmak, kodu değiştirebilme yeteneğini korumak demektir.
:::

Bu not, Robert C. Martin'in *Clean Code* kitabındaki test anlayışını anlatan serinin ilk parçası. Testlerin nasıl yazılacağına geçmeden önce neden bu kadar özen istediklerini açıklıyor. Kitabın bazı bölümlerini Martin'in çalışma arkadaşları yazmıştır; serideki notlar o bölümlerin yazarını ayrıca belirtir.

## Atılan testten test paketine

Kitabın *Birim Testleri* bölümü, Martin'in doksanlarda yazdığı bir zamanlayıcıyla açılır. Martin kodu klavyeden elle deneyip çalıştığını görünce deneme kodunu atmıştı. Bugün aynı kodu işletim sisteminin saatinden yalıtacağını, zamanı testin içinden ilerleteceğini ve testleri kodla aynı depoya koyacağını söyler. Böylece kodla çalışan herkes testleri kolayca çalıştırabilir.

Aradaki fark, testin bir kerelik bir deneme olmaktan çıkıp kodla birlikte saklanan ve değişen bir test paketine dönüşmesidir. Bu notun geri kalanı, o paketin neden bakım istediğini anlatıyor.

## Testler kodla birlikte yazılır

*TDD'nin Üç Yasası* başlığı, Test Güdümlü Geliştirmenin (TDD) kurallarını şöyle verir:

1. Başarısız bir birim testi olmadan üretim kodu (production code) yazılmaz.
2. Test, başarısız olmaya yetecek kadar yazılır. Derlenmemek de başarısızlıktır.
3. Üretim kodu, yalnızca kırmızı testi yeşile çevirecek kadar yazılır.

Bu üç kural yarım dakikalık bir döngü kurar ve test her zaman kodun birkaç saniye önünde ilerler. Bu şekilde çalışan biri her gün onlarca, her yıl binlerce test yazar. Martin, boyca üretim koduyla yarışabilen bu test yığınının başlı başına zor bir yönetim sorunu olabileceğini söyler. Nedeni bir sonraki başlıkta ortaya çıkar: testler üretim koduyla birlikte değişmek zorundadır.

## Kirli test, hiç test olmamasından iyi değildir

*Testleri Temiz Tutmak* başlığı, Martin'in koçluk yaptığı bir ekibin hikâyesini anlatır. Ekip, test kodunu üretim kodunun kalite ölçütlerinden bilerek muaf tutmuştu. Değişken adlarının iyi seçilmesi, test fonksiyonlarının kısa ve açıklayıcı olması beklenmiyordu. Testin çalışması ve kodu kapsaması yeterli sayılıyordu.

Sorun, testlerin üretim koduyla birlikte değişmek zorunda olmasıydı. Hikâye şöyle ilerledi:

1. Üretim kodu değiştikçe eski testler kırıldı ve dağınık test kodu onları yeniden geçirmeyi zorlaştırdı. Pakete yeni test eklemek, yeni üretim kodunu yazmaktan uzun sürmeye başladı.
2. Test paketinin bakımı her sürümde pahalandı ve geliştiricilerin en büyük şikâyeti oldu. Tahminler büyüdükçe suç testlere yüklendi.
3. Ekip sonunda test paketini tamamen attı.
4. Testsiz kalınca bir değişikliğin başka bir yeri bozmadığını doğrulayamadılar. Hata oranı arttı.
5. Hatalar arttıkça değişiklik yapmaktan korktular ve kodu temizlemeyi bıraktılar. Kod çürüdü.

Martin'in vardığı sonuç açık: kirli test, hiç test olmamasına eşittir, hatta ondan kötü olabilir. Başarısızlığın kaynağı test yazmak değil, testlerin dağınık kalmasına izin vermekti. Bu yüzden test kodu da üretim kodu gibi düşünülerek tasarlanmalı ve aynı temizlikte tutulmalıdır.

## Testler değişimi mümkün kılar

*Testler -likleri Mümkün Kılar* başlığı, esneklik, bakım kolaylığı ve yeniden kullanılabilirlik gibi niteliklerin kaynağını testlerde görür. Testi olan geliştirici değişiklikten çekinmez. Testi olmayan için ise her değişiklik yeni bir hata riski taşır. Test yoksa, mimari ne kadar esnek olursa olsun, fark edilmeden hata sokma korkusu geliştiriciyi değişiklikten alıkoyar.

Testler varken durum tersine döner. İyi tasarlanmamış, karmaşık bir kod bile korkmadan değiştirilebilir, hatta tasarımı iyileştirilebilir. Kapsam arttıkça korku azalır.

Bu ilişki iki yönde işler. Testler kirlendikçe kodu değiştirme yeteneği de azalır ve kodun yapısını iyileştirmek zorlaşır. Bölümün *Sonuç* kısmı bir adım daha ileri gider: testler üretim kodunun esnekliğini, bakım kolaylığını ve yeniden kullanılabilirliğini korudukları için belki de ondan bile önemlidir. Bölüm aynı uyarıyla kapanır: testler çürürse kod da çürür.

## Test yazmak tasarımı iyileştirir

Kitabın Jeff Langr'ın yazdığı *Ortaya Çıkış* bölümü, Kent Beck'in dört basit tasarım kuralını önem sırasıyla verir: tüm testleri çalıştırır, tekrar içermez, yazanın niyetini ifade eder, sınıf ve metot sayısını en aza indirir. *Basit Tasarım Kuralı 1: Tüm Testleri Çalıştırır* başlığı bu sıralamanın gerekçesini anlatır.

Çalıştığı doğrulanamayan bir sistem, kâğıt üzerindeki tasarımı ne kadar iyi olursa olsun güven vermez. Langr'a göre böyle bir sistemi hiç kullanıma sunmamak savunulabilir bir tutumdur. Test edilebilir olma isteği de tasarımı kendiliğinden iyileştirir:

- Tek bir sorumluluğu olan küçük sınıflar kolay test edilir. Test yazdıkça sınıflar küçülür.
- Sıkı bağlı kod zor test edilir. Test yazdıkça bağımlılığı tersine çevirme (DIP), bağımlılık enjeksiyonu ve arayüz gibi bağlaşımı (coupling) azaltan araçlara yönelinir.

Kalan üç kural, *Basit Tasarım Kuralları 2–4: Yeniden Düzenleme* başlığında anlatıldığı gibi testlere dayanır. Kod her birkaç satırda bir gözden geçirilir ve gerekirse yeniden düzenlenir (refactoring). Ardından testler çalıştırılıp hiçbir şeyin bozulmadığı gösterilir. Test yoksa bu temizlik yapılamaz, çünkü kodu bozma korkusu onu engeller.

## Google bu konuda

*Software Engineering at Google* kitabı aynı dersi kendi deneyiminden anlatır.

- **Google Web Server hikâyesi** (*Testing Overview* bölümü): Google'ın ilk yıllarında mühendislerin kendi yazdığı testler pek önemsenmiyordu. Google Web Server ekibinde bir noktada canlıya çıkan sürümlerin yüzde 80'inden fazlası, kullanıcıyı etkileyen hatalar yüzünden geri alındı. Ekibin teknik lideri, her kod değişikliğinin testleriyle birlikte gelmesini ve testlerin sürekli çalışmasını şart koştu. Bir yıl içinde acil düzeltme sürümleri yarıya indi. Üstelik bu düşüş, projenin her çeyrekte rekor sayıda değişiklik aldığı bir dönemde yaşandı.
- **Bakımı zor testin iki belirtisi** (*Unit Testing* bölümü, *The Importance of Maintainability* başlığı): Google'ın bakımı zor test için verdiği iki belirti, Martin'in "kirli test" dediği şeye karşılık gelir. *Kırılgan test*, üretim kodundaki hiçbir hata getirmeyen, zararsız ve ilgisiz bir değişiklikte kırılan testtir. *Anlaşılmaz test* ise kırıldığında neyin yanlış gittiği, nasıl düzeltileceği ve testin neyi korumaya çalıştığı anlaşılamayan testtir.
- **Ayrıldığı nokta:** Google'ın test bölümleri önce test yazmayı şart koşmaz. Google'da her değişikliğin testleriyle birlikte gelmesi beklenir. Test kodunun hangi noktalarda üretim kodundan farklı olabileceğini iki kaynak da ayrıca tartışır. Bu konu serinin *Nelerden ödün verilir* notunun konusu.

## Bu projede yaşadığımız

7 Kasım 2026'da `main` dalında testleri çalıştırdığımızda üç testin, kodda hiçbir hata yokken kırıldığını gördük. Örnek veri modülünde iki ön koşulun yeri değiştirilmişti. Bu dosya hem IDE'nin açtığı deneme dosyasıydı hem de üç testin verisiydi. Testler ilk ön koşula sırası üzerinden, "birinci blok" diye erişiyordu. Sıra değişince birinci blokta başka bir metin vardı ve testlerin beklediği sonuç tutmadı. Kod doğru çalışıyordu, eskiyen şey testlerin beklentisiydi.

Bu durum kırılgan test tanımına yakındı: hata getirmeyen ve testlerin amacıyla ilgisi olmayan bir sıra değişikliği üç testi kırmıştı. Fark, değişenin üretim kodu değil testlerin verisi olmasıydı. Testler deneme dosyasına bağlı kaldıkça bu dosyadaki her düzenleme onları yeniden kırabilirdi. Böyle kırılmalar sıklaştıkça onları onarmanın maliyeti de büyür. Martin'in anlattığı ekip de testleri onarmanın maliyeti büyüdükçe paketi bütünüyle atmıştı.

Aynı günkü çalıştırmada dördüncü bir testin yalnızca macOS'ta kırık olduğu görüldü. O hikâye testlerin her ortamda aynı sonucu vermesiyle ilgili olduğu için serinin *F.I.R.S.T.* notunda anlatılıyor. Olay o gün ayrıca kayda geçirilmişti.
