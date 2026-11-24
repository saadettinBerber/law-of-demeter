# Rust CLI

Rust seçimi açıktır; argümansız komut mevcut Java/Maven akışını kullanır.

```bash
law-of-demeter --rust /path/to/Cargo.toml --package shop \
  --only 'shop/shop::PaymentService#on_parameter' --show-allowed
```

`--package` zorunludur ve tek normal lib hedefi olan Cargo paketini seçer. Binary,
test ve örnek hedefleri bu dilimde analiz edilmez. Rust yolunda JDK veya Maven
aranmaz; Java'nın `--source`, `--classpath`, `--jdk-home`, `--release` seçenekleriyle
birlikte kullanılması kullanım hatasıdır. Seçenek sırası bağımsızdır; değerler
`--flag=value` biçiminde de verilir. Boşluk içeren yollar tırnak içine alınır.

## Cargo yapılandırması

- `--features payments,audit`: Özellik listesi; virgül veya boşlukla ayrılır, tekrarlanabilir.
- `--no-default-features`: Varsayılan özellikleri kapatır.
- `--target aarch64-unknown-linux-gnu`: Cargo hedef platformunu seçer. Gerekli platform ve bağımlılıklar önceden hazır olmalıdır.
- `--target-dir /writable/analysis`: Analiz/build çıktısı. Göreli yol çalışma dizinine göre mutlaklaştırılır. Varsayılan gerçek workspace kökündeki `target/law-of-demeter` dizinidir.

PATH üzerinde Rust/Cargo 1.99.1, aynı toolchain'in rust-src bileşeni ve
`libexec/rust-analyzer-proc-macro-srv` dosyası gerekir. Otomatik kurulum veya
global Cargo ayarı değişikliği yapılmaz. Metadata ve backend build işlemleri
`--locked --offline` kullanır: geçerli `Cargo.lock` ve hazırlanmış bağımlılık
cache'i gereklidir. Build script'leri ve proc macro'lar kullanıcının kodunu
çalıştırabilir. Host önkoşullarını istemeyen akış [Docker belgesindedir](container.md).

## Seçim ve rapor

`--only` tekrarlanabilir. Okunabilir tam hedef `package/target::module::Type#method`
biçimindedir. `Type#method`, `module::Type#method` veya `target::module::Type`
yalnız gerçek proje tipi kataloğunda tek eşleşme varsa kabul edilir. Belirsizlik
adaylarıyla açık hata verir; benzer ad seçilmez. Cargo paket kimliğini içeren tam
kimlik de kabul edilir; bu kimliğin `#` karakterleri korunur, son metot ayırıcısı
ayrı çözülür. Seçim bütün workspace tiplerinin yüklenmesini daraltmaz.

Çağrı satırı `file:line  package/target::module::Type#method  call()  Verdict: reason`
biçimindedir. `--show-allowed` izinli çağrıları ve nedenlerini de gösterir. Analiz
sorunları Read, Parse, TypeResolution, TargetResolution, Origin, Expansion,
Unsupported, Compilation veya Build türüyle; konum, bilinen gövde sahibi ve
mesajla yazılır. Gövde sahibi bilinmiyorsa proje kapsamı gösterilir. Kaynak ve tip
okuma hataları ayrıca listelenir. İç kimlikler kararlarda korunur; yalnız sunumda
Cargo metadata ile okunabilir ad kullanılır.

| Çıkış | Sonuç |
|---|---|
| 0 | Tam analiz, ihlal/hibrit yok. Geçerli boş lib için açıkça 0 metot/0 çağrı. |
| 1 | Tam analiz, ihlal veya hibrit var. |
| 2 | Kullanım/başlatma/manifest hatası veya tam analizde gerçekten eşleşmeyen seçim. |
| 3 | Başlamış fakat eksik analiz; bilinen bulgular yine raporlanır. |

Analiz sorunları, kaynak/tip okuma hataları veya çözülemeyen çağrı varsa 3,
ihlalden önceliklidir. Eksik gövde boş metot sayılmaz. Eksiklik varken unmatched
seçim kesin yokluk sayılmaz; seçilen sayımlar korunur ve gerekirse bütün projeden
seçim doğrulamasını engelleyen sorunlar ayrıca gösterilir. Yükleme hiç başlamadan
başarısızsa açık nedeniyle 2 döner. Java'nın mevcut rapor/çıkış davranışı korunur.

## Kapsam

Bu, bütün Rust dilini destekleyen bir denetleyici değildir. Doğrudan parametre ve
self çağrıları, literal argümanlar ve normal metot zincirleri ilk dilimdedir.
Gerçek çağrı hedefi ve tipleri Rust adaptöründen gelir. S2 parametre çağrısı ile
S5 dönen cüzdan zincirinin izin/ihlal ayrımı terminale kadar sınanır.

Alan kökeni, yereller/oluşturma, genel kontrol akışı, bağımsız fonksiyonlar,
closure/async, trait/generic dispatch, standart koleksiyon politikaları ve genel
makro traversal'ı ilk dilimde tamamlanmamıştır. Etkin desteklenmeyen gövdeler
eksik raporlanır. `#[cfg]` ile dışlanan kod sorun sayılmaz; `cfg!` çalışma zamanı
dalını derlemeden çıkarmaz. Getter/setter/hibrit sınıflandırmasının genel Rust
politikası bu CLI değişikliğinde uygulanmaz. Adaptörün ayrıntılı kapsamı
[ilk kaynak dilimindedir](rust-source-slice.md).

## Doğrulama kaydı

24 Kasım 2026, macOS ARM64, Rust/Cargo 1.99.1; `target/rust-cli-journey` mutlak
çıktı yoluyla, dev/test debug 0, incremental 0 ve jobs 1 kullanıldı. Java örnekleri
mevcut JDK 21 ve Maven'i yalnız süreç ortamından aldı. Host toolchain veya global
ayar değiştirilmedi.

| Kontrol | Gerçek sonuç |
|---|---|
| `cargo test --workspace --offline --locked -- --test-threads=1` | 1283 geçti, 0 hata, 8 ignored. Yeni CLI bölümü 21 gerçek process testi. |
| `cargo fmt --all --check` | exit 0. |
| `cargo clippy --workspace --all-targets --offline --locked -- -D warnings` | exit 0. |
| `cargo test -p law-of-demeter-cli --test examples --offline --locked -- --ignored --test-threads=1` | 2/2 geçti; mevcut Java raporları korundu. |
| `docker build --progress=plain -t law-of-demeter:rust-cli .` | exit 0; runtime Rust/Cargo 1.99.1, sysroot `/opt/rust`. |
| `CONTAINER_IMAGE=law-of-demeter:rust-cli cargo test -p law-of-demeter-cli --test container --test rust_binary --offline --locked -- --ignored --test-threads=1` | 6/6 geçti: Java 0/1/2, Rust 0/1/3. |

Sekiz ignored testin tamamı yukarıdaki açık koşularda çalıştırıldı. Linux/ARM64
container akışı gerçek imajda sınandı; başka mimarilerde aynı sonuç iddia edilmez.

[Process testleri](../crates/cli/tests/rust_binary.rs) küçük, gerçek Cargo
kütüphaneleri oluşturup binary→Cargo→adaptör→CheckProject→terminal sınırını
çalıştırır. Geçersiz JAVA_HOME ve javac içermeyen PATH ile S2 başarısı; S5'in tam
dosya/satır/gerekçesi; boş lib; seçim; cfg/özellik/paket/target; okuma, parse, tip,
hedef, build ve desteklenmeyen gövde sonuçları sınanır. Stub veya snapshot bu
sınırın yerine kullanılmaz. Mevcut `examples/rust-scenarios` da gerçek CLI'da
S2/S5 seçimiyle çalıştırıldı: `core.rs:13` izinli `debit`, `core.rs:29` izinli
`wallet` ve tek ihlal `debit`; 2 metot, 3 çağrı, 0 analiz sorunu, exit 1.

İlk argüman kırmızıları eksik Rust seçeneklerini ve opaque kimliğin eski Java
ayırıcısına düşmesini gösterdi. İlk binary kırmızısı Rust komutunun Maven kökü
aramasıydı. Eksik seçili gövde testi, tüm proje sayımlarına genişleyen raporu
yakaladı; düzeltme seçilen sayımları korur. İlk başarılı olmayan koşulardaki iki
gerekçe beklentisi mevcut İngilizce rapor sözlüğüne düzeltildi; bunlar yeni
davranış kanıtı sayılmadı.

İlk çalıştırmada yeşil olan sonuç testleri geçici bozulmayla denetlendi: eksiklik
önceliği 3'ten 1'e çevrilince ihlal+issue testi gerçek çıkış assertion'ında
kırıldı; son çıkış kodunu değiştirmek 21 process testinin tamamını assertion'da
kırdı. Bozulmalar geri alındı ve son workspace koşusu yeşil geçti. Eski Docker
imajı üç Rust testinde bilinmeyen `--rust` hatasıyla kırıldı; son imajda hepsi
geçti. Log'lar `target/rust-cli-journey` altındadır. Test okunabilirliği ayrıca
salt okunur geçişte incelendi; bağımsız değişiklik incelemesi bu kaydın dışında.
