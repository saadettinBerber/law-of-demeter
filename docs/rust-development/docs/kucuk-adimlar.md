# Kart sorusu arama · Küçük adımlarla yeniden düzenleme

## Ne yapıyoruz?

Bir listeyi sırayla tarayıp verilen metinle başlayan ilk kart sorusunu
buluyoruz. Örneğin `Rust` aramasının ilk sonucu `Rust'ta sahiplik`.
Eşleşme yoksa işlev `None` döndürüyor. Girdileri ödünç alma ve yalnızca
bulunan soruyu kopyalama adımlarında bu davranışı koruyoruz.

## Hangi dosyalara bakıyoruz?

`examples/first_matching_prompt.rs` dosyasının başında arama işlevi,
ardından çalıştırılabilir örnek ve iki davranış testi var.
[Arama kodu ve testleri](../examples/first_matching_prompt.rs)

## Nasıl çalıştırıyoruz?

Depo kökünde önce örneği çalıştır, sonra aynı dosyadaki testleri koştur.

```sh
cargo run --locked --example first_matching_prompt
cargo test --locked --example first_matching_prompt
```

İlk komut `Rust'ta sahiplik` yazdırır. İkinci komut ilk eşleşmenin
seçildiğini ve eşleşme yoksa sonuç dönmediğini sınar.

Robert C. Martin, *Clean Code*’un 14. bölümündeki “On Incrementalism” kısmında
(s. 212–214) bir düzenlemeyi küçük adımlara bölüp her adımda davranışı yeniden
doğrulamayı anlatıyor. Burada önce girdilerin sahipliğini, sonra kopyanın ne
zaman alındığını değiştiriyoruz. [*Clean Code*, 14. bölüm](https://www.oreilly.com/library/view/clean-code-a/9780136083238/chapter14.xhtml)

## Başlangıç · Çalışan davranışı sabitle

**Her testte tek bir davranışı sınamak.** Martin, bir testin birkaç ilgisiz
davranışı bir araya getirmesi hâlinde okuyanın testin neyi koruduğunu
anlamak için fazladan uğraşacağını anlatıyor. Bu yüzden örnekte iki ayrı
test var. Biri birden fazla eşleşme varsa ilkinin seçildiğini, diğeri
eşleşme yoksa `None` döndüğünü gösteriyor. İlk test, bulunan metnin liste
kapsamdan çıktıktan sonra da kullanılabildiğini gösteriyor.
[*Clean Code*, 9. bölüm, “Single Concept per Test”, s. 131](https://www.oreilly.com/library/view/clean-code-a/9780136083238/chapter09.xhtml)

Fonksiyon listeyi ve öneki sahipleniyor. Her soruyu karşılaştırmadan önce
kopyalıyor. Eşleşmeyen kopyalar döngü içinde elden çıkıyor.

```rust
fn first_matching_prompt(prompts: Vec<String>, prefix: String) -> Option<String> {
    for prompt in &prompts {
        let candidate = prompt.clone();
        if candidate.starts_with(prefix.as_str()) {
            return Some(candidate);
        }
    }

    None
}
```

Çağrı, listeyi ve yeni oluşturulan öneki fonksiyona taşır.

```rust
let picked = first_matching_prompt(prompts, String::from("Rust"));
```

## 1 · Girdileri ödünç al

**Fonksiyon argümanlarını çağıranın anlayacağı biçimde seçmek.** Martin,
argümanların okuyana o çağrı için gereksiz ayrıntı yükleyebileceğini
anlatıyor. Rust’ta bu arama için çağıranın yalnızca bir soru dilimi ve
bir metin vermesi yeterli. `&[String]` ve `&str` imzası, aramanın bu
girdileri okuyacağını ve sahipliğini devralmayacağını gösteriyor.
[*Clean Code*, 3. bölüm, “Function Arguments”, s. 40](https://www.oreilly.com/library/view/clean-code-a/9780136083238/chapter03.xhtml)

Arama, soru listesini ve öneki okuyor. İmza bu iki girdiyi ödünç alabilir.
Çağrı `first_matching_prompt(&prompts, "Rust")` olur ve liste çağıranda kalır.
Kopyanın yeriyle `Option<String>` dönüşü bu adımda aynı kalır.

```rust
fn first_matching_prompt(prompts: &[String], prefix: &str) -> Option<String> {
    for prompt in prompts {
        let candidate = prompt.clone();
        if candidate.starts_with(prefix) {
            return Some(candidate);
        }
    }

    None
}
```

## 2 · Yalnızca eşleşeni kopyala

**Niyeti kodda açıkça ifade etmek.** Martin, kodu daha sonra değiştirecek
kişinin yazarın niyetini koddan anlayabilmesini istiyor. Kopyayı
`if` dalındaki `return Some(...)` ifadesine aldığımızda, okuyan kişi
kopyanın döndürülecek metin için üretildiğini aynı yerde görüyor.
[*Clean Code*, 12. bölüm, “Expressive”, s. 175](https://www.oreilly.com/library/view/clean-code-a/9780136083238/chapter12.xhtml)

`starts_with` metni okuyabildiği için karşılaştırma sırasında kopya
gerekmiyor. Kopyayı eşleşme dalına taşı. Böylece eşleşme varsa bir, yoksa
sıfır `clone` çağrısı yapılır. Çağıran yine bağımsız bir `String` alır.

```rust
fn first_matching_prompt(prompts: &[String], prefix: &str) -> Option<String> {
    for prompt in prompts {
        if prompt.starts_with(prefix) {
            return Some(prompt.clone());
        }
    }

    None
}
```

## Her adımda aynı davranışı kontrol et

**Yeniden düzenlemeyi küçük, doğrulanabilir adımlarla yapmak.** Martin,
`Args` ayrıştırıcısında önce bir seçenek türünü değiştiriyor. Bir test
eksik seçeneğin eski davranışının bozulduğunu gösterince bir sonraki
adımdan önce onu düzeltiyor. Burada da önce girdi imzasını değiştir,
iki kontrolü çalıştır, sonra kopyanın yerini değiştirip aynı kontrolleri
yeniden çalıştır. Böylece bir hata çıkarsa hangi değişiklikte başladığını
görebilirsin. [*Clean Code*, 14. bölüm, “On Incrementalism”, s. 212–214](https://www.oreilly.com/library/view/clean-code-a/9780136083238/chapter14.xhtml)

Ayrı bir deneme dosyasında, yukarıdaki `first_matching_prompt`
sürümlerinden birini aşağıdaki `main` ile birleştir. Başlangıç sürümünü
denerken iki çağrıdaki `&prompts` yerine `prompts`, `"Rust"` yerine
`String::from("Rust")` kullan. Her sürümde beklenen sonuçlar aynı. İlk
kontrol iki eşleşmeden ilkinin seçildiğini ve metnin liste kapsamdan
çıktıktan sonra kullanılabildiğini gösterir. İkincisi eşleşme yokluğunu
kontrol eder.

```rust
fn main() {
    let picked: Option<String> = {
        let prompts = vec![
            String::from("SQLx ile sorgu"),
            String::from("Rust'ta sahiplik"),
            String::from("Rust'ta hata yönetimi"),
        ];
        first_matching_prompt(&prompts, "Rust")
    };
    assert_eq!(picked.as_deref(), Some("Rust'ta sahiplik"));

    let prompts = vec![String::from("SQLx ile sorgu")];
    assert_eq!(first_matching_prompt(&prompts, "Rust"), None);
}
```

Depodaki sürümü ve iki testi yukarıdaki komutlarla yeniden çalıştırabilirsin.

## Ödünç döndürmek ayrı bir karar

**Kararı gerekli bilgiye sahip olana kadar ertelemek.** Martin,
tasarım kararlarını ihtiyaç belli olana kadar ertelemenin daha bilinçli
seçimler yapmayı sağladığını anlatıyor. Bu örnekte çağıran sonucu liste
kapsamdan çıktıktan sonra da kullanıyor. Bu yüzden `Option<String>`
dönüşünü koruyoruz. Çağıranın ihtiyacı değişirse ödünç dönüşü ayrı bir
sözleşme değişikliği olarak değerlendiriyoruz.
[*Clean Code*, 11. bölüm, “Optimize Decision Making”, s. 167–168](https://www.oreilly.com/library/view/clean-code-a/9780136083238/chapter11.xhtml)

Çağıran metni yalnızca liste kapsamdayken kullanacaksa dönüşü `Option<&str>`
olarak tasarlayabilirsin. Aşağıdaki imzada `'a` sonucu listeye bağlar.
Önek yalnızca karşılaştırma sırasında kullanılır ve daha erken kapsamdan
çıkabilir. Döngü eşleşen sorunun metnini doğrudan ödünç verir.

```rust
fn first_matching_prompt<'a>(prompts: &'a [String], prefix: &str) -> Option<&'a str> {
    for prompt in prompts {
        if prompt.starts_with(prefix) {
            return Some(prompt.as_str());
        }
    }

    None
}
```

Bu sürümü yukarıdaki `main` ile denemek için `picked` türünü
`Option<&str>` yap ve ilk karşılaştırmada `picked` kullan. Derleyici bu
kez listenin içinden ödünç alınan metnin blok dışında kullanıldığını
gösterir. Metin listeden bağımsız yaşayacaksa ikinci adımdaki
`Option<String>` sözleşmesi bu ihtiyacı karşılar. [Rust Book · Ödünç alma](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html) · [Rust Book · Yaşam süreleri](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)

## Ek okuma · Sipariş arama örneği

Let’s Get Rusty’nin *Fix Your Rust Functions* videosunda bir müşterinin
son siparişlerini bulan işlev önce girdilerini ödünç alıyor. Sonra
yalnızca döndüreceği siparişleri kopyalıyor. Kart sorusu arama örneği
bu iki kararı ilk eşleşmeyi bulma üzerinden gösteriyor.
[Fix Your Rust Functions](https://www.youtube.com/watch?v=X5Bf_Udenys)
