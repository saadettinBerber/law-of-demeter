# Host toolchain kurmadan Java ve Rust analizi

Docker CLI ve çalışan bir Linux container engine gerekir. Host makinede Rust,
JDK veya Maven gerekmez. Komutlar macOS/Linux kabuğu içindir.

Repo kökünde imajı derle:

```bash
docker build --tag law-of-demeter:local .
```

Çalışma imajı Java 21 JDK'sını, Maven 3.9.11'i, CLI'ı ve taşınabilir Rust/Cargo
1.99.1 toolchain'ini taşır. Eşleşen rust-src ile proc-macro sunucusu aynı
toolchain'den gelir; Rust için host kurulumu yapılmaz. Build script ve proc macro
derlemeleri için C/C++ derleyicisi de bulunur. JDK'nın `lib/modules` ve `lib/ct.sym` dosyaları tip
okumak için gereklidir; yalnızca JRE yeterli değildir. Daha eski Java sürümleri
`--release` ile seçilebilir; Java 21'den yeni bir JDK bu imaja dahil değildir.

Argümansız komut Java/Maven analizini korur. Rust yalnız `--rust` ile seçilir;
desteklenen dar kaynak kapsamı [Rust CLI belgesinde](rust-cli.md) açıklanır.

## Maven projesini hazırla

Denetlenecek Maven projesinin köküne geç. Derleme ve classpath üretimi yazma
izni ister; `--user` üretilen dosyaların host kullanıcısına ait kalmasını sağlar:

```bash
docker run --rm \
  --user "$(id -u):$(id -g)" \
  --mount "type=bind,src=$PWD,dst=/workspace" \
  --entrypoint mvn \
  law-of-demeter:local \
  -B -q -Dmaven.repo.local=/workspace/.container-m2 \
  compile dependency:build-classpath -Dmdep.outputFile=target/classpath.txt
```

Bu komut proje ağacında `target` ve `.container-m2` oluşturur; Maven proje
eklentileri de çalışır. Bağımlılıklar ilk çalıştırmada indirilir. `.container-m2`
yerel önbellektir; sürüm kontrolüne eklenmemelidir. İmajın JDK 21'i projeyi
derleyebilmelidir. Projenin özel ayarları, depoları ve kimlik bilgileri imaja
kopyalanmaz.

Classpath konteyner içinde üretilir; jar yolları `/workspace/.container-m2`
altını gösterir. Host'ta üretilmiş `target/classpath.txt` içindeki mutlak yollar
bu mount düzeninde genellikle bulunmaz. Derleme ve analiz aynı `/workspace`
yolunu kullanmalıdır.

## Analizi çalıştır

Aynı proje kökünde, kaynakları ve derlenmiş sınıfları salt okunur bağla:

```bash
docker run --rm \
  --user "$(id -u):$(id -g)" \
  --mount "type=bind,src=$PWD,dst=/workspace,readonly" \
  law-of-demeter:local
```

CLI, `src/main/java`, `target/classes` ve `target/classpath.txt` dosyalarını
mevcut Maven keşfiyle bulur. Analiz projeye yazmaz. İmaj varsayılan olarak root
olmayan kullanıcıyla çalışır; `--user` host'taki erişim izinlerini korur.

CLI seçenekleri imaj adından sonra verilir. Yollar host'a değil konteynere
aittir; çoklu classpath ayırıcısı `:` olur. Örneğin bütün kaynakları okuyup yalnız
bir metodu raporlamak için:

```bash
docker run --rm \
  --user "$(id -u):$(id -g)" \
  --mount "type=bind,src=$PWD,dst=/workspace,readonly" \
  law-of-demeter:local --only 'shop.OrderService#pay' --show-allowed
```

Maven kullanmayan projelerde derlenmiş sınıflar hazırlanır, ardından
`--source src --classpath out` gibi açık yollar verilir. `--source` kullanımı
Maven keşfini kapatır; bütün kaynak kökleri ve gerekli classpath girişleri açıkça
verilmelidir. Maven'in bu imajdaki yerine `--entrypoint javac` ile doğrudan Java
derleyicisi de çalıştırılabilir.

Rapor stdout'a, bağlam ve sonuç stderr'e yazılır. `docker run` CLI çıkış kodunu
aktarır: `0` uygun, `1` ihlal/hibrit, `2` kullanım veya başlatma hatasıdır. Mevcut
CLI'da okunamayan kaynaklar ve tipler raporlanır fakat çıkış kodunu değiştirmez;
başarılı kod tek başına eksiksiz analiz kanıtı değildir. `update` yerine kaynak
checkout'u güncelleyip imajı yeniden derle.

## Gerçek analiz kontrolü

Repo kökünde, imaj derlendikten sonra:

```bash
cargo test -p law-of-demeter-cli --test container -- --ignored
cargo test -p law-of-demeter-cli --test rust_binary rust_container -- --ignored
```

Bu geliştirici kontrolü macOS/Linux host'ta Rust ve Cargo gerektirir; yukarıdaki
imajla analiz akışı bunları gerektirmez. Normal test koşusunda container testleri
ignored kalır; açıkça `--ignored` ile çalıştırılır. Her test
`target/container-rust-smoke` altında boşluk içeren, sonunda temizlenen ayrı bir
geçici proje oluşturur, `Order` ve `Wallet` örneğini imajdaki `javac` ile derler
ve host dosya sahipliğini denetler. Salt okunur mount üzerinde uygun
çağrının `0`, yabancı cüzdana uzanan çağrının gerekçesiyle `1`, eşleşmeyen seçimin
`2` döndürmesini; çözülemeyen çağrı, kaynak ve tip okuma hatası olmamasını sınar.
Başka imaj adı için `CONTAINER_IMAGE` ortam değişkenini kullan:

```bash
CONTAINER_IMAGE=law-of-demeter:local cargo test -p law-of-demeter-cli --test container -- --ignored
```

Kontrol yerel imajı kullanır. Engine veya imaj bulunamazsa hata verir; analiz
testleri geçmiş sayılmaz.

## Rust kütüphanesini hazırla ve denetle

Projenin geçerli `Cargo.lock` dosyası bulunmalıdır. Kaynaklar salt okunur kalır;
Cargo cache'i ve analiz/build çıktısı ayrı yazılabilir bağlanır. İlk kez
bağımlılıkları indirmek için proje kökünde:

```bash
mkdir -p .container-cargo .container-analysis
docker run --rm \
  --user "$(id -u):$(id -g)" \
  --mount "type=bind,src=$PWD,dst=/workspace,readonly" \
  --mount "type=bind,src=$PWD/.container-cargo,dst=/cargo-cache" \
  --env CARGO_HOME=/cargo-cache \
  --entrypoint cargo \
  law-of-demeter:local fetch --locked
```

Bu hazırlık ağ erişimi kullanabilir; özel depo ve kimlik bilgileri ayrıca
sağlanır. Analiz backend'i locked/offline çalıştığı için cache, seçilen hedef
platform dahil gereken bağımlılıkları içermelidir. Yerel path bağımlılıkları da
konteynerde aynı manifest düzeninde bulunmalıdır. Cache ve çıktı dizinleri yerel
dosyalardır; sürüm kontrolüne eklenmemelidir.

```bash
docker run --rm \
  --user "$(id -u):$(id -g)" \
  --mount "type=bind,src=$PWD,dst=/workspace,readonly" \
  --mount "type=bind,src=$PWD/.container-cargo,dst=/cargo-cache" \
  --mount "type=bind,src=$PWD/.container-analysis,dst=/analysis" \
  --env CARGO_HOME=/cargo-cache \
  law-of-demeter:local --rust Cargo.toml --package shop \
  --target-dir /analysis --only 'shop/shop::PaymentService#on_parameter' --show-allowed
```

Salt okunur kaynakta `--target-dir /analysis` gerekir; varsayılan workspace
`target/law-of-demeter` yolu yazılabilir olmayabilir. İmaj root olmayan UID ile
çalışır; `--user` host dosya izinlerini korur. Varsayılan Cargo cache'i
`/tmp/law-of-demeter-cargo` yazılabilirdir fakat kalıcı cache için yukarıdaki mount
kullanılır. Cargo build script'leri ve proc macro'lar analiz sırasında kullanıcı
kodunu çalıştırabilir.

Rust için `0` tam temiz, `1` tam ihlal/hibrit, `2` kullanım/başlatma veya kesin
eşleşmeme, `3` eksik analizdir; bilinen bulgular 3 durumunda da yazılır. Üç Rust
container testi gerçek binary→Cargo→adaptör→CheckProject yolunda S2/S5 ve eksik
gövde sonuçlarını, boşluklu yolları, root olmayan kullanıcıyı, yazılabilir cache
ve çıktıyı, salt okunur kaynağı sınar. Mevcut üç Java smoke testi aynı imajda
0/1/2 sözleşmesini denetler.

## Taban imajları

Dockerfile, Rust 1.99.1 / Debian Bookworm ve Maven 3.9.11 / Temurin JDK 21 /
Ubuntu Noble imajlarını manifest digest'leriyle sabitler. `Cargo.lock` derleme
sırasında `--locked` ile kullanılır. Digest sabitlemek güvenlik güncellemelerini
otomatik almaz; yeni sürüme geçerken digest'leri yenileyip smoke kontrolünü
tekrar çalıştır. Bütün çıktının bit düzeyinde yeniden üretilebilirliği iddia
edilmez.

Resmî kaynaklar: [Rust imajı](https://github.com/rust-lang/docker-rust),
[Maven imajı](https://github.com/carlossg/docker-maven),
[Docker ENTRYPOINT ve CMD](https://docs.docker.com/reference/dockerfile/#entrypoint),
[Docker bind mount](https://docs.docker.com/engine/storage/bind-mounts/).
