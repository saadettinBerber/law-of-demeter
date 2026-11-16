# Örnek projeler

Araç iki Maven projesinde uçtan uca denenir.

- **`shop`:** Tek modüllü proje. [Senaryoların](../SCENARIOS.md) örnekleri burada gerçek kod olarak durur. [README](../README.md)'deki her ihlalin düzeltilmiş hâli de vardır.
- **`warehouse`:** Çok modüllü proje (`catalog` ve `orders`). `orders`, `catalog` tiplerini kullanır; araç modül sınırını aşan çağrıları da yargılar.

## Çalıştırma

Maven, JDK ve ısınmış `~/.m2` gerekir; derleme çevrimdışı yapılır. Komut repo kökünde çalışır:

```bash
cargo test -p law-of-demeter-cli --test examples -- --ignored
```

Test her projeyi derler ve aracı `--show-allowed` ile çalıştırır.

## Beklenen rapor

Her projede bir `expected-report.txt` vardır. Test, çıktıyı bu dosyayla satır satır karşılaştırır; çıkış kodu da rapora uymalıdır. Örnek kodu değişince rapor da güncellenir.

[SCENARIOS.md](../SCENARIOS.md) her senaryodan bu projelerdeki bir satıra bağlanır. Bağlantı, raporda bulgusu olan satırı göstermelidir. Maven gerektirmeyen şu test bunu denetler:

```bash
cargo test -p law-of-demeter-cli --test scenario_links
```
