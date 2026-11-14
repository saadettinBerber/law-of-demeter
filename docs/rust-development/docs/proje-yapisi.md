# Kart listesini gösterme · Tek dosyadan Cargo workspace’ine

## Ne yapıyoruz?

Önce iki kart sorusunu terminale yazdırıyoruz. Sonra aynı listeyi bir
başlıkla gösteren ikinci bir program ekliyoruz. Örnek, bu ihtiyaçlar
ortaya çıktıkça kodun tek dosyadan modüle, ortak kütüphaneye ve ayrı
Cargo paketlerine nasıl taşındığını gösteriyor.

## Hangi dosyalara bakıyoruz?

Her aşamada önce değişikliğin başladığı dosyayı aç.

| Aşama | İlk bakılacak dosya |
| --- | --- |
| Tek dosya | Liste ve terminale yazan kod aynı dosyada. [main.rs](../examples/proje-yapisi/00-tek-dosya/src/main.rs) |
| Modül | Listeyi hazırlayan iş ayrı dosyaya taşınıyor. [cards.rs](../examples/proje-yapisi/01-modul/src/cards.rs) |
| Kütüphane | İki programın çağırdığı işlev dışarı açılıyor. [lib.rs](../examples/proje-yapisi/02-kutuphane/src/lib.rs) |
| Workspace | Ayrı paketler üye olarak listeleniyor. [Cargo.toml](../examples/proje-yapisi/03-workspace/Cargo.toml) |

## Nasıl çalıştırıyoruz?

Depo kökünden tek dosyalık sürümü çalıştır. Dört aşama bu depodaki
yerel paketlerle çevrimdışı çalışır. Her sonraki aşamanın komutu kendi
bölümünde yer alıyor.

```sh
cargo run --quiet --offline --locked --manifest-path examples/proje-yapisi/00-tek-dosya/Cargo.toml
```

```text
Rust'ta sahiplik
Rust'ta hata yönetimi
```

## Hangi yapıyı ne zaman seçiyoruz?

| Şu anda neye ihtiyacın var? | Yapı |
| --- | --- |
| Bir giriş noktası ve rahat okunan birkaç satır | Tek dosya |
| Birlikte değişen kodu aynı yerde tutmak | Aynı crate içinde modül |
| Birden fazla giriş noktasının aynı kodu çağırması | Aynı paket içinde `src/lib.rs` |
| Bir programı diğerinin bağımlılıklarını almadan derlemek | Aynı workspace içinde ayrı Cargo paketleri |

**Modül** bir crate içindeki kodu düzenler. **Crate**, derleyicinin bir
kütüphane veya çalıştırılabilir program olarak derlediği birimdir.
**Paket**, bir `Cargo.toml` ile tanımlanır ve bir kütüphane ile birden fazla
çalıştırılabilir hedef barındırabilir. **Workspace**, ayrı Cargo paketlerini
birlikte yönetir. [Rust Book · Paketler ve crate’ler](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html) · [Cargo Book · Workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html)

## Başlangıç · Tek dosyada çalışan örnek

Programın tek işi iki kart sorusunu, verildikleri sırayla terminale yazdırmak.
Listeyi satırlara dönüştüren işlevle terminale yazan giriş noktası şimdilik
aynı dosyada rahatça okunuyor. [Başlangıç kodu](../examples/proje-yapisi/00-tek-dosya/src/main.rs)

```rust
const PROMPTS: [&str; 2] = ["Rust'ta sahiplik", "Rust'ta hata yönetimi"];

fn main() {
    println!("{}", render_prompts());
}

fn render_prompts() -> String {
    PROMPTS.join("\n")
}
```

**Çalışan yapıyı küçük adımlarla iyileştirmek.** Robert C. Martin, mevcut
davranışı yeniden çalıştırılabilen kontrollerle belirleyip her küçük
değişiklikten sonra tekrar doğruluyor. Burada başlangıç çıktısını kayda
alıyoruz. Modül ve paket sınırları değişirken bu iki satır aynı sırayla
dönecek. [*Clean Code*, 14. bölüm, “On Incrementalism”, s. 212–214](https://www.oreilly.com/library/view/clean-code-a/9780136083238/chapter14.xhtml)

Yukarıdaki başlangıç komutunun çıktısını sonraki aşamalarda
karşılaştırmak için kullan.

## 1 · Birlikte değişen kodu modüle taşı

Kart listesini ve onu satırlara dönüştüren işlevi `cards.rs` içine taşı.
`main.rs` terminale ne yazacağını `cards::render_prompts()` ile alır.
`mod cards;` bildirimi bu dosyayı aynı çalıştırılabilir crate’in parçası
yapar. Giriş noktası ve kart metni artık ayrı yerlerde okunur. Kart metninin
nasıl hazırlandığını değiştirmek istediğinde `cards.rs` dosyasına gidersin.
[Giriş noktası](../examples/proje-yapisi/01-modul/src/main.rs) · [Kart modülü](../examples/proje-yapisi/01-modul/src/cards.rs)

```rust
mod cards;

fn main() {
    println!("{}", cards::render_prompts());
}
```

**Tek sorumluluk ilkesi ve birbiriyle ilişkili kodu birlikte tutmak.**
Martin, bir modülün sorumluluğunu hangi nedenle değiştiğine bakarak tarif
ediyor. Birbirine ait veri ve işlemlerin aynı yerde tutulmasını da
önemsiyor. Örnekte soru metinleri ve onları satırlara dönüştüren iş aynı
nedenle değişiyor. Ekrana yazma biçimi ise giriş noktasının kararı.
Böylece ayrımın gerekçesi dosyanın uzunluğu yerine değişecek davranıştan
geliyor. [*Clean Code*, 10. bölüm, “The Single Responsibility Principle” ve “Cohesion”, s. 138–141](https://www.oreilly.com/library/view/clean-code-a/9780136083238/chapter10.xhtml)

```sh
cargo run --quiet --offline --locked --manifest-path examples/proje-yapisi/01-modul/Cargo.toml
```

Çıktıyı başlangıçtaki iki satırla karşılaştır. Paket hâlâ tek
çalıştırılabilir hedefe sahip. Kart metninin hazırlanması terminal
kodundan bağımsız değişmeye başladığında bu modül ayrımı işe yarar.

## 2 · Ortak işi kütüphane hedefinden çağır

Şimdi ikinci bir program, aynı kartları bir başlıkla göstermek istiyor.
Kart metnini hazırlayan işlevi `src/lib.rs` üzerinden dışarı aç. Düz
çıktı veren `src/main.rs` ile `src/bin/preview.rs` aynı
`card_tools::render_prompts()` işlevini çağırır. `tests/` içindeki
entegrasyon testi de kütüphanenin dışarı açtığı aynı işlevi kullanır.
[Kütüphane](../examples/proje-yapisi/02-kutuphane/src/lib.rs) · [Düz çıktı](../examples/proje-yapisi/02-kutuphane/src/main.rs) · [Önizleme](../examples/proje-yapisi/02-kutuphane/src/bin/preview.rs)

```text
02-kutuphane/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── cards.rs
│   ├── main.rs
│   └── bin/preview.rs
├── tests/
└── preview-format/
```

`lib.rs`, modülün iç düzenini saklayıp yalnızca iki programın kullanacağı
işlevi dışarı açıyor. [Entegrasyon testi](../examples/proje-yapisi/02-kutuphane/tests/cards.rs)

```rust
mod cards;

pub use cards::render_prompts;
```

**Ortak davranışın tekrarını önlemek.** Martin aynı bilginin birden fazla
yerde kodlanmasının, değişikliklerin bir yerde unutulmasına yol
açabileceğini gösteriyor. Örnekte kart sırası ve satırların hazırlanması
tek işlevde kalıyor. Her program yalnızca çıktıyı nasıl sunacağını
belirliyor. Dışarıdan çağrılan bir test de ortak işlevin ürettiği metni
doğruluyor. [*Clean Code*, 12. bölüm, “No Duplication”, s. 173–174](https://www.oreilly.com/library/view/clean-code-a/9780136083238/chapter12.xhtml)

```sh
cargo run --quiet --offline --locked --manifest-path examples/proje-yapisi/02-kutuphane/Cargo.toml --bin card-tools
cargo run --quiet --offline --locked --manifest-path examples/proje-yapisi/02-kutuphane/Cargo.toml --bin preview
cargo test --offline --locked --manifest-path examples/proje-yapisi/02-kutuphane/Cargo.toml
```

Düz çıktı aynı iki satır. Önizlemenin başına `Kart önizlemesi` ekleniyor.
Bu başlığı ekleyen `preview-format` da örnek içindeki yerel bir
kütüphane. Onu yalnız önizleme çağırıyor. Buna rağmen iki program da
`card-tools` paketinin hedefleri olduğu için biçimlendirici bağımlılığı
paketin ortak `[dependencies]` bölümünde yer alıyor. Aşağıdaki ağaç
paketin bu bağımlılığını gösterir. [Paket bildirimi](../examples/proje-yapisi/02-kutuphane/Cargo.toml) · [Cargo Book · Çalıştırılabilir hedefler](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#binaries)

```sh
cargo tree --offline --locked --manifest-path examples/proje-yapisi/02-kutuphane/Cargo.toml -p card-tools
```

`--bin` aynı paket içindeki çalıştırılabilir hedefi seçer. `-p` ise
Cargo paketini seçer. Bu ayrım bir sonraki aşamada, düz çıktı ile
önizleme ayrı paketlere taşındığında görünür.
[Cargo Book · Hedefler](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#binaries) · [Paket seçimi](https://doc.rust-lang.org/cargo/reference/workspaces.html#package-selection)

## 3 · Bağımlılıkları ayrı paketlerde tanımla

Yeni gereksinim şu. Düz çıktı programı kendi başına derlenebilsin ve
önizleme biçimlendiricisini bağımlılık olarak almasın. Ortak kart işini
`card-core` paketine taşı. Düz çıktı için `card-cli`, önizleme için
`card-preview` ayrı paketler olsun. Workspace kökünde üye paketleri
listele. Her paketin bağımlılıklarını kendi `Cargo.toml` dosyası
belirlesin. [Workspace bildirimi](../examples/proje-yapisi/03-workspace/Cargo.toml)

Kök bildirim üye paketleri bir araya getiriyor.

```toml
[workspace]
members = ["card-core", "card-cli", "card-preview", "preview-format"]
resolver = "3"
```

| Paket | Yerel bağımlılıkları |
| --- | --- |
| `card-core` | Standart kütüphane yeterli |
| `card-cli` | `card-core` |
| `card-preview` | `card-core` ve `preview-format` |
| `preview-format` | Standart kütüphane yeterli |

**Yeni bir mimari sınırı ihtiyaç ortaya çıktığında kurmak.** Martin önce
sorumlulukları ayrılmış basit bir yapı kurmayı, yeni altyapıyı çalışan
kullanım senaryoları gerektirdikçe eklemeyi anlatıyor. Burada iki program
ortak kodu çağırmaya başladığında kütüphane hedefi yeterliydi. Paket
ayrımını gerekçelendiren yeni ihtiyaç, düz programın önizlemeye özgü
bağımlılığı almadan derlenmesi. Workspace paketleri birlikte yönetir.
Bağımlılık ayrımını her paketin kendi bildirimi sağlar.
[*Clean Code*, 11. bölüm, “Test Drive the System Architecture” ve “Optimize Decision Making”, s. 166–168](https://www.oreilly.com/library/view/clean-code-a/9780136083238/chapter11.xhtml) · [Cargo Book · Paket seçimi](https://doc.rust-lang.org/cargo/reference/workspaces.html#package-selection)

```sh
cargo tree --offline --locked --manifest-path examples/proje-yapisi/03-workspace/Cargo.toml -p card-cli
cargo tree --offline --locked --manifest-path examples/proje-yapisi/03-workspace/Cargo.toml -p card-preview
cargo run --quiet --offline --locked --manifest-path examples/proje-yapisi/03-workspace/Cargo.toml -p card-cli
cargo run --quiet --offline --locked --manifest-path examples/proje-yapisi/03-workspace/Cargo.toml -p card-preview
cargo test --offline --locked --manifest-path examples/proje-yapisi/03-workspace/Cargo.toml --workspace
```

İlk ağaçta `card-core` var. İkinci ağaçta `preview-format` da var.
İki programın çıktısı bir önceki aşamayla aynı. Böylece bağımlılık
ayrımını ağaçtan, kullanıcıya görünen davranışı da programı çalıştırıp
testleri koşarak kontrol edebilirsin.

## Ek okuma · Statik site üreticisi

Let’s Get Rusty’nin *3 way to structure your Rust projects* videosu aynı
yapısal kararları bir Markdown site üreticisi üzerinden gösteriyor.
Kod rehberi, bu uygulamadaki dosya ve bağımlılık değişikliklerini
zaman bağlantılarıyla açıklıyor. [Statik site üreticisinin kodu](videodaki-site-uretici.md)
