# Statik site üreticisinin kodu

`sitegen`, bir klasördeki Markdown dosyalarından HTML sayfaları üreten
örnek bir uygulama. Let’s Get Rusty’nin *3 way to structure your Rust
projects* videosunda bu uygulama modüllere, bir kütüphaneye ve sonunda
ayrı Cargo paketlerine bölünüyor. [3 way to structure your Rust projects](https://www.youtube.com/watch?v=mkW3dTxBmm0)

Aşağıdaki kod kesitleri bu rehber için yazıldı. Her girdi ayrı bir çıktı
dosyası üretiyor ve dosya hataları çağırana iletiliyor. Sunucu bölümünde
izleme ve sunma akışı taslak olarak gösteriliyor. Aynı yapı kararlarını
çevrimdışı denemek için kart listesi örneğini kullanabilirsin.
[Kart listesi rehberi](proje-yapisi.md)

| Kod adımı | Videodaki başlangıcı |
| --- | --- |
| [Tek dosya](#0--tek-dosya) | [0:19](https://www.youtube.com/watch?v=mkW3dTxBmm0&t=19s) |
| [Modüller](#1--ayrıştırma-ve-şablon-ayrı-modüllere-taşınır) | [0:48](https://www.youtube.com/watch?v=mkW3dTxBmm0&t=48s) |
| [Ortak kütüphane](#2--oluşturma-akışı-librs-üzerinden-açılır) | [1:24](https://www.youtube.com/watch?v=mkW3dTxBmm0&t=84s) |
| [İkinci program](#3--ikinci-çalıştırılabilir-program-buildi-çağırır) | [2:23](https://www.youtube.com/watch?v=mkW3dTxBmm0&t=143s) |
| [Paket sınırına ihtiyaç](#4--tek-paketin-bağımlılık-sınırı-görünür-olur) | [2:42](https://www.youtube.com/watch?v=mkW3dTxBmm0&t=162s) |
| [Workspace](#5--workspace-ile-manifestler-ayrılır) | [3:02](https://www.youtube.com/watch?v=mkW3dTxBmm0&t=182s) |
| [Yapıyı seçmek](#6--yapıyı-seçerken-hangi-soruya-bakılır) | [3:36](https://www.youtube.com/watch?v=mkW3dTxBmm0&t=216s) |

## 0 · Tek dosya

`src/main.rs` şablonu bir kez okuyor. `content` içindeki her Markdown
dosyası için metni okuyor, HTML gövdesine dönüştürüyor, gövdeyi şablona
yerleştiriyor ve sonucu `public` altına yazıyor. Örneğin
`content/rust.md`, `public/rust.html` oluyor. Şablonda HTML gövdesinin
yerine geçeceği bir `{{body}}` işareti var. [Videoda tek dosya · 0:19](https://www.youtube.com/watch?v=mkW3dTxBmm0&t=19s) · [Markdown dönüştürme işlevi](https://docs.rs/markdown/1.0.0/markdown/fn.to_html.html)

Bu kesiti denemek için `sitegen` adlı bir Cargo paketinin
`[dependencies]` bölümüne `markdown = "1"` ekle. Paket kökünde
`layout.html` dosyasına `<main>{{body}}</main>` yaz ve `content`
dizinine iki Markdown dosyası koy. `cargo run` komutuyla çıktılarını
`public` dizininde görebilirsin.

```rust
use std::{fs, io, path::Path};

fn main() -> io::Result<()> {
    let layout = fs::read_to_string("layout.html")?;
    fs::create_dir_all("public")?;

    for entry in fs::read_dir("content")? {
        let input = entry?.path();
        if !input.is_file() || !input.extension().is_some_and(|ext| ext == "md") {
            continue;
        }

        let source = fs::read_to_string(&input)?;
        let body = markdown::to_html(&source);
        let page = layout.replace("{{body}}", &body);

        if let Some(name) = input.file_name() {
            let output = Path::new("public").join(name).with_extension("html");
            fs::write(output, page)?;
        }
    }

    Ok(())
}
```

**Çalışan davranışı küçük adımlarla korumak.** İlk karşılaştırma için iki
Markdown dosyasından oluşan bir girdi ve bunlara karşılık gelen iki HTML
dosyası kullan. Sonraki aşamada dosyaların adlarını ve içeriklerini aynı
girdiyle yeniden karşılaştır. Martin de `Args` örneğinde mevcut davranışı
kontrol edebildiği için yapıyı küçük parçalar hâlinde değiştiriyor.
[*Clean Code*, 14. bölüm, “On Incrementalism”, s. 212–214](https://www.oreilly.com/library/view/clean-code-a/9780136083238/chapter14.xhtml)

## 1 · Ayrıştırma ve şablon ayrı modüllere taşınır

İki yeni dosya aç. `src/parse.rs` Markdown’ı HTML gövdesine dönüştürüyor.
`src/template.rs` gövdeyi HTML şablonuna yerleştiriyor. Bu kesitte
`wrap` şablonu parametre olarak alıyor, böylece `main` dosyayı bir kez
okuyup her sayfada kullanabiliyor. [Videodaki modül çağrıları · 1:34](https://www.youtube.com/watch?v=mkW3dTxBmm0&t=94s)

`src/parse.rs`

```rust
pub fn render(source: &str) -> String {
    markdown::to_html(source)
}
```

`src/template.rs`

```rust
pub fn wrap(layout: &str, body: &str) -> String {
    layout.replace("{{body}}", body)
}
```

`src/main.rs` içindeki değişiklik

```diff
+mod parse;
+mod template;
+
 use std::{fs, io, path::Path};

@@
-        let body = markdown::to_html(&source);
-        let page = layout.replace("{{body}}", &body);
+        let body = parse::render(&source);
+        let page = template::wrap(&layout, &body);
```

**Tek sorumluluk ilkesi ve birlikte değişen kodu bir arada tutmak.** Markdown
kuralları değişirse `parse.rs`, HTML yerleşimi değişirse `template.rs`
değişir. Dosyaların okunması ve yazılması aynı akışta kalır. Ayrım,
değişiklik nedenlerinin farklı olmasına dayanır.
[*Clean Code*, 10. bölüm, “The Single Responsibility Principle” ve “Cohesion”, s. 138–141](https://www.oreilly.com/library/view/clean-code-a/9780136083238/chapter10.xhtml)

## 2 · Oluşturma akışı lib.rs üzerinden açılır

Bir önceki `src/main.rs` dosyasını `src/lib.rs` olarak taşı. `mod` ve
`use` bildirimleri bu dosyada kalıyor. Eski `main` işlevinin imzasını
değiştir. Gövde aynı dosyaları üretmeye devam ediyor. [Videoda kütüphaneyi çıkarma · 1:24](https://www.youtube.com/watch?v=mkW3dTxBmm0&t=84s)

```diff
-fn main() -> io::Result<()> {
+pub fn build() -> io::Result<()> {
```

Yeni `src/main.rs`

```rust
use sitegen::build;
use std::io;

fn main() -> io::Result<()> {
    build()
}
```

**Ortak davranışın tekrarını önlemek ve başlangıç kodunu ortak işten
ayırmak.** Sayfanın nasıl üretileceği `build` içinde bir kez tanımlanır.
Komut satırı giriş noktası onu çağırır. Bir entegrasyon testi de
`sitegen::build` işlevini içe aktarıp aynı üretim akışını kullanabilir.
[*Clean Code*, 11. bölüm, “Separation of Main”, s. 155](https://www.oreilly.com/library/view/clean-code-a/9780136083238/chapter11.xhtml) · [12. bölüm, “No Duplication”, s. 173–174](https://www.oreilly.com/library/view/clean-code-a/9780136083238/chapter12.xhtml)

## 3 · İkinci çalıştırılabilir program build’i çağırır

`src/bin/serve.rs` eklenince Cargo aynı paketten ikinci bir
çalıştırılabilir program üretir. Videodaki akış taslağında `watch`,
içerik değişikliklerinde `build` işlevini yeniden çağırır. `serve`,
`public` altındaki üretilmiş dosyaları sunar. Aşağıdaki taslak bu üç
sorumluluğun bağlantısını gösteriyor. [Videoda sunucu giriş noktası · 2:38](https://www.youtube.com/watch?v=mkW3dTxBmm0&t=158s) · [Cargo Book · Çalıştırılabilir hedefler](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#binaries)

```text
// src/bin/serve.rs · akış taslağı
use sitegen::build;

fn main() {
    build();                   // İlk çıktıyı oluştur.
    watch("content", build);   // İçerik değiştiğinde yeniden oluştur.
    serve("public");           // Üretilen dosyaları sun.
}
```

**Kurulumu ortak davranıştan ayırmak.** Dosya izleme ve sunucuyu bağlama
işi ikinci giriş noktasına ait. Üretim kuralı komut satırı aracının
çağırdığı kütüphanede kalıyor. Örneğin şablon ekleme kuralı değiştiğinde,
iki program da yeni kuralı aynı `build` üzerinden alıyor.
[*Clean Code*, 11. bölüm, “Separate Constructing a System from Using It”, s. 154–155](https://www.oreilly.com/library/view/clean-code-a/9780136083238/chapter11.xhtml)

## 4 · Tek paketin bağımlılık sınırı görünür olur

Bu aşamada iki çalıştırılabilir hedef de tek `Cargo.toml` altında.
Videoda gösterilen dört bağımlılık bu paketin ortak listesinde. Aşağıdaki
kesitteki sürüm aralıkları öğretici için seçildi. [Videodaki tek paket bildirimi · 2:48](https://www.youtube.com/watch?v=mkW3dTxBmm0&t=168s)

```toml
[dependencies]
markdown = "1" # Ortak site üretimi
clap = "4"     # Komut satırı seçenekleri
axum = "0.8"   # Önizleme sunucusu
tokio = "1"    # Sunucunun eşzamansız çalışma ortamı
```

`cargo build --bin sitegen` çalıştırılabilir hedefi seçer. Manifestteki
zorunlu bağımlılıklar hâlâ `sitegen` paketine aittir. Paket sınırını
ayırma gerekçesi burada ortaya çıkıyor. Komut satırı aracını sunucudan
bağımsız derlemek istiyoruz. Bu farkı yerel kart örneğindeki
`cargo tree` komutuyla da görebilirsin.
[Cargo Book · Çalıştırılabilir hedefler](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#binaries) · [Tek paket ağacını dene](proje-yapisi.md#2--ortak-işi-kütüphane-hedefinden-çağır)

**Kararı ihtiyaç belirginleşince vermek.** Martin, karar anına kadar
edinilen bilginin daha iyi bir tasarım seçmeyi sağladığını anlatıyor.
Burada yeni paket sınırının somut gerekçesi bağımsız derleme ihtiyacı.
[*Clean Code*, 11. bölüm, “Optimize Decision Making”, s. 167–168](https://www.oreilly.com/library/view/clean-code-a/9780136083238/chapter11.xhtml)

## 5 · Workspace ile manifestler ayrılır

Videoda workspace üyelerinin dizinleri `core`, `cli` ve `serve`.
Aşağıdaki kesitler bu üç dizindeki bağımlılık ayrımını örnekliyor.
Paketlere bu öğreticide `sitegen-core`, `sitegen-cli` ve `sitegen-serve`
adlarını veriyoruz. [Videodaki workspace bildirimi · 3:20](https://www.youtube.com/watch?v=mkW3dTxBmm0&t=200s) · [Cargo Book · Workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html)

```toml
# Kök Cargo.toml
[workspace]
members = ["core", "cli", "serve"]
resolver = "3"
```

```toml
# core/Cargo.toml
[package]
name = "sitegen-core"
version = "0.1.0"
edition = "2024"

[dependencies]
markdown = "1"
```

```toml
# cli/Cargo.toml
[package]
name = "sitegen-cli"
version = "0.1.0"
edition = "2024"

[dependencies]
sitegen-core = { path = "../core" }
clap = "4"
```

```toml
# serve/Cargo.toml
[package]
name = "sitegen-serve"
version = "0.1.0"
edition = "2024"

[dependencies]
sitegen-core = { path = "../core" }
axum = "0.8"
tokio = "1"
```

İki giriş noktası da artık `use sitegen_core::build;` ile aynı
kütüphaneyi çağırır. `cargo tree -p sitegen-cli` seçilen paketin
bağımlılıklarını gösterir. Bu yapıda çekirdek de Axum kullanmadığı için
komut satırı aracının ağacında Axum yer almaz. Aynı ayrımı depodaki
çevrimdışı örnekte `card-cli` ve `card-preview` üzerinde deneyebilirsin.
[Cargo Book · Paket seçimi](https://doc.rust-lang.org/cargo/reference/workspaces.html#package-selection) · [Yerel workspace örneği](proje-yapisi.md#3--bağımlılıkları-ayrı-paketlerde-tanımla)

**Yeni mimari sınırı çalışan ihtiyaç gerektirdiğinde kurmak.** Modüller
kodun yerini, kütüphane kodu kimlerin çağırabildiğini, ayrı paketler ise
hangi bağımlılıklarla derlendiğini düzenledi. Son adımda üretim işlevi
aynı kaldı. Değişen şey derlemenin paket sınırı.
[*Clean Code*, 11. bölüm, “Test Drive the System Architecture”, s. 166–167](https://www.oreilly.com/library/view/clean-code-a/9780136083238/chapter11.xhtml)

## 6 · Yapıyı seçerken hangi soruya bakılır?

Videonun son bölümü kararları topluyor. Dosya içinde işlerin yerini
bulmak zorlaştığında önce modül sınırını düşün. Aynı işi iki giriş
noktası kullanacaksa kütüphane hedefini değerlendir. Programların
bağımlılıkları birbirinden ayrılmalıysa paket sınırı kur. Her kararın
gerekçesi bir önceki yapıda ortaya çıkan ihtiyaç.
[Videoda yapı seçimi · 3:36](https://www.youtube.com/watch?v=mkW3dTxBmm0&t=216s) · [*Clean Code*, 12. bölüm, “Minimal Classes and Methods”, s. 176](https://www.oreilly.com/library/view/clean-code-a/9780136083238/chapter12.xhtml)
