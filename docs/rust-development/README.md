<p align="center">
  <img src="media/banner.gif" width="100%" alt="Rust ile geliştirme. Review imzalarında ödünç alma, durum değişikliği ve hata sonucu sırayla vurgulanıyor.">
</p>

<p align="center">
  <img alt="Rust · Edition 2024" src="https://img.shields.io/badge/Rust-Edition%202024-903b26?style=flat-square&amp;logo=rust&amp;logoColor=white">
  <img alt="Kaynak · Clean Code" src="https://img.shields.io/badge/kaynak-Clean%20Code-5b4237?style=flat-square">
  <img alt="Kaynak · Google Rust" src="https://img.shields.io/badge/kaynak-Google%20Rust-5b4237?style=flat-square">
</p>

<p align="center">
  <a href="#bir-kartın-yolculuğu">Kart oturumu</a> ·
  <a href="#küçük-adımlarla-iyileştirme">Kart sorusu arama</a> ·
  <a href="#rust-proje-yapısı">Kart listesini gösterme</a> ·
  <a href="#clean-code">Clean Code</a> ·
  <a href="#google-rust">Google Rust</a> ·
  <a href="#geliştirme-akışı">Geliştirme akışı</a>
</p>

**Rust ile geliştirme** bir karar defteri. Bir özellik yazarken nereden
başladığımızı, iş kuralını hangi türe koyduğumuzu, verinin sahipliğini kime
verdiğimizi ve bir değişikliği nasıl incelediğimizi anlatıyor. Üç bağımsız
örnek, bir kart çalışma aracının farklı işlerini ele alıyor.

Rust ve Cargo kuruluysa komutları terminalde bu README’nin bulunduğu
dizinden çalıştırabilirsin.

| Örnek | Ne yapıyoruz? | Hangi dosyalara bakıyoruz? | Nasıl çalıştırıyoruz? |
| --- | --- | --- | --- |
| [Kart oturumu](docs/kart-oturumu.md) | Cevabı açıp değerlendirme yapıyoruz. | [Oturum kodu](src/lib.rs) ve [davranış testleri](tests/review.rs) | [Test komutu](docs/kart-oturumu.md#nasıl-çalıştırıyoruz) |
| [Kart sorusu arama](docs/kucuk-adimlar.md) | `Rust` ile başlayan ilk soruyu buluyoruz. | [Arama kodu ve testleri](examples/first_matching_prompt.rs) | [Çalıştırma ve test komutları](docs/kucuk-adimlar.md#nasıl-çalıştırıyoruz) |
| [Kart listesini gösterme](docs/proje-yapisi.md) | İki soruyu önce düz metin, sonra başlıklı önizleme olarak gösteriyoruz. | [Tek dosyalık başlangıç kodu](examples/proje-yapisi/00-tek-dosya/src/main.rs) | [Başlangıç komutu ve dört aşama](docs/proje-yapisi.md#nasıl-çalıştırıyoruz) |

## Bir kartın yolculuğu

<p align="center">
  <img src="media/review.gif" width="100%" alt="Kart Hidden durumunda başlar. reveal_answer kartı Revealed durumuna taşır. grade sonucu Graded durumunda saklar. Gizli cevabı değerlendirme ve ikinci değerlendirme reddedilir.">
</p>

`Hidden` durumunda değerlendirme isteği reddedilir. `reveal_answer()` cevabı
açıp metni ödünç verir. `grade(Outcome::Known)` değerlendirmeyi kaydeder.
Aynı çağrı tekrar gelirse ilk sonuç korunur. Bu sıranın Rust’taki karşılığını,
hata yollarını ve derleyici alıştırmasını aynı örnekte inceleyebilirsin.

[Kart oturumunu adım adım oku →](docs/kart-oturumu.md) · [Rust kodu](src/lib.rs) · [Davranış testleri](tests/review.rs)

## Clean Code

Bir sonraki değişikliği düşün. Test, bugünkü davranışın hangisini koruyor?
Aynı kural iki yerde yaşıyorsa hangisi zamanla geride kalacak? Bir çağrıyı
okuyan kişi argümanın ve sonucun anlamını görebiliyor mu? Kart oturumunda
bu sorular bir karar zincirine dönüşüyor. Test sırayı sabitliyor.
`Review::grade` kuralın sahibi oluyor. `Outcome::Known` çağrı yerinde
niyeti gösteriyor. Özel bir `enum` da üç durumu ifade etmeye yetiyor.

[Clean Code’u Rust’ta uygulamak →](docs/clean-code.md)

### Küçük adımlarla iyileştirme

Robert C. Martin, *Clean Code*’un 14. bölümündeki “On Incrementalism”
kısmında `Args` ayrıştırıcısını önce yalnızca boolean seçenekleri için
yeniden düzenliyor. Birim ve kabul testleri, eksik bir seçeneğin eski
davranışının bozulduğunu gösterince Martin önce bu davranışı geri getiriyor.
Böylece her adımın etkisini ayrı görebiliyor. [*Clean Code*, 14. bölüm, s. 212–214](https://www.oreilly.com/library/view/clean-code-a/9780136083238/chapter14.xhtml)

Kart sorusu arama örneği, verilen önekle başlayan **ilk** soruyu buluyor.
Eşleşme yoksa `None` dönüyor. Başlangıçta fonksiyon listeyi ve öneki
sahipleniyor ve baktığı her metni eşleşmeyi kontrol etmeden önce kopyalıyor.

| Küçük adım | Ne değişir? |
| --- | --- |
| Başlangıç · Davranışı testle sabitle | İlk eşleşme ve eşleşme yokluğu için ayrı testler kur. |
| 1. Girdiyi ödünç al | Fonksiyon argümanlarını yaptığı işe göre seç. Arama, listeyi `&[String]` ve öneki `&str` olarak okuyabilir. |
| 2. Önce seç, sonra kopyala | Niyeti kodda göster. Kopyalamayı, eşleşen değerin döndürüldüğü dalda yap ve aynı testleri yeniden çalıştır. |
| Ayrı karar · Ödünç dönüşü değerlendir | Kararı çağıranın ihtiyacına göre ver. Bu örnekte `Option<String>`, bulunan metni liste kapsamdan çıktıktan sonra da kullanabilmeyi sağlıyor. |

[Fonksiyonu adım adım incele](docs/kucuk-adimlar.md) · [Çalışan örnek ve iki test](examples/first_matching_prompt.rs)

```sh
cargo run --locked --example first_matching_prompt
cargo test --locked --example first_matching_prompt
```

### Rust proje yapısı

Bir modül, aynı nedenle değişen kodu bir arada tutar. İki çalıştırılabilir
program aynı işlevi kullanacaksa tek pakette `lib.rs` ile paylaşılabilir.
Programları farklı bağımlılıklarla derlemek gerektiğinde ayrı Cargo
paketleri anlam kazanır. Rehber, bu kararları çevrimdışı çalışan dört
aşama ve her aşamanın Clean Code dayanağıyla gösteriyor. Düz çıktı ve
başlıklı önizleme aynı kart listesini kullanıyor.

[Modülden kütüphaneye, oradan workspace’e →](docs/proje-yapisi.md)

## Google Rust

Google’ın Rust rehberi, uygulama koduyla başkalarının kullanacağı kütüphane
API’sini ayrı değerlendiriyor. Uygulamada çağıranların ihtiyaçları biliniyorsa
bir `Vec` veya somut tür yeterli olabilir. Kütüphanede çağıranların veriyi
farklı şekillerde tüketmesi, iterator ya da daha esnek bir API’yi anlamlı
kılabilir. Google’ın inceleme rehberi de değişikliği küçük tutmayı ve
tasarım, davranış, karmaşıklık ile testleri birlikte değerlendirmeyi anlatıyor.
[Google Rust Style Guide](https://google.github.io/styleguide/rustguide.html#apply-api-design-rules-in-context) · [Google Engineering Practices](https://google.github.io/eng-practices/review/)

[Google’ın Rust ve kod inceleme yaklaşımı →](docs/google-rust.md)

## Geliştirme akışı

| Adım | İncelemeye getirdiğin şey |
| --- | --- |
| **Davranışı yaz** | Bir kabul ve bir ret örneği. Örneğin gizli cevaba verilen değerlendirme reddedilir ve oturum sürdürülebilir. |
| **İmzayı seç** | Kimin sahip olduğu, neyin değiştiği ve çağıranın hangi sonuçları ayırt edeceği. |
| **Sınırı kur** | Ürün kuralının sahibi. Gerekirse ağ, dosya veya veritabanı işleminin yeri. |
| **Test et ve incele** | Kuralı bozan değişiklikte kırılan bir test ve küçük bir değişiklik açıklaması. |

```sh
cargo check --locked
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Kökteki `cargo test` kart oturumunun testlerini çalıştırır. Kart sorusu
arama ve kart listesi aşamalarının komutlarını yukarıdaki tablodan açabilirsin.

Ajanlar için kullanım sırası [AGENTS.md](AGENTS.md) dosyasında. İki GIF’i
yeniden üretmek için Node.js ve FFmpeg kurulu bir ortamda
`bash media/kaynak/gif-uret.sh` çalıştırılabilir. [HyperFrames kaynağı](media/kaynak/index.html)

<p align="center"><sub>README düzeni <a href="https://github.com/saadettinBerber/oku-components">oku-components</a>’tan esinlendi. Animasyonlar <a href="https://hyperframes.heygen.com">HyperFrames</a> ile üretildi.</sub></p>
