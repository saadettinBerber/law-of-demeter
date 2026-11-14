# Kart oturumu

## Ne yapıyoruz?

Kullanıcı önce cevabı görür, sonra bildim veya bilemedim diye
değerlendirir. Gizli cevap değerlendirilemez. Sonuç oluştuktan sonra bir
`grade` çağrısı daha aynı oturumu değiştiremez. Cevap tekrar gösterilebilir.

## Hangi dosyalara bakıyoruz?

`src/lib.rs` içindeki `Review` oturumun durumunu ve izin verilen
geçişleri tanımlar. `tests/review.rs` bu davranışları dışarıdan çağıran
bir kodun gözünden sınar. [Oturum kodu](../src/lib.rs) · [Davranış testleri](../tests/review.rs)

## Nasıl çalıştırıyoruz?

Bu örnek bir kütüphane. Oturumu denemek için depo kökünde davranış
testlerini çalıştır.

```sh
cargo test --locked --test review
```

Üç testin de geçtiğini göreceksin. Testler gizli cevabın
değerlendirilemediğini, cevabı yeniden açmanın sonucu silmediğini ve boş
metnin reddedildiğini gösterir.

## İmza üzerinden veri akışını izle

| Çağrı | Veri ne olur? | Sözleşme |
| --- | --- | --- |
| `Review::try_new(prompt, answer)` | İki `String` oturuma taşınır. Metin kontrolü `trim` ile yapılır ve asıl biçim korunur. | Boş soru veya cevap kabul edilmez. Başarılı oturum gizli cevapla başlar. |
| `review.prompt()` | `&self` üzerinden `&str` ödünç alınır. | Okuma oturumu değiştirmez. |
| `review.reveal_answer()` | `&mut self` oturuma özel erişim verir. Dönüşteki `&str` oturumun içinde duran metindir. | İlk çağrıda cevap açılır. Bir sonuç varsa sonraki açış onu korur. |
| `review.grade(Outcome::Known)` | Değer alanı `Outcome` taşır. `&mut self` geçişe izin verir. | Başarılı çağrı `Ok(())` döner. Gizli ve değerlendirilmiş durumlarda hata farklıdır. |
| `review.outcome()` | Küçük bir `Copy` değeri döner. | Değerlendirme yapıldıysa `Some`, henüz yapılmadıysa `None`. |

İki `bool` alanı aynı anda birçok birleşime izin verirdi. Mevcut model
`Hidden`, `Revealed` ve `Graded(Outcome)` durumlarını adlandırır.
Özel alan sayesinde çağıran kod oturumu doğrudan `Graded` yapamaz. Bu
uygulamanın durumu bellekte durur. Kalıcılık eklendiğinde çakışan yazmalar
ve tekrar denemeler ayrıca tasarlanır. [Rust Book · Enum](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html)

## Testi inceleme gözüyle oku

| Test | Koruduğu davranış | Testi bozacak değişiklik |
| --- | --- | --- |
| `accepts_a_grade_only_after_the_answer_is_revealed` | İlk istek reddedilir. Oturum açılınca geçerli istek kabul edilir. | `Hidden` durumunda puana izin vermek veya reddedilen isteğin bir sonuç yazması. |
| `revealing_the_answer_again_preserves_the_grade` | İkinci açış ilk sonucu korur. Tekrar değerlendirme reddedilir. | `reveal_answer` her çağrıda durumu sıfırlarsa veya `grade` eski sonucu ezerse. |
| `rejects_blank_text_and_preserves_valid_formatting` | Boş metin reddedilir, geçerli metin özgün biçimiyle döner. | Boşluğu geçerli saymak veya geçerli metnin başındaki boşluğu kayıtta silmek. |

Sıra kuralını değiştiren bir PR’da önce bu tabloyu düşün. Bugünkü testi
kıran fark istenen davranışın parçasıysa beklentiyi de değiştir. Davranış
aynı kalacaksa testin iç durum adlarına veya satır sırasına bağlanmasına
gerek yok. [Clean Code’un test yaklaşımı](clean-code.md) · [Google’ın inceleme ölçütleri](google-rust.md)

## İki alıştırma

### 1. Derleyicinin itirazını gör

`tests/review.rs` dosyasına aşağıdaki deneme testini ekle ve depo
kökünden `cargo test --locked --test review` komutunu yeniden çalıştır.
Ödünç alınan metin kullanımdan çıkmadan oturumu değiştirmeye çalıştığı
için bu sürümde derleyici hata verir.

```rust
#[test]
fn try_using_a_borrowed_answer_after_grading() {
    let mut review = Review::try_new("borrow".into(), "ödünç almak".into()).unwrap();
    let answer = review.reveal_answer();
    review.grade(Outcome::Known).unwrap();
    println!("{answer}");
}
```

`answer` hâlâ kullanılacakken `grade` aynı oturuma tekrar özel erişim
istiyor. `println!` satırını `grade` öncesine al. Son kullanım o satırda
olunca ödünç alma sona erer ve sonraki çağrı derlenir. Metni gerçekten daha
sonra kullanmak gerekiyorsa `.to_owned()` ile oturumdan bağımsız bir
`String` oluşturmayı dene. Bu kez kopyalamanın hangi ihtiyacı karşıladığı
imzadan ve kullanımdan görülebilir. [Rust Book · Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)

### 2. Değişiklik talebini küçük bir PR’a dönüştür

Kullanıcı daha önce verdiği değerlendirmeyi değiştirebilsin. Önce şu iki
davranışı tarif et. İstemci aynı `grade` çağrısını tekrar yolladığında ne
olur? Kullanıcı açıkça değerlendirmeyi düzeltmek istediğinde ne olur?
Seçimin doğrultusunda başarısız ve başarılı bir senaryo yaz. Ardından
imzanı, geçişini ve dönüş değerini tasarla. Bir inceleyiciye bırakacağın kısa
açıklama davranış farkını ve onu koruyan testi göstermeli.
