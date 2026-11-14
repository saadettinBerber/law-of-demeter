# Google’ın kamuya açık Rust yaklaşımı

Google Rust Style Guide biçimin yanında API ve uygulama tasarımını da ele alıyor.
Kılavuz önce kamusal Rust biçim ve API yönergelerini temel alıyor. Ek kuralları
okunabilirlik, tutarlılık ve çok geniş kod tabanlarında araçla değişiklik
yapabilmek açısından gerekçelendiriyor. Bir çağrının kökeni açık olduğunda
inceleyen kişi kodu daha kolay izler. Aynı düzen bir yeniden adlandırma aracının
işini de kolaylaştırabilir. [Google Rust Style Guide · Amaç ve kapsam](https://google.github.io/styleguide/rustguide.html#goals-of-the-style-guide) · [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

Google’ın yayımladığı belgeler üç farklı soruya yanıt veriyor. Rust rehberi
imza ve uygulama kararlarını anlatıyor. Engineering Practices değişikliğin
nasıl hazırlanıp incelendiğini açıklıyor. Chromium ve Fuchsia belgeleri de
Rust kodunun kendi yapı sistemi, yürütme modeli ve test altyapısıyla nasıl
birleştiğini gösteriyor.

## Önce çağıranı düşün

`Review` bu depodaki kart oturumu örneğidir. Kartın soru ve cevap metnini
saklar, kullanıcının cevabı açıp değerlendirme yapmasını yönetir.
[Kart oturumu](kart-oturumu.md)

| Konu | Google rehberindeki gerekçe | Kart oturumundan örnek |
| --- | --- | --- |
| Uygulama ve kütüphane | Rehber API kurallarını kütüphanelerde bağlayıcı sayıyor. Uygulamada ise kullanım senaryoları daha iyi bilindiği için doğrudan çözümü tercih ediyor. | `Review::try_new` zaten saklayacağı iki `String`i alıyor. Genel amaçlı bir kütüphanede çağıranların hangi türlere sahip olduğunu görüp `Into<String>` esnekliğini ayrıca değerlendirebiliriz. |
| Ödünç alma ve paylaşım | Salt okuma için `&str` ve `&[T]` gibi veri görünümleri çağıranı belirli bir kapsayıcıya bağlamıyor. Derin kopya ve ortak sahiplik kendi maliyetini taşıyor. | `prompt` bir metin görünümü veriyor. Oturum kendi `String`ini tutuyor. Başka bir görev gerçekten aynı değere sahip olacaksa `Arc` kararının anlamı değişiyor. |
| Sonucun biçimi | Kütüphane çağıranları iterator ile farklı şekillerde tüketebilir. Uygulamada tüketim şekli belliyse veya zaten bir koleksiyon oluşuyorsa `Vec` daha okunur olabilir. | Bir listeyi her seferinde bellekte hazırlayıp JSON’a çevirecek bir uygulamada `Vec` doğrudan seçilebilir. Büyük ya da tembel işlenecek akışta bellek ve yaşam süresi yeniden değerlendirilir. |
| İşlem ve hata | Belirsiz bir `bool` yerine anlamı taşıyan `enum` kullanılır. `Option`, `Result` ve panic arasında seçim API sözleşmesine göre yapılır. | `grade(Outcome::Known)` çağrısı kararı gösteriyor. `None` henüz sonuç bulunmadığını, `Err(AnswerHidden)` hangi işlemin eksik olduğunu bildiriyor. |
| Trait ve makro | Ek soyutlama, çağrı yerini ve araçların kodu takip etmesini zorlaştırabilir. Rehber sıradan Rust öğeleriyle başlanmasını ve ihtiyaç oluştuğunda trait eklenmesini öneriyor. | Kart oturumu somut bir tür. Aynı işlemin farklı depolama uygulamalarına ihtiyacı olduğunda trait’in yeri kartın kendisi değil depolama sınırı olur. |

İş kuralı durumları ve hataları zaten alan diliyle tarif edebiliyorsa
çözüme yeni bir yardımcı dil eklemek bakım maliyetini artırabilir. Örneğin
`match`, üç durumun geçişini birlikte gösterir. `Review::try_new`
başarısız olabilen bir kurucu olduğunu adında taşır. `open` gibi bir isim
ise dış dünyaya gerçekten bağlanan bir işlemin niyetini daha iyi anlatabilir.
[API tasarımı](https://google.github.io/styleguide/rustguide.html#apply-api-design-rules-in-context) · [Sahiplik](https://google.github.io/styleguide/rustguide.html#ownership-and-borrowing) · [Hatalar](https://google.github.io/styleguide/rustguide.html#reporting-errors) · [Kurucular](https://google.github.io/styleguide/rustguide.html#naming-initialization-functions)

Bir bağımlılığın gerçekten paylaşıldığı örnek Flashcards’ta bulunuyor. Birden
fazla HTTP isteği aynı repository’yi kullanıyor. Uygulama onu `Arc<dyn Repository>`
olarak alıyor. Kart kuralı doğrulama sırasında bu paylaşım mekanizmasını
bilmeden çalışıyor. Aynı tercihi tek bir `Review` nesnesinin her alanına
uygulamak için bir sahiplik gerekçesi yok. [Flashcards Rust rehberi](https://github.com/saadettinBerber/flashcards/blob/main/docs/better_rust.md)

## Tutarlılığın bakım maliyetine etkisi

Bir modül düzeyindeki fonksiyon çağrısında Google rehberi modül adını da
göstermeyi öneriyor. Örneğin `sqlite::open(path)` ifadesini okuyan biri
çağrının kökenini aynı satırda görür. Tür adları için rehberin tercihi
içe aktarıp yalın kullanmaktır. Geniş `use module::*` içe aktarmaları ise
hangi adların nereden geldiğini belirsizleştirebilir. Bu seçimler bir
yeniden adlandırma ya da taşıma çalışmasında hem insanın hem aracın
değişiklik yerlerini bulmasını kolaylaştırır. [Google Rust · Kod düzeni](https://google.github.io/styleguide/rustguide.html#code-organization)

Kapsam kütüphane sınırına gelince karar değişir. Depodaki özel `State`
enum’una yeni bir varyant eklersek derleyicinin ilgili `match`leri
göstermesi işimize yarar. Bir kütüphanenin yayımlanan enum’unda ise dış
kullanıcılar da bütün varyantları eşleştirebilir. Google’ın
`#[non_exhaustive]` önerisi, gelecekte yeni bir varyant eklenebileceğini
dış çağıranların baştan hesaba katmasını sağlar. İç tasarımı genişletmek
ile başkalarının sözleşmesini genişletmek farklı maliyetler taşır.
[Google Rust · Public enum’lar](https://google.github.io/styleguide/rustguide.html#apply-non_exhaustive-to-publicly-exposed-enums) · [Rust Reference · non_exhaustive](https://doc.rust-lang.org/reference/attributes/type_system.html#the-non_exhaustive-attribute)


## Kod incelemesi nasıl ilerler?

Google’ın belgelerinde **CL**, incelemeye gönderilen değişiklik. Burada aynı
fikir bir commit veya PR üzerinden uygulanabilir. Kamuya açık süreç bir
değişikliği, kod tabanının zaman içindeki sağlığına katkısına göre tartıyor.
İnceleyen kişi çalışmayı kendi zevkine göre yeniden yazdırmak yerine teknik
sonuca, mevcut sözleşmelere ve kullanıcının ihtiyacına bakıyor. Bir tercih
standart veya doğruluk meselesiyse nedenini açıklıyor. Kişisel öneriyi de
öneri olarak işaretleyebiliyor. [Google · İnceleme standardı](https://google.github.io/eng-practices/review/reviewer/standard.html)

1. **Yazar değişikliği sınırlıyor.** Bir CL kendi başına açıklanabilen bir
   davranışı ele alıyor. Açıklamada kullanıcı davranışı, gerekçe ve kontrol
   yolu var. Büyük iş gerektiğinde her biri incelenebilir adımlara ayrılıyor.
2. **İnceleyen önce tasarımı okuyor.** Bu kural doğru modülde mi? Çağıranlar
   davranışı anlıyor mu? Değişiklik sistemin geri kalanına uyuyor mu?
3. **Ardından davranış ve karmaşıklık geliyor.** Hata yolları, eşzamanlılık
   ve gerçekten gerekli olan esneklik ele alınıyor. Testin bir hata
   oluştuğunda kırılıp kırılmayacağına bakılıyor.
4. **Ad, yorum ve belge de inceleniyor.** Adlar niyeti taşıyor. Yorumlar
   kodun zaten gösterdiği satırı tekrar etmek yerine gerekçeyi açıklıyor.
   Kullanıcıya açık davranış değiştiğinde ilgili belge de güncelleniyor.

[Google · Küçük CL’ler](https://google.github.io/eng-practices/review/developer/small-cls.html) · [CL açıklamaları](https://google.github.io/eng-practices/review/developer/cl-descriptions.html) · [İnceleme ölçütleri](https://google.github.io/eng-practices/review/reviewer/looking-for.html)

**Bu repo için örnek.** Kullanıcı değerlendirmesini değiştirebilsin istendiğinde ilk
inceleme sorusu yeni metot adı değildir. Bir istemci, yanıtı zaman aşımına
uğradığı için eski `grade` çağrısını tekrar gönderirse bu bir düzeltme sayılacak
mı? Ürün iki eylemi ayrı tutacaksa ilk kaydı `grade`, açık bir düzeltmeyi
`change_grade` temsil edebilir. Bu durumda test, eski `grade` çağrısının
sonucu değiştirmediğini ve açık düzeltmenin değiştirdiğini ayrı ayrı gösterir.
Yalnız bellekteki oturum değişecekse ilk CL’nin kapsamı bu olabilir. Kayıt
API’si veya SQLite’a geçmiş bir sonuç söz konusuysa revizyon ve atomik güncelleme
kendi incelemesini gerektirir. [Kart örneği](kart-oturumu.md)

## Proje bağlamını ayır

| Bağlam | Yayımlanan uygulama | Aktarılabilir soru |
| --- | --- | --- |
| Genel Rust rehberi | Açık imzalar, anlaşılır hatalar, olağan Rust araçları ve incelenebilir kod. | Çağıran, kodun veri ve hata sözleşmesini imzadan anlayabiliyor mu? |
| Chromium | İlk taraf Rust kütüphaneleri GN ile karma Rust/C++ derlemesine giriyor. Rehber Chromium’da `async/await` kullanmamayı ve thread yerine sequence kullanmayı söylüyor. Testlerde Chromium’un gtest entegrasyonu mevcut. | Bir iş mevcut yürütme ve test altyapısına nasıl uyacak? |
| Fuchsia | Rust rehberindeki Fuchsia’ya özel kural, platformun `fuchsia_sync::Mutex` türünü kullanıyor. | Senkronizasyon aracını uygulamanın çalıştığı platform ve bağlam belirliyor mu? |
| Flashcards | Axum ve Tokio ağ işlemlerini yürütüyor. SQLx SQLite bağlantısı kalıcılığı sağlıyor. Kart ve sınav kararları senkron Rust olarak kalıyor. | Dış işi yapan kod ile test edilecek ürün kuralının sınırı nerede? |

[Google Rust · Eşzamanlılık](https://google.github.io/styleguide/rustguide.html#concurrency) · [Chromium · Rust entegrasyonu](https://github.com/chromium/chromium/blob/e39c67e720091d6a290a091aa46bceaef43adb2f/docs/rust.md) · [Fuchsia · Rust](https://fuchsia.dev/fuchsia-src/development/languages/rust) · [Flashcards mimarisi](https://github.com/saadettinBerber/flashcards/blob/main/ARCHITECTURE.md)

Clean Code’un artımlı tasarım anlayışı bu inceleme süreciyle aynı soruyu
paylaşıyor. Bir değişikliği yazdığımızda, sonraki kişi davranışı güvenle
anlayıp değiştirebilecek mi? Clean Code notları bunu test, niyet ve
sorumluluk üzerinden tarif ediyor. Google’ın rehberleri buna ekipçe ortak
API tercihleri, küçük CL’ler ve bağlama göre uygulama kuralları ekliyor.
