---
title: "Sınırlar ve learning test"
ikon: 🧭
kapak: purple
tarih: 2026-11-07
etiketler: [clean-code, testler]
ozet: Üçüncü taraf kod sistemin içinde dolaştırılmaz, küçük bir sınırın arkasına alınır; kütüphane öğrenme testleriyle (learning test) tanınır, bu testler yeni sürüm çıktığında yeniden çalıştırılır ve sınır, üretim kodunun kullandığı yoldan sınanır.
---

:::callout 🧭 purple
**Üçüncü taraf kodu sistemin içinde dolaştırma, onu küçük bir sınırın arkasına al ve o sınırı testlerle tanımla.** Bir kütüphaneyi üretim kodunda deneyerek değil, öğrenme testleriyle (learning test) tanı. Bu testler kütüphanenin yeni bir sürümü çıktığında yeniden çalıştırılır ve davranış değişikliğini hemen gösterir. Henüz var olmayan bir sistem için de istediğin arayüzü kendin tanımla, gerçek sistem gelince ona bir adaptörle bağlan.
:::

Bu not serinin altıncı parçası. Önceki notlar kendi kodumuzu nasıl test ettiğimizi anlatıyordu. Bu not, kontrolümüzde olmayan kodla buluştuğumuz yeri ele alıyor. Kaynağı, kitabın James Grenning'in yazdığı *Sınırlar* bölümü.

## Sınırdaki arayüzü dolaştırma

*Üçüncü Taraf Kod Kullanmak* başlığı bir gerilimle başlar. Kütüphane yazanlar ürünlerini olabildiğince çok kullanıcıya uyacak kadar geniş tutar. Kullananlar ise yalnızca kendi işlerine yarayan dar bir arayüz ister. Grenning'e göre bu gerilim sistemin sınırlarında soruna yol açabilir.

Örnek, Java'nın `Map` arayüzüdür. `Map` çok geniş bir arayüzdür ve bu genişlik bir yük de olabilir:

- Uygulama bir `Map`'i elden ele geçirdiğinde, onu alan sınıfların hiçbirinin içinden bir şey silmemesi istense bile her biri onu tümden temizleyebilir.
- Bir `Map`'e hangi türden nesnelerin konacağı güvenilir biçimde sınırlanamaz.
- Değeri her okuyan yer onu doğru türe kendisi dönüştürmek zorunda kalır. Java'ya generics gelince bu dönüştürme ortadan kalktı, ama `Map`'in gereğinden fazla yetenek sunması değişmedi.
- `Map` sistemin her yerine dağılmışsa arayüzü değiştiğinde düzeltilecek yer sayısı da çok olur. Grenning, generics geldiğinde gereken değişikliğin büyüklüğü yüzünden generics'e geçemeyen sistemler gördüklerini anlatır.

Çözüm, `Map`'i `Sensors` gibi küçük bir sınıfın içine almaktır. Sınırdaki arayüz böylece gizlenir ve uygulamanın geri kalanını çok az etkileyerek değişebilir. Sınıf yalnızca uygulamanın ihtiyacı olanı sunar, yanlış kullanılması zorlaşır ve iş kurallarını kendisi uygulayabilir.

Grenning'in derdi `Map`'in kendisi değil, sınırdaki bir türün sistemde elden ele dolaşmasıdır. Böyle bir tür, onu kullanan sınıfta ya da yakın sınıf ailesinde tutulur ve dışa açık API'lerde ne alınır ne döndürülür. Her `Map` kullanımını sarmalamak gerekmez.

## Öğrenme testleri

*Sınırları Keşfetmek ve Öğrenmek* başlığına göre başkasının kodunu doğrulamak bize düşmez, ama kullandığımız parçayı testle yoklamak işimize yarar. Yeni bir kütüphaneyi hem tanımaya hem de koda yerleştirmeye aynı anda girişmek zorluğu katlar. Sonuç çoğu zaman, hatanın kimin kodunda olduğunu bulmaya çalışılan uzun hata ayıklama günleridir.

Grenning bunun yerine, kütüphaneyi nasıl anladığımızı ayrı testlerle sınamayı önerir. Jim Newkirk bu testlere öğrenme testi (learning test) adını vermiştir. Bir öğrenme testi kütüphaneyi tam da uygulamada kullanacağımız biçimde çağırır, ama üretim kodunun dışında kalır. Böylece her test, API hakkındaki tek bir varsayımı sınayan küçük ve kontrollü bir deney olur.

*log4j'yi Öğrenmek* başlığı bu süreci bir günlük (log) kütüphanesiyle adım adım gösterir. İlk test konsola kısa bir ileti yazdırmayı bekler, kütüphane eksik bir bileşenden yakınır. Her adımda biraz daha belge okunur, yeni bir test denenir ve kütüphanenin tuhaf, belki hatalı sayılabilecek davranışları ortaya çıkar. Sonunda öğrenilenler birkaç basit testte toplanır. Bu bilgi uygulamanın kendi günlük sınıfına aktarılır ve uygulamanın geri kalanı log4j'nin arayüzünü hiç görmez.

## Öğrenme testleri bedavadan da iyidir

*Öğrenme Testleri Bedavadan da İyidir* başlığı bu testlerin hesabını çıkarır. API zaten öğrenilecekti, testler de bunu yalıtılmış ve kesin bir biçimde yapmayı sağladı. Yani ek bir maliyetleri yoktur. Asıl kazanç sonradan gelir. Kütüphanenin sahipleri kendi ihtiyaçları için kodu değiştirir, hata düzeltir, özellik ekler ve her sürüm bizim kodumuz için yeni bir risk taşır. Yeni bir sürüm çıktığında öğrenme testleri yeniden çalıştırılır ve beklentimize uymayan bir değişiklik hemen görünür.

Grenning'e göre öğrenecek bir şey kalmasa da sınırın, üretim kodunun yaptığı çağrıları aynen yineleyen testleri olmalıdır. Bunlara giden sınır testleri der. Bu testler olmazsa yeni sürüme geçişi kolaylaştıracak bir dayanak kalmaz ve eski sürümde gereğinden uzun kalmaya özenebiliriz.

## Henüz var olmayan kodu kullanmak

*Henüz Var Olmayan Kodu Kullanmak* başlığı başka bir sınırı ele alır: bilinenle bilinmeyen arasındaki sınırı. Grenning, bir telsiz haberleşme sistemi geliştiren ekibinden söz eder. Ekip, "verici" adlı bir alt sistem hakkında çok az şey biliyordu ve o alt sistemin arayüzü henüz tanımlanmamıştı. Ekip beklemek yerine işe, bilinmeyen kısımdan uzak bir yerden başladı.

Çalıştıkça sınırın öbür yanından ne istediklerini netleştirdiler ve istedikleri arayüzü kendileri tanımladılar: verilen frekansta, verilen veri akışını yayınlayan bir `Transmitter`. Bu arayüz onların kontrolündeydi, bu yüzden onu kullanan kod okunur ve amacına odaklı kaldı. Gerçek API sonradan tanımlanınca, kendi arayüzlerinden gerçek API'ye çeviren bir adaptör (ADAPTER) yazdılar. Adaptör, API ile konuşan kodu tek bir yerde topladı. API değiştiğinde değişecek yer de yalnızca orasıydı.

Bu tasarımın test için de bir getirisi vardı. Kendi arayüzleri, testte gerçek verici yerine sahte bir verici (`FakeTransmitter`) kullanmaya izin veriyordu. Gerçek API geldikten sonra da onu doğru kullandıklarını gösteren sınır testleri yazılabildi.

## Temiz sınırlar

*Temiz Sınırlar* başlığı bölümü toparlar. Sınırlar, değişikliğin en çok yaşandığı yerlerdir. Kontrolümüz dışındaki kodu kullanırken kendi emeğimizi korumak ve ileride değişikliğin pahalıya çıkmasını önlemek özen ister. Grenning'in önerileri şunlar:

- Sınırdaki kodu açıkça ayır ve ondan ne beklediğini testlerle yaz.
- Kodunun yalnızca birkaç yeri üçüncü taraf kodun ayrıntılarını bilsin.
- Kontrol edemediğin bir şeye değil, kontrol edebildiğin bir şeye dayan.
- Sınırı ya `Map` örneğindeki gibi sarmalayarak ya da kendi arayüzünden sağlanan arayüze çeviren bir adaptörle yönet.

## Öğrenme testleri nerede durur?

Kitap bu soruyu yanıtlamaz. Bizce öğrenme testleri, *Kapsam ve uç durumlar* notunda varılan ölçüte uyar: ayrı bir yeri, başka bir zamanda ya da başka bir ortamda çalışması gereken testler hak eder.

- Kütüphane programın içine derleniyor ve test için dışarıda bir kurulum gerekmiyorsa, öğrenme testleri her çalıştırmada koşabilir. O zaman "yeni sürümde yeniden çalıştır" kuralı kendiliğinden işler. Bu durumda öğrenme testleri, sınadıkları sınırın öteki testlerinin yanında durabilir.
- Test gerçek bir sunucuya ya da makinede kurulu bir araca ihtiyaç duyuyorsa ayrı bir yerde durması mantıklıdır. Yine de tek bir komutla çalıştırılabilmeli, eksik ortamı da açıkça söylemelidir.

## Google bu konuda

*Software Engineering at Google* kitabında "öğrenme testi" adıyla bir uygulama yok. Ama aynı soruna birkaç yönden yaklaşır.

- **Beyoncé kuralı** (*Testing Overview* bölümü): Bozulmasını istemediğin her şeyi test et. İlgisiz bir altyapı değişikliği testlerden geçip ürünü bozarsa, düzeltmek ve eksik testi eklemek ürünün sahibine düşer. Bizce bu, bir bağımlılıktan beklediğimiz davranışı kendi testimizle sabitlememiz gerektiği anlamına gelir.
- **Sürüm numarası bir tahmindir** (*Dependency Management* bölümü, *The Limitations of SemVer* başlığı): Anlamsal sürüm numarası, bakımcının yeni sürümün ne kadar uyumlu olduğuna dair tahminidir, bir kanıt değildir. "Güvenli" görünen bir yama sürümü de kullanıcıları bozabilir.
- **Hyrum Yasası** (*What Is Software Engineering?* bölümü): Bir API'nin yeterince kullanıcısı olduğunda, sözleşmede ne yazdığının önemi kalmaz. Sistemin gözlemlenebilen her davranışına birileri güvenir. Bizce öğrenme testlerinin küçük sürüm değişikliklerinde bile çalıştırılmasının nedeni budur.
- **Sahte uygulama yoksa sarmala** (*Test Doubles* bölümü, *What to Do If a Fake Is Not Available* başlığı): Bağımlı olunan bir API'nin sahte uygulaması (fake) yoksa önce sahiplerinden istenir. Olmazsa API'ye yapılan bütün çağrılar tek bir sınıfta toplanır ve o sınıfın sahtesi yazılır. Bu, Grenning'in adaptör ve sahte verici yaklaşımıyla aynı yapıdadır.
- **Sahte uygulamayı gerçeğin sahibi yazar** (aynı bölüm, *When Should Fakes Be Written?* başlığı): Sahte uygulamayı yazmak ve bakımını yapmak, gerçek uygulamanın sahibi olan ekibe düşer.
- **Sahte uygulama gerçeğe sadık tutulur** (aynı bölüm, *Fakes Should Be Tested* başlığı): Sahte uygulamanın da testleri olmalıdır. Google, API'nin dışa açık arayüzüne karşı yazılan ve hem gerçek uygulamada hem sahte uygulamada çalıştırılan sözleşme testlerini (contract test) önerir.
- **Birim testleri sürüm uyumunu yakalamaz** (*Larger Testing* bölümü): Google Maps API'sinin yeni bir sürümüne geçilmesini örnek verir. Uyum sorunlarında birim testlerinin büyük olasılıkla yardım etmeyeceğini, insanın programı çalıştırıp denemeye yöneleceğini söyler. Sahte uygulamaların zamanla gerçeğinden uzaklaşması da birim testlerinin kör noktalarından biri olarak sayılır.

**İki kaynağın buluştuğu yer:** Grenning kendi arayüzünü tanımlamayı, gerçek API'ye adaptörle bağlanmayı, testte sahte uygulama kullanmayı ve gerçek API için sınır testleri yazmayı önerir. Google aynı yapıyı kurar ve sahte uygulamanın gerçeğinden uzaklaşmaması için sözleşme testlerini ekler.

## Bu projede yaşadığımız

**Bir öğrenme testi baştan beri vardı.** 6 Kasım 2026'daki ilk sürümden beri CSDB'nin başlık okuyucusunda, kendini açıkça öğrenme testi olarak tanıtan bir test vardı. S1000D dosyaları çizimlerini satır içi bir DTD'de varlık olarak tanımlar. İlk sürümde XML kütüphanesi roxmltree'nin böyle bir DTD'yi varsayılan ayarıyla reddettiği yalnızca testin açıklamasında yazıyordu. Testin kendisi yalnızca bizim okuyucumuzun bu dosyayı kabul ettiğini denetliyordu. 7 Kasım 2026'nın ilk dakikalarında XML okuma kodu ayrı bir modüle alınırken teste, kütüphanenin reddettiğini denetleyen bir satır eklendi. O andan sonra test iki şeyi birlikte sabitliyordu. Kütüphane bir sürümde varsayılanını değiştirirse ilk denetim kırılacak ve davranış farkını hemen gösterecekti. Bu, Grenning'in öğrenme testiyle giden sınır testini tek bir testte birleştiren küçük bir örnekti.

**Öğrenme testleri özellikle birlikte gitti.** 7 Kasım 2026'nın ilk dakikalarında yazı tamamlama denemesiyle birlikte, arayüz kütüphanesi GPUI Kit'in tamamlama menüsünü nasıl açıp kapadığını sabitleyen üç öğrenme testi eklendi. Aynı gün öğlen deneme kaldırılınca bu testler de onunla birlikte silindi. Öğrenilen bilgi artık yalnızca git geçmişindeydi.

**Typst sınırın arkasındaydı, ama öğrenme testi yoktu.** 7 Kasım 2026'da Typst yalnızca dizgi modülünün iki dosyasında kullanılıyordu. Modül dışarıya kendi sayfa ve belge türlerini veriyor, Typst'in belge türünü kendi türünün içine alıyor, hatalarını da düz metne çeviriyordu. Uygulamanın geri kalanı Typst'i görmüyordu. Bu, kitaptaki `Sensors` örneğiyle aynı yaklaşımdı. Modülün testleri Typst'i üretim kodunun kullandığı yoldan çağırıyordu: örnek veri modülünü sayfalara döküyor, PDF'e çeviriyor, PDF'in sayfa sayısını önizlemeninkiyle karşılaştırıyordu. Bunlar Grenning'in giden sınır testleri dediği türdendi. Ama Typst'in tek bir davranışını "biz bunu böyle anladık" diye sabitleyen bir öğrenme testi yoktu. XML okuyucusu roxmltree de yalnızca belge modülünde ve CSDB modülünün XML okuyucusunda kullanılıyordu ve CSDB'deki XML yardımcıları modülün dışına açılmıyordu.

**Kütüphane davranışını kırılan bir testten öğrendik.** 7 Kasım 2026'da bir test yalnızca macOS'ta kırıldı. Nedeni testin geri almak için `ctrl-z`'ye basmasıydı. GPUI Kit ise geri almayı macOS'ta `cmd-z`'ye, öteki sistemlerde `ctrl-z`'ye bağlıyordu. Bu bilgiyi bir öğrenme testi baştan sabitleyebilirdi: geri almanın hangi sistemde hangi tuşa bağlı olduğu ve her sistemde doğru tuşa çevrilen `secondary-z` kısayolunun nasıl davrandığı. Aynı gün notları hazırlarken, örnek şablonun istediği Noto Sans fontunun Typst'in programla gelen fontları arasında olmadığını da fark ettik.

**Bilgi yorumlarda yaşıyordu.** 7 Kasım 2026'da projenin bağımlılık dosyasında arayüz kütüphanesinin ve Typst'in sürümleri tam olarak sabitlenmişti. Yanlarındaki yorumlar kütüphaneler hakkında öğrenilmiş bilgileri taşıyordu. Testler için kullanılan GPUI sürümünün, arayüz kütüphanesinin sabitlediği sürümle aynı kalması gerektiği yazıyordu, yoksa testler derlenmezdi. Arayüz kütüphanesinin test özelliğini açmanın, başka iki bağımlılığın sürümünü uygulama için de geriye çekeceği yazıyordu. İlkini derleyici yakalıyordu. İkincisi yalnızca yorumda yaşıyordu ve bir yükseltmede onu denetleyen bir test yoktu.

**Clean Code deposunda öğrenme testleri.** clean-code-philosophy deposunda öğrenme testleri ayrı bir klasörde toplanıyordu. 29 Eylül 2026'daki ilk testler SonarQube'un gerçek sunucusunu kullanıyordu. 4 Kasım 2026'da klasöre PMD'nin gerçek kurulumunu kullanan testler eklendi. PMD'nin sürümü test dosyasında sabitti ve "PMD yükseltilince bu testler yeniden koşulur" kuralı deponun yönergesinde yazılıydı. Ama yeniden çalıştırmayı tetikleyen bir düzenek yoktu, iş hatırlamaya kalıyordu. SonarQube'un sürümü ise depoda hiçbir yerde sabitlenmemişti. Sunucu adresi ve erişim anahtarının dosya yolu kodun içine sabit yazılmıştı. 7 Kasım 2026'da bu notlar için incelendiğinde, anahtar dosyası olmayan bir makinede anahtarı okuyan testler, eksik ortamı anlatan bir ileti yerine ham bir `FileNotFoundError` ile kırıldı.

## Kitabın söylemedikleri

- Öğrenme testlerinin nerede tutulacağı ve her çalıştırmada mı koşacağı söylenmiyor. Kitap yalnızca yeni sürüm çıktığında çalıştırılmalarından söz ediyor.
- Yeniden çalıştırmanın nasıl tetikleneceği anlatılmıyor.
- Bir özellik kaldırıldığında onunla birlikte gelen öğrenme testlerinin ne olacağı anlatılmıyor.
- Sahte uygulamanın gerçeğine sadık kalmasının nasıl sağlanacağı anlatılmıyor. Google'ın sözleşme testi önerisi bu boşluğu dolduruyor.
- Bir arayüz çatısı gibi kodun her yerine yayılan üçüncü taraf kodun nasıl sınırlanacağı anlatılmıyor. Bölümün örnekleri bir veri yapısı, bir günlük kütüphanesi ve henüz yazılmamış bir alt sistem.
