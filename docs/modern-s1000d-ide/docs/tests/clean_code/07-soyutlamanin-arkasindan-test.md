---
title: "Soyutlamanın arkasından test"
ikon: 🎭
kapak: blue
tarih: 2026-11-07
etiketler: [clean-code, testler]
ozet: Değişken bir ayrıntıya doğrudan bağlanan kod zor test edilir; o ayrıntı küçük bir soyutlamanın arkasına alınır, dışarıdan verilir ve testte yerine sabit davranan bir uygulama konur. Ama hızlı ve belirlenimci bir gerçek uygulama varsa test onu kullanır.
---

:::callout 🎭 blue
**Değişken bir ayrıntıya doğrudan bağlanan kod zor test edilir.** O ayrıntıyı küçük bir soyutlamanın arkasına al, kodun içinde kurma, dışarıdan ver. Testte soyutlamanın yerine sabit davranan bir uygulama konabilir. Ama her şeyi soyutlamak gerekmez: hızlı ve her seferinde aynı sonucu veren bir gerçek uygulama varsa test onu kullanır. Soyutlama gerçek bir ihtiyaç için kurulur, kural olsun diye değil.
:::

Bu not serinin yedinci parçası. *Sınırlar ve learning test* notu, kontrolümüz dışındaki kodu bir sınırın arkasına almayı anlatıyordu. Bu not aynı fikri kendi kodumuzun içine taşıyor: bir parçayı test edebilmek için bağımlılıklarını nasıl ayırırız? Kaynakları kitabın dört bölümü: Martin'in Jeff Langr'la birlikte yazdığı *Sınıflar*, Kevin Dean Wampler'ın yazdığı *Sistemler*, Langr'ın yazdığı *Ortaya Çıkış* ve Martin'in yazdığı *Nesneler ve Veri Yapıları*.

## Somut ayrıntıya bağlı kod zor test edilir

*Sınıflar* bölümündeki *Değişimden Yalıtma* başlığının çıkış noktası şudur: ihtiyaçlar değiştikçe kod da değişir ve bir ayrıntıya doğrudan bağlanan kod o ayrıntıyla birlikte sarsılır. Arada bir arayüz ya da soyut sınıf olması bu sarsıntıyı sınırlar.

Bölüm bunun test tarafını bir yatırım portföyü örneğiyle anlatır. Portföyün değerini hesaplayan `Portfolio` sınıfı fiyatları doğrudan Tokyo borsasından çekiyorsa, testi de piyasayla birlikte oynar. Sonucu dakikadan dakikaya değişen bir hesap için test yazılamaz.

Çözüm üç adımdır:

1. Tek bir metodu olan bir `StockExchange` arayüzü tanımlanır: bir sembolün güncel fiyatını sormak.
2. Tokyo borsasına bağlanan sınıf bu arayüzü uygular.
3. `Portfolio` borsayı kendisi bulmaz, yapıcısında bir `StockExchange` alır.

Test artık gerçek borsa yerine kendi küçük uygulamasını verir. Bu uygulama her sembol için sabit bir fiyat döndüren basit bir tablodur, örneğin Microsoft için hep 100 dolar. Test, beş hisselik bir portföyün 500 dolar ettiğini bekler ve her çalıştırmada aynı sonucu alır.

Bölümün bundan çıkardığı ders iki yönlüdür. Böyle test edilebilecek kadar gevşek bağlı bir sistem, aynı zamanda daha esnek ve yeniden kullanılabilir olur. Tasarım da bağımlılığı tersine çevirme ilkesine (DIP) uyar: `Portfolio` artık fiyatın nereden geldiğini bilmez, yalnızca "fiyat sor" kavramına dayanır.

## Kurmak ile kullanmak ayrı işlerdir

*Sistemler* bölümündeki *Bir Sistemin Oluşturulmasını, Kullanılmasından Ayırmak* başlığı, Wampler'ın penceresinden gördüğü bir otel inşaatıyla açılır. İnşaat halindeki bina ile bitmiş otel, araçlarıyla ve içindeki insanlarla bambaşka yerlerdir. Wampler'a göre yazılımda da nesnelerin kurulup birbirine bağlandığı başlangıç, programın çalışma mantığından ayrı tutulmalıdır.

Bu iki iş çoğu uygulamada iç içe geçer. Wampler'ın örneği tembel kurulumdur (lazy initialization): bir nesne, ilk gerektiği anda onu kullanan metodun içinde kurulur. Bu, gereksiz kurulumu önler. Ama kullanan kodu kurduğu somut sınıfa ve onun bütün bağımlılıklarına zincirler. Kurulan nesne ağırsa, birim testinde metot çağrılmadan önce yerine bir test ikizi yerleştirmek gerekir. Metot hem kurduğu hem kullandığı için iki iş yapar ve kurma dalının da ayrıca test edilmesi gerekir. Üstelik kullanan sınıf, her durumda hangi nesnenin doğru olduğuna kendisi karar vermek zorunda kalır.

Bu küçük kurma alışkanlıkları çoğaldıkça, kurma stratejisi uygulamanın her yerine dağılır. Wampler'ın önerdiği çözümler şunlar:

- **Kurmayı main'e taşı.** *Main'in Ayrılması* başlığına göre nesneler main'de ya da onun çağırdığı modüllerde kurulur ve uygulamaya hazır verilir. Bağımlılık okları hep main'den uygulamaya doğru bakar. Uygulama nesnelerin nasıl kurulduğunu bilmez.
- **Ne zaman kurulacağına uygulama karar veriyorsa fabrika kullan.** *Fabrikalar* başlığına göre, örneğin bir sipariş sistemi sipariş satırlarını kendisi oluşturmalıysa, soyut bir fabrika zamanlamayı uygulamaya bırakır ama kurma ayrıntılarını main tarafında tutar.
- **Bağımlılıkları dışarıdan ver.** *Bağımlılık Enjeksiyonu* başlığına göre bir nesne bağımlılıklarını kendisi kurmaz, bu sorumluluğu yetkili bir mekanizmaya devreder. Bu mekanizma genellikle main ya da bu iş için yazılmış bir kaptır (container). Gerçek bağımlılık enjeksiyonunda sınıf tamamen edilgendir: bağımlılıklarını yapıcı argümanlarından ya da ayarlayıcı metotlardan alır.

Bizce test açısından en önemli sonuç şu: bağımlılığını dışarıdan alan bir sınıf, testte gerçek bağımlılık yerine bir ikizle kolayca kurulabilir. Tembel kurulumda ise ikizi yerleştirmek için sınıfın iç düzenine uymak gerekir.

## Her sınıfa arayüz gerekmez

Langr'ın *Ortaya Çıkış* bölümündeki *En Az Sınıf ve Metot* başlığı bir fren koyar. Sınıfları ve metotları küçültme çabası da aşırıya kaçabilir ve sistemi minik parçalara boğabilir. Bazen bunun arkasında, her sınıfa bir arayüz isteyen bir kodlama standardı gibi dogmalar vardır. Langr bu durumda kurala değil işe yarayana bakmayı önerir. Ama sınıf sayısını düşük tutmak, basit tasarımın dört kuralı içinde en düşük önceliğe sahiptir. Testler, tekrarsızlık ve açık niyet ondan önce gelir.

Bizce `Portfolio` örneğiyle bu kural birlikte okunmalıdır. Soyutlama, borsa gibi değişken ya da yavaş bir ayrıntı için kurulur. Her sınıfın önüne bir arayüz koymak test edilebilirliği artırmaz, yalnızca sınıf sayısını artırır.

## Nesne mi, veri yapısı mı?

Martin'in *Nesneler ve Veri Yapıları* bölümündeki *Veri/Nesne Karşı Simetrisi* başlığı, soyutlamanın neyi kolaylaştırıp neyi zorlaştırdığını gösterir. Nesne verisini saklar ve davranış sunar. Veri yapısı verisini açar ve davranış taşımaz. Martin'e göre bu iki tanım neredeyse birbirinin zıddıdır.

Kitabın şekil örneği bu farkın sonuçlarını gösterir:

| Yaklaşım | Yeni bir işlem eklemek (ör. çevre hesabı) | Yeni bir tür eklemek (ör. yeni bir şekil) |
|---|---|---|
| Veri yapıları ve onlar üzerinde çalışan fonksiyonlar | Kolay: şekillere dokunulmaz | Zor: her fonksiyon değişir |
| Nesneler ve çok biçimli metotlar | Zor: her şekil sınıfı değişir | Kolay: var olan işlemlere dokunulmaz |

Martin'e göre ikisinden biri her zaman doğru değildir. Sık sık yeni tür eklenecekse nesneler, sık sık yeni işlem eklenecekse veri yapıları uygundur. Her şeyin nesne olduğu fikri bir efsanedir. *Melez Yapılar* başlığı ise yarı nesne yarı veri yapısı olan tasarımlara karşı uyarır. Bunlar hem yeni işlem hem de yeni tür eklemeyi zorlaştırır.

Bizce bunun teste bir etkisi vardır. Testte bir bağımlılığın yerine sahtesini koymak, aslında o soyutlamaya yeni bir tür eklemektir. Bu yüzden yerine sahtesi konacak şeyin arkasında bir arayüz olması doğaldır. Türleri baştan belli, kapalı bir kümede ise sahte bir tür eklenmez. Orada testler her işlemi her tür için sınar.

## Google bu konuda

*Software Engineering at Google* kitabının *Test Doubles* bölümü, test ikizlerini Clean Code'dan çok daha ayrıntılı ele alır.

- **Bir tercih sırası vardır.** Önce gerçek uygulama kullanılır. O olmuyorsa sahte uygulama (fake) gelir ve Google bunu çoğu zaman en iyi seçenek sayar. Bir fonksiyonun ne döndüreceğini testte sabitlemek (stub) ve fonksiyonun nasıl çağrıldığını doğrulamak (etkileşim testi) en sona kalır.
- **Gerçek uygulama ilk tercihtir.** Gerçek uygulama hızlı, belirlenimci ve bağımlılıkları kolay kurulan olduğu sürece ilk seçenektir. Para, tarih ve koleksiyon gibi değer nesneleri en açık örnektir. Gerçek kodun testte çalışması, ikizlerle yalıtılmış bir testin kaçırabileceği hataları yakalar. Gerçeği öne alan bu yaklaşım klasik test, mock çerçevelerini öne alan yaklaşım ise mockist test olarak bilinir. Google mockist tarzı ölçeklemenin zor olduğunu, klasik tarzı tercih ettiğini söyler.
- **Hız ve belirlenimcilik ölçüttür.** Kesin bir eşik yoktur. Test başına bir milisaniye yavaş sayılmaz, ama beş yüz testlik bir pakette test başına bir saniye fazla olabilir. Sınırda kalan durumlarda gerçek uygulamayla başlanır, yavaşlayınca ikize geçilir. Dış bir servise bağlı bir test hermetik değildir. Saat gibi bağımlılıklar için sabit zaman döndüren bir ikiz kullanılır.
- **Test edilebilirlik baştan kurulur.** Kodun testte üretimdekinden farklı bir bağımlılıkla çalışabildiği yere dikiş yeri (seam) denir. Bunu kurmanın yaygın yolu bağımlılık enjeksiyonudur. Testsiz yazılmış koda sonradan test edilebilirlik kazandırmak büyük emek isteyebilir. Google'a göre ideal olan, testte de üretimdeki kurma kodunu (fabrikayı ya da bağımlılık enjeksiyonunu) kullanmaktır. Bunun için o kurma kodu ikizleri kabul edecek kadar esnek olmalıdır. Bu, Wampler'ın kurmayı tek bir yerde toplama önerisiyle aynı yöne bakar.
- **Stub'ın tehlikeleri.** Stub yazmak kolaydır, ama aşırı kullanıldığında üç sorun doğurur:
  - Test anlaşılmaz olur: neden o fonksiyonların sabitlendiğini anlamak için sistemi zihinde çalıştırmak gerekir.
  - Test kırılgan olur: üretim kodunun iç ayrıntıları teste sızar.
  - Test daha az güvence verir: sabitlenen davranışın gerçek sözleşmeye uyduğu garanti değildir.

  Stub, sistemi zor oluşturulan belirli bir duruma sokmak için uygundur, örneğin boş olmayan bir liste ya da bir hata döndürmek. Her stub bir doğrulamayla ilişkili olmalıdır. Stub'ın uygun olduğu durumda bile Google gerçek uygulamayı ya da sahtesini yeğler.
- **Etkileşimi değil durumu test et.** Bir fonksiyonun çağrıldığını doğrulayan etkileşim testi, yalnızca çağrının yapıldığını gösterir, işin doğru yapıldığını değil. İç ayrıntılara bağlı olduğu için de kırılgandır. Etkileşim testi iki durumda uygundur: gerçek uygulama çok yavaş olduğu ve sahte uygulama bulunmadığı için durumu sınamak imkânsızsa, ya da çağrıların sayısı ve sırası davranışın kendisiyse. Örneğin bir önbelleğin veritabanı okumalarını azaltıp azaltmadığı. Genel kural olarak yalnızca durumu değiştiren çağrılar (e-posta göndermek, kayıt yazmak gibi) doğrulanır, yalnızca veri döndüren çağrılar doğrulanmaz.
- **Mock çerçevelerinin bedeli.** Google'da mock çerçeveleri başta her çiviye uyan bir çekiç gibi görünmüştü. Yıllar sonra bunların sürekli bakım isteyen ama nadiren hata yakalayan testler ürettiği görüldü ve eğilim daha gerçekçi testlere döndü. Google Testing Blog'daki *Don't Overuse Mocks* (2013) yazısı aynı dersi bir ödeme örneğiyle verir. Beş çağrıyı mock'layıp tek bir çağrıyı doğrulayan bir test yerine, ödemeyi yapıp sunucudaki son çekimin 500 dolar olduğunu denetleyen bir test bazen hem daha basit hem daha yararlı olabilir.

**İki kaynağın buluştuğu ve ayrıldığı yer:** Bizce `Portfolio` örneğindeki sabit fiyatlı borsa, Google'ın stub için uygun gördüğü kullanıma yakındır: sistemi belirli bir duruma sokar. Google yine de önce bir sahte uygulama arardı. Langr'ın her sınıfa arayüz dogmasına karşı çıkması da Google'ın gerçek uygulamayı öne almasıyla aynı sonuca varır, ama gerekçeleri farklıdır: Langr sınıf sayısına, Google testin gerçeğe yakınlığına bakar. Asıl ayrılık şu: Clean Code test ikizlerinin türlerini ayırt etmez ve mock'a karşı bir uyarı yapmaz. Google türleri ayırır, bir tercih sırası koyar ve mock'un aşırı kullanımına karşı açıkça uyarır.

## Bu projede yaşadığımız

**Hiç sahte uygulama yoktu.** 7 Kasım 2026'da projede tek bir arayüz (trait), sahte uygulama ya da mock yoktu. Testler gerçek parçaları kullanıyordu. Typst'i gerçekten çalıştırıyor, önizleme testlerinde bunun için çok küçük bir sayfa şablonu veriyordu. Dosya sistemine geçici klasörlerde gerçekten yazıyordu. Arayüz kütüphanesini ise onun test kipinde, pencere göstermeden çalıştırıyordu. Bu, klasik denen tarzdı: gerçek uygulama yeterince hızlı ve belirlenimci olduğu sürece ilk tercihti. Gerçeği kullanılmayanlar saat ile dosya ve klasör seçme pencereleriydi. İkisini de arayüz kütüphanesinin test platformu taklit ediyordu: saat elle ilerletiliyor, pencerelerin kullanıcıya soracağı yanıt testte önceden veriliyordu.

**Kök nesne giriş dosyasında kuruluyordu.** Aynı tarihte programın giriş dosyası kendini, parçaları birbirine bağlayıp pencereyi açan dosya olarak tanıtıyordu. Uygulamanın ana nesnesi orada, örnek CSDB klasörüyle kuruluyordu. Paneller ise ana nesnenin yapıcısında kuruluyordu ve aynı nesne çalışma mantığını da, klasör seçmeyi de taşıyordu. Bizce bu, Wampler'ın kurmayı main'e taşıma önerisine kısmen uyuyordu.

**Kullanılırken kurulan font deposu.** Typst'e verilen font deposu ilk gerektiği anda kuruluyor ve program boyunca tek bir ortak değer olarak yaşıyordu. İçine önce makinedeki sistem fontları, sonra Typst'in kendi fontları ekleniyordu. Bizce bu, Wampler'ın anlattığı tembel kurulumun daha katı bir biçimiydi: Wampler'ın örneğinde testte bir ikiz yerleştirilebiliyordu, buradaki ortak değer ise testte hiç değiştirilemiyordu. *F.I.R.S.T.* notunda anlatılan risk, sistem fontlarının taranmasından ve örnek şablonun makinede kurulu olmayabilecek bir font istemesinden doğuyordu. Bizce depo dışarıdan verilseydi testler sistem fontlarını taramayan sabit bir depo geçebilirdi.

**Blok türleri sabit bir listeydi.** Belgenin blok türleri sabit bir liste olarak tanımlanmıştı ve bu listeyi belge dizini ile blok editörü ele alıyordu. Önizleme ve PDF bu türleri kullanmıyordu, XML'i Typst şablonu doğrudan okuyordu. Bizce bu, Martin'in karşı simetrisinde veri yapısı tarafına uyan bir seçimdi: editöre yeni bir işlem eklemek kolaydı, yeni bir blok türü eklemek ise dizindeki ve editördeki her eşlemeyi değiştirmeyi gerektirirdi.

## Kitabın söylemedikleri

- Test ikizlerinin türleri arasında ayrım yapılmıyor. Kitap test ikizleri için Meszaros'un *xUnit Test Patterns* kitabına gönderme yapar, ama "test uygulaması", "test ikizi", "mock nesne" ve "sahte" sözcüklerini gevşek kullanır.
- Gerçek uygulamanın ne zaman bir test ikizine tercih edileceği söylenmiyor. Kitaptaki tek ipucu, kurulan nesnenin ağır olmasıdır.
- Mock çerçevelerinin bedelinden söz edilmiyor.
- Sahte bir uygulamanın gerçeğine sadık tutulması anlatılmıyor. Bu konu *Sınırlar ve learning test* notunda Google'ın sözleşme testleriyle ele alınıyor.
- Veri/nesne karşı simetrisinin teste etkisi anlatılmıyor.
