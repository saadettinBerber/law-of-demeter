# Clean Code’u Rust’ta uygulamak

Clean Code notlarındaki dört basit tasarım kuralı bir öncelik sırası veriyor.
**Testler geçsin, aynı bilgi iki kez yaşamasın, niyet kodda görülsün, yeterli
olan en küçük tasarım kalsın.** Rust bu sırayı değiştirmez. Bazı kararları
`enum`, görünürlük ve sahiplik aracılığıyla derleyicinin denetleyebileceği
hâle getirir. Davranışın kullanıcı için doğru olup olmadığına test ve inceleme
ile karar veririz. [Clean Code zihin haritası · Bl. 9–12](https://github.com/saadettinBerber/clean-code-philosophy/blob/5321af5/zihin-haritasi.md)

## Dört kural bir değişiklikte nasıl çalışır?

Kart oturumu, kullanıcının cevabı açmasını ve kendi yanıtını değerlendirmesini
modelleyen küçük bir Rust örneği. `Review` bu sırayı yönetiyor.
[Kart oturumunun kodu ve testleri](kart-oturumu.md)

| Sıra | Ne arıyoruz? | Kart oturumunda karşılığı |
| --- | --- | --- |
| 1 · Testler geçer | Bir sonraki değişiklikte hangi davranışın korunacağını bilmek. | Gizli cevabı değerlendirme reddedilir. Cevap açılınca aynı oturum değerlendirilebilir. Test yalnız hatayı değil, hatadan sonra devam edilebildiğini de görür. |
| 2 · Tekrar azalır | Aynı kuralın iki yerde ayrı ayrı değişmesi gerekmesin. | Değerlendirme sırası `Review::grade` içinde yaşar. Bir arayüz veya komut satırı aracı eklenirse değerlendirmeyi bu metottan yapabilir. |
| 3 · Niyet okunur | Bir çağrıyı okuyan kişi etkisini ve argümanın anlamını anlayabilsin. | `grade(Outcome::Known)` seçimi açıkça taşır. `reveal_answer` okuma ile birlikte bir durum geçişi olduğunu söyler. |
| 4 · Tasarım yeterli kalır | Her tür ve soyutlama gözlenen bir ihtiyacı karşılasın. | Üç durumu tek özel `enum` ifade eder. Durumların her biri için ayrı trait ve tür hiyerarşisi kurmaya gerek kalmaz. |

Dördüncü kural ilk üçüne hizmet eder. İki testte benzer kurulum varsa ortak
bir yardımcı, testin asıl davranışını daha okunur yapabilir. Bir yardımcı
çağrısının ardına gizlenen üç farklı koşulu anlamak için dosyada gezinmek
gerekiyorsa kurulumun testte görünür kalması daha açık olur. Aynı şekilde iki
`match` ifadesinin benzemesi tek başına tekrar sayılmaz. Biri durum geçişini,
öteki mevcut sonucu okumayı tanımlar. Birleştirmeden önce aynı sebeple değişip
değişmediklerine bak. [Tamam tanımı · Bl. 9 ve 12](https://github.com/saadettinBerber/clean-code-philosophy/blob/5321af5/tamam-tanimi.md)

## Sorumluluk, sınır ve değişim

**Tek sorumluluk ilkesi.** Bir modülün hangi nedenle değiştiği,
sorumluluğunu gösterir. `Review` bir kart çalışma denemesinin sırasını yönetir.
Kullanıcının beklenen davranışı değiştiğinde bu tür değişir. SQLite şeması,
HTTP durum kodu veya ekrandaki metin değiştiğinde ilgili sınır değişir. Bir
metodun uzunluğu inceleme sinyalidir. Asıl soru, tek bir ürün kararı değiştiğinde
hangi modülleri açmak zorunda kalacağımızdır. Bir fonksiyonu üçe bölmek tek
başına bu bağımlılığı çözmez.

**Açık-kapalı ilkesi.** Yeni davranış eklerken değişikliği ilgili genişleme
noktasında tut. Sorunun özü yeni bir *durum* eklemekse, özel
bir `enum` ve kapsamlı `match` etkilenen dalları görünür kılar. Derleyici
işlenmeyen yeni varyantı gösterir. Yeni *kayıt yöntemi* geliyorsa kart
durumlarına yeni bir varyant eklemek anlamsız olur. Genişleme noktası o zaman
depolama sınırıdır. Hangi eksende değişim beklediğini somut bir istekle
belirle, ardından tasarımı o eksende aç.

**Bağımlılıkların tersine çevrilmesi ilkesi ve bağımlılık enjeksiyonu.**
İş kuralı, belirli bir veritabanı yerine ihtiyaç duyduğu davranışın
sözleşmesine dayanır. Bağımlılık enjeksiyonu, bu sözleşmeyi karşılayan
uygulamayı iş kuralına dışarıdan verir. Bir hesaplama yalnızca tarih istiyorsa
tarihi değer olarak geçirmek yeterli olabilir. Birden çok işlem sırasında saat
okunacaksa veya gerçek depoya bağlanmadan iş akışını sınamak gerekiyorsa,
çağıran tarafın kurduğu küçük bir arayüz işe yarayabilir. Trait’i tüketicinin
ihtiyacından tarif et. Örneğin uygulama kaydın alınmasını istiyorsa `SaveReview`
yalnızca ilgili işlemi anlatır. Uygulama SQLite havuzunu kendi açmaya
başladığında kuralı sınamak için disk ve kurulum da gerekebilir. Havuzu
başlangıçta oluşturup uygulamaya verme kararı bu bağı ayırır. [Clean Code zihin haritası · Bl. 6, 10 ve 11](https://github.com/saadettinBerber/clean-code-philosophy/blob/5321af5/zihin-haritasi.md)

## Hata da sözleşmenin parçası

Bir isteğin kabul edilmemesi çağırana bir sonraki adımı söylemeli. Boş kart
metni düzenlenebilir. Gizli cevap önce açılabilir. Önceden değerlendirilmiş
kart için mevcut sonuç gösterilebilir. Üçünü aynı hata metnine sıkıştırırsan
arayüz davranışını metin ayrıştırarak seçmek zorunda kalır. İçeride anlamlı
varyantlar, sınırda kullanıcıya yönelik mesajlar kullan. `None`, henüz sonuç
olmayan oturumun olağan durumudur. `Err`, denenen işlemin başarısız olduğunu
bildirir.

Rust’ta bazı hatalar çalışma anına kalmadan engellenebilir. Üç durumun ayrı
varyantları varsa aynı anda hem gizli hem değerlendirilmiş bir oturum
kurulamaz. Yine de geçişin hangi sırada yapılacağı bir ürün kuralıdır.
`grade` bu kuralı uygular. Derleyici durumların temsilini, testler izin
verilen geçişleri korur. [Clean Code · Bl. 7 ve 8](https://github.com/saadettinBerber/clean-code-philosophy/blob/5321af5/zihin-haritasi.md) · [Rust Book · Result](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)

## Artımlı geliştirme ve inceleme

1. Bir davranışı sabitle. Örneğin ikinci değerlendirme reddedildiğinde ilk
   kararın korunduğunu gösterecek bir test yaz veya mevcut testi bul.
2. Çalışan küçük adımı kur. İş kuralını doğru yere yerleştir ve gereksiz
   sahiplik aktarımını, kopyalamayı veya yeni bağımlılığı gözden geçir.
3. Testler yeşilken niyeti açığa çıkar. Ad, tür, modül ve tekrar hakkında
   ayrı karar ver. Kapsamı büyüten yeniden düzenlemeyi ayrı değişiklik olarak
   incelemek daha kolaydır.
4. Davranışı gerçekleştiği sınırda gözle. Saf sıra kuralı için oturum testi
   yeterli olabilir. İki istemcinin aynı kaydı güncellemesi söz konusuysa
   geçici SQLite ve gerçek işlem sınırı gerekir. Tek tek doğru iki çağrının
   birlikte atomik olduğu varsayımıyla ilerleme.
5. Biçim ve lint sonuçlarını değerlendir. Rustfmt biçimi düzenler. Clippy
   bazı riskli veya okunması zor kalıpları işaretler. Hangi davranışın kimde
   kaldığını kod incelemesinde ayrıca değerlendir.

Bu repo, Clean Code notlarından türetilen bir **Rust uygulaması** gösteriyor.
Notlardaki Java terimlerinin karşılığını davranış üzerinden arıyoruz.
Örneğin kapsülleme özel alanlar ve herkese açık birkaç metotla, durumlar
`enum` ile, dış sınırlar gerektiğinde trait ile ifade ediliyor. Satır ve
argüman sayısı incelemeyi başlatabilir. Karar için test, sorumluluk ve okuma
maliyeti birlikte ele alınır. [Clean Code zihin haritası · Bl. 13–14 ve 17](https://github.com/saadettinBerber/clean-code-philosophy/blob/5321af5/zihin-haritasi.md)

Bir sonraki okuma, aynı önceliklerin Google’ın kamuya açık Rust rehberi ve
kod inceleme sürecinde nasıl karşılık bulduğunu ele alıyor. [Google’ın Rust yaklaşımı](google-rust.md)
