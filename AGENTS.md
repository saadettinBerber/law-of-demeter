# law-of-demeter

Java kodunda Law of Demeter ihlallerini bulan Rust aracı. Kural [README](README.md)'de, beklenen davranış [senaryolarda](SCENARIOS.md), yapılacaklar [karar listesinde](decision.md) durur.

Önce bu üç belgeyi oku. Sıradaki işi karar listesinin tamamlanmamış satırlarından belirle; ilgili senaryoyu ve mevcut kodu kontrol et.

## Mimari

---

**Altıgen mimari. Bağımlılık yalnızca içe doğru akar: cli → adapter → core.**

- **core:** Domain ve application.

  - Kurallar domain'de, sade Rust nesnelerinde durur. Domain dosya okumaz, Java ayrıştırmaz ve dış crate kullanmaz.

  - Application kullanım senaryolarını yürütür; dış ihtiyaçları port'larla ister.

- **Adapter crate'leri:** Port'ları gerçek teknolojiye bağlar ve iş kuralı taşımaz.

  - `class-file`: Derlenmiş Java sınıflarından tip şeklini okur.

  - `java-source`: Java kaynak kodunu ayrıştırır.

- **cli:** Nesneleri birbirine bağlar ve aracı başlatır.

- Her katman ayrı bir crate'tir; bağımlılık yönünü derleyici korur.

- Tip şekli (`TypeShape`), `.class` okuyucu ile domain arasındaki sözleşmedir. Değiştirmen gerekirse yalnızca ekleme yap.

## Kapsam

---

**Araç herhangi bir Java projesinde çalışır; repo belirli bir projeye ait bilgi taşımaz.**

- Belgelere, testlere, test verilerine ve commit mesajlarına gerçek bir projenin sınıf, paket ya da dosya adlarını ve bulgu sayılarını yazma.

- Örneklerde genel adlar kullan: `Order`, `Customer`, `Wallet`.

## Komutlar

---

**Komutlar repo kökünde çalışır.**

- Test: `cargo test --workspace`

- Biçim: `cargo fmt --all --check`

- Lint: `cargo clippy --workspace --all-targets -- -D warnings`

## Test Driven Development

---

**Davranış değişikliği testle başlar.**

1. Davranışı anlatan testi yaz ve kırmızı olduğunu gör. Test, derleme hatası ya da yanlış kurulum yüzünden değil, beklenen davranışın yokluğu yüzünden kırılmalı.

2. Testi geçirecek en az kodu yaz.

3. Testler yeşilken refactor et.

- **Her test:**

  - Kur, işlet, denetle düzenindedir. Bölümler boş satırla ayrılır; bölüm yorumu yazılmaz.

  - Tek bir davranışı sınar ve adı o davranışı söyler.

  - Çağıranın kullandığı arayüzden sınar; özel yardımcıları ve iç yapıyı sınamaz.

  - Beklenen değeri implementasyondan bağımsız yazar.

- **İlk çalıştırmada yeşil olan test:** Kodu geçici olarak bozarak testin hatayı yakaladığını göster, sonra kodu geri al.

- **Dış kütüphane:** Davranışını kullanmadan önce learning test ile sına.

## Kod

---

**Niyeti adlar ve yapı anlatsın.**

- Kodda yorum yazma.

- Kod, test adları ve dal adları İngilizce olsun.

- Gerekmeyen öğeyi `pub` yapma.

- Beklenen hatayı `Result` ile bildir. Üretim yolunda `unwrap` ya da `expect` kullanma.

- Yeni bağımlılığı yalnızca adapter ya da cli crate'ine ekle.

## Commit

---

**Mantıksal parça başına küçük commit.**

- Her commit derlenir; testleri, biçim ve lint denetimini geçer.

- Mesaj tek satır ve Türkçe olsun; Conventional Commits biçiminde, kapsam parantez içinde.

- Gövde ve ortak yazar satırı ekleme. Bir mesajda iki işi birleştirme.
