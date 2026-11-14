---
title: "Mimariyi testle sürmek"
ikon: 🏗️
kapak: brown
tarih: 2026-11-07
etiketler: [clean-code, testler, mimari]
ozet: Mimari baştan büyük tasarlanmaz; kaygılar ayrıldığı ve her adımı otomatik testler koruduğu sürece bugünün ihtiyacı kadar kurulur, değişim geldikçe küçük adımlarla büyütülür ve kararlar en iyi bilginin elde olduğu ana ertelenir.
---

:::callout 🏗️ brown
**Mimari baştan büyük tasarlanmaz, testlerle korunarak küçük adımlarla büyütülür.** Bugünün ihtiyacı kadarı kurulur. Yeni bir ihtiyaç geldiğinde kod yeniden düzenlenir ve genişletilir. Bunu güvenli kılan iki şey vardır: kaygıların birbirinden ayrılmış olması ve her adımdan sonra sistemin hâlâ çalıştığını gösteren otomatik testler. Kararlar da en iyi bilginin elde olduğu ana kadar ertelenir.
:::

Bu not serinin sekizinci ve son parçası. Önceki notlar tek tek testleri ve bağımlılıkları ele alıyordu. Bu not ölçeği büyütüyor: testler bir sistemin mimarisinin zaman içinde büyümesine nasıl izin verir? Kaynakları kitabın beş bölümü: Kevin Dean Wampler'ın *Sistemler* bölümü, Martin ile Jeff Langr'ın *Sınıflar* bölümü, Martin'in *Ardışık İyileştirme* ve *SerialDate'i Yeniden Düzenleme* bölümleri ve Langr'ın *Ortaya Çıkış* bölümü.

## Sistemler şehirler gibi büyür

Wampler *Sistemler* bölümünü bir şehir benzetmesi üzerine kurar. Bir şehri tek kişi yönetemez. Şehir, her biri bir işten sorumlu ekiplerle ve bu ekiplerin bütünü bilmeden çalışmasına izin veren katmanlarla ayakta durur. Wampler'ın sorusu, yazılım sistemlerinin de bu ayrımı taşıyıp taşımadığıdır.

*Ölçeği Büyütmek* başlığında benzetme büyümeye uzanır. Hiçbir kasaba, bir gün büyüyeceğini umarak ortasından altı şeritli bir yol geçirmez. Yollar ve altyapı nüfus arttıkça genişler. Bu sancılıdır ama kaçınılmazdır. Wampler buradan yazılım için şu sonucu çıkarır: sistemi ilk seferde doğru kurmak mümkün değildir. Bugünün hikâyeleri yazılır, yarının hikâyeleri için sistem yeniden düzenlenir. Kod düzeyinde bunu TDD ve yeniden düzenleme sağlar. Mimari düzeyde de aynı büyüme mümkündür, çünkü yazılım bir binanın betonu gibi donmaz. Tek koşul, kaygıların birbirinden ayrılmış olmasıdır.

Wampler'ın karşı örneği EJB2'dir. İş mantığı ağır EJB2 konteynerine sıkıca bağlıydı. Yalıtarak test etmek için ya konteyneri taklit etmek ya da testleri gerçek bir sunucuya dağıtmak gerekiyordu. Aynı bağ, kodun o mimarinin dışında kullanılmasını da engelliyordu.

## Mimariyi testle sürmek

*Sistem Mimarisini Test Güdümlü Geliştirin* başlığına göre iş mantığı, mimariyle ilgili kaygılardan arındırılmış düz nesnelerle yazılırsa mimari de kod gibi testle büyütülebilir. Yeni teknolojiler ihtiyaç doğdukça eklenir.

Wampler bu yüzden her şeyi baştan tasarlamayı (Big Design Up Front) zararlı bulur. Emek verilmiş bir tasarımdan vazgeçmek zordur ve erken seçilen mimari sonraki düşünceyi kendi kalıbına sokar. Bir dipnotta, bunun iyi bir alışkanlık olan önden tasarımla karıştırılmaması gerektiğini de belirtir. Bina mimarı inşaat başladıktan sonra iskeleti değiştiremez. Yazılımda ise kaygılar ayrıldıysa köklü değişiklikler bile karşılanabilir.

Wampler'ın önerdiği başlangıç, saf denecek kadar basit ama parçaları iyi ayrılmış bir mimaridir. Hikâyeler hızla teslim edilir, altyapı sistem büyüdükçe eklenir. Bu plansız yola çıkmak anlamına gelmez: kapsam, hedef ve genel yapı hakkında bir beklenti vardır, ama rota değiştirme yeteneği korunur. Bölümün son sözü de ölçekten bağımsızdır: ne sistemde ne modülde gereğinden karmaşık bir çözüme uzanılmalıdır.

## Kararları ertele

*Karar Vermeyi Optimize Edin* başlığında Wampler kararların zamanlamasına odaklanır. Karar ne kadar öne çekilirse onu besleyecek kullanıcı geri bildirimi, düşünme süresi ve uygulama deneyimi o kadar azalır. Bu yüzden kararı mümkün olan son ana bırakmak tembellik değil, daha iyi bilgiyle seçmektir. Kaygıları ayrılmış bir sistem bu ertelemeye izin verir. Modülerlik, kararların tek bir kişide toplanmadan dağıtılmasını da sağlar.

*Standartları Akıllıca Kullanın, Kanıtlanabilir Değer Kattıklarında* başlığı standartları reddetmez. Fikirleri ve bileşenleri yeniden kullanmayı, deneyimli insan bulmayı kolaylaştırdıklarını kabul eder. Ama yalnızca gösterilebilir bir değer kattıklarında seçilmelerini ister. Wampler'ın örneği, daha hafif bir tasarımın yeteceği yerde EJB2'yi sırf standart olduğu için seçen ekiplerdir.

## Değişim geldiğinde düzenle

Martin ile Langr'ın *Sınıflar* bölümündeki *Değişim İçin Düzenleme* başlığı aynı tutumu sınıf düzeyine indirir. Her değişiklik sistemin geri kalanını bozma riski taşıdığı için sınıflar bu riski küçültecek biçimde düzenlenir.

Örnek, SQL cümleleri üreten ve henüz güncelleme cümlesini desteklemeyen bir `Sql` sınıfıdır. Sınıfın iki ayrı değişme nedeni vardır: yeni bir cümle türünün eklenmesi ve var olan bir türün ayrıntısının değişmesi. Yani sınıf tek sorumluluk ilkesini çiğner. Yalnızca tek bir cümle türüne hizmet eden özel metotlar bunun ipucudur.

Bölüm yine de harekete geçmek için bu ipucunu yeterli saymaz, ölçüt değişimin kendisidir. Sınıf kendi içinde tamamsa ya da güncellemeye bir süre gerek yoksa ona dokunulmaz. Sınıfı değiştirmek için açmak gerektiği an tasarımı düzeltmek düşünülmelidir.

Düzeltilmiş tasarımda her cümle türü ayrı bir sınıftır, ortak yardımcılar da küçük sınıflara ayrılmıştır. Sınıflar birbirinden yalıtıldığı için her biri ayrı ayrı kolayca test edilir. Güncelleme desteği, var olan hiçbir sınıfa dokunmadan yeni bir sınıfla gelir. Bu, açık-kapalı ilkesidir (OCP): sistem değiştirilerek değil, genişletilerek büyür.

Bizce bu, Wampler'ın yol benzetmesinin sınıf ölçeğindeki karşılığıdır: yol, trafik gerçekten arttığında genişletilir.

## Küçük adımlar, her adımda çalışan sistem

Martin'in *Ardışık İyileştirme* bölümündeki *Kademecilik Üzerine* başlığı büyümenin güvenli yolunu anlatır. Martin'e göre bir programı bozmanın kestirme yolu, iyileştirme bahanesiyle yapıyı tek seferde baştan değiştirmektir. Ardından eski davranışa geri dönmek çok zorlaşır.

Martin'in çaresi TDD'nin bir kuralıdır: sistem hiçbir an bozuk bırakılmaz. Bunun ön koşulu, davranışın değişmediğini her an doğrulayabilen otomatik bir test paketidir. Martin, argüman ayrıştırıcısını hedeflediği yapıya çok sayıda küçük adımla taşır. Bir adım testleri kırınca, yeni bir adıma geçmeden önce o kırılmayı giderir.

*SerialDate'i Yeniden Düzenleme* bölümünde de aynı tutum görülür. Martin sınıfı yeniden düzenlemeden önce, mevcut testlerin kapsamının yetmediğini görür ve önce kendi testlerini yazar. Bu örnek *Kapsam ve uç durumlar* notunda anlatılıyor. *Neden temiz test?* notunda anlatılan basit tasarım kuralları da aynı yere varır: yeniden düzenleme ancak testler bozulmadığını gösterebildiği için güvenle yapılır.

## Google bu konuda

- **Test yazmak tasarımı sınar** (*Testing Overview* bölümü, *Benefits of Testing Code* başlığı): Yeni kod için test yazmak, API'yi kullanarak denemenin pratik bir yoludur. Kod zor test ediliyorsa, nedeni çoğu zaman tek bir parçada toplanmış çok fazla sorumluluk ya da yönetilmesi zor bağımlılıklardır.
- **Değişmeyen testler** (*Unit Testing* bölümü, *Strive for Unchanging Tests* başlığı): Saf bir yeniden düzenleme mevcut testleri değiştirmemelidir. Değiştiriyorsa ya düzenleme saf değildir ya da testler yanlış soyutlama düzeyindedir. Bizce mimarinin testlerle güvenle büyüyebilmesinin ön koşulu budur: yapı değişirken davranışı anlatan testler yerinde kalmalıdır.
- **Küçük değişiklikler** (Google'ın kod incelemesi rehberi, *Small CLs*): Küçük değişiklikler daha hızlı ve daha dikkatli incelenir, hata getirme olasılıkları daha düşüktür, reddedilirlerse daha az emek boşa gider, birleştirilmeleri ve geri alınmaları daha kolaydır, iyi tasarlanmaları da daha kolaydır.
  - Doğru boyut, tek bir şeyi ele alan ve kendi başına anlaşılabilen bir değişikliktir. İlgili testler aynı değişikliğe girer.
  - Kesin bir sınır yoktur. Yüz satır genellikle makul, bin satır genellikle fazladır.
  - Yeniden düzenlemeler genellikle özellik ve hata düzeltmelerinden ayrı bir değişiklikte yapılır.
  - Saf bir yeniden düzenleme de testlerle korunmalıdır. Testi olmayan bir kod yeniden düzenlenecekse testler eklenmelidir. Bu testler önce ayrı bir değişiklikle gönderilebilir.
  - Her değişiklikten sonra sistem çalışmaya devam etmelidir. Büyük görünen bir işten önce, yolu açacak bir yeniden düzenleme değişikliği yapmak düşünülmelidir.

**İki kaynağın buluştuğu yer:** Martin'in kademeciliği ile Google'ın küçük değişiklik rehberi aynı şeyi söyler: yapı, her adımı testlerle korunan küçük değişikliklerle değiştirilir. İkisi de testi olmayan kodu yeniden düzenlemeden önce test ekler. Google buna değişikliğin boyutu ve incelenmesi açısından somut ölçüler ekler.

## Bu projede ve Rust rehberinde yaşadığımız

**Rust rehberindeki dört aşama.** 4 Kasım 2026'da rust-development rehberine eklenen bir örnek, bir kart listesini gösteren programın dört aşamada büyümesini anlatıyordu: tek dosya, modül, kütüphane ve ayrı paketler.

- **Modül:** birlikte değişen kodu aynı yerde toplamak. Rehberin kendi anlatımına göre bu ayırma, kart metni programın geri kalanından ayrı değişmeye başladığında işe yarayacak bir hazırlıktı.
- **Kütüphane:** ikinci bir programın aynı kodu çağırması gerekiyordu.
- **Ayrı paketler:** düz çıktı veren programın, önizlemeye özgü bir bağımlılığı almadan derlenebilmesi gerekiyordu.

Son iki aşamayı somut bir ihtiyaç başlatıyordu, modül aşaması ise o ihtiyaçtan önce yapılıyordu. Rehber ayrı paketleri baştan kurmuyordu. Bizce bu, Wampler'ın yol benzetmesinin ve "değişim geldiğinde düzenle" kuralının küçük ölçekli bir uygulamasıydı.

Davranışın korunması için rehber, başlangıçtaki çıktıyı kayda alıp her aşamada çıktının aynı kaldığını kontrol etmeyi öneriyordu. İlk iki aşamada bu karşılaştırma elle yapılıyordu. Otomatik test ancak kütüphane aşamasında geldi. Martin'in kademeciliği ise her adımda istendiği an çalıştırılabilen otomatik testler ister. Bizce rehberin ilk iki aşaması bu açıdan kitabın gerisinde kalıyordu. Elle yapılan karşılaştırma, *F.I.R.S.T.* notundaki "kendini doğrulayan test" kuralına da uymuyordu.

**Henüz gerekmeyen altyapıyı kaldırmak.** 7 Kasım 2026'da yazı tamamlama denemesi, blok editörüne bağlanana kadar projeden kaldırıldı ve yaklaşık 790 satır silindi. Bizce henüz bir kullanıcı hikâyesine hizmet etmeyen altyapıyı taşımamak, Wampler'ın "bugünün hikâyelerini uygula" önerisine uyuyordu. Bunun bir bedeli de oldu: denemeyle birlikte gelen öğrenme testleri de gitti. Bu, *Sınırlar ve learning test* notunda anlatılıyor.

**Ertelenen bir karar.** 7 Kasım 2026'da XSD doğrulaması için iki aday, LemMinX ve libxml, aynı resmî şemayla, aynı örnek modüllerle ve bilerek bozulmuş bir kopyayla denendi. İkisi de aynı şema hatalarını buldu. Bedelleri, lisansları ve hata konumunu gösterme biçimleri bir tabloda karşılaştırıldı. Araştırma libxml'i önerdi, ama karar verilmedi. Açık kalan sorular yazıldı: hangi aday seçilecek, hangi S1000D sürümü hedeflenecek, şemalar nasıl dağıtılacak. Bizce bu, Wampler'ın kararı en iyi bilgiyle verilebilecek son ana erteleme önerisinin bir örneğiydi. Yapılan deneme de o kararı besleyecek bilgiyi topluyordu.

**Tek paket, modüller.** 7 Kasım 2026'da proje tek bir paketti ve kodu kaygılara göre modüllere ayrılmıştı: CSDB okuma, belge modeli, sayfa dizgisi (Typst), blok editörü ve paneller. Ayrı bir kütüphane ya da ayrı paketler yoktu. Bizce bu boyutta modüller yeterliydi. Rust rehberindeki gibi ayrı bir kütüphane ya da paket gerektiren bir ihtiyaç, örneğin aynı kodu çağıracak ikinci bir program, henüz doğmamıştı.

## Kitabın söylemedikleri

- Bölüm kaygıları ayırmanın araçlarını anlatır (bağımlılık enjeksiyonu, vekil nesneler, görünüme yönelik programlama), ama kodun modül, kütüphane, paket ya da servise hangi sırayla bölüneceğini anlatmaz. Örnekleri Java ve EJB dünyasındandır.
- Bir değişikliğin ne kadar büyük olabileceğine dair bir ölçü verilmiyor. Google'ın küçük değişiklik rehberi bu boşluğu dolduruyor.
- "Mümkün olan son an"ın nasıl anlaşılacağı, yani bir kararın artık ertelenemeyeceği an tarif edilmiyor.
- Mimari değişirken hangi testlerin yerinde kalması gerektiği söylenmiyor. Google'ın değişmeyen testler kuralı bunu açıkça koyuyor.
