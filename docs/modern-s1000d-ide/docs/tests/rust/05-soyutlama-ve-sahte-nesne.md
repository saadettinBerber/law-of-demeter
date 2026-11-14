---
title: "Soyutlama ve sahte nesne"
ikon: 🎭
kapak: blue
tarih: 2026-11-08
etiketler: [rust, testler]
ozet: Bir bağımlılığı testte değiştirmenin en kısa yolu çoğu zaman onu soyutlamamak, değeri parametre olarak vermektir. Gerçek uygulama hızlı ve belirlenimciyse test onu kullanır. Dış bir kaynak için küçük bir trait tanımlanır ve testte elle yazılmış bir sahte uygulama kullanılır. Kapalı bir küme enum olarak kalır. &self alan bir trait için kayıt tutan mock nesne iç değişebilirlik ister; Rust Book bunun için RefCell kullanır. Bağımlılıkları programın giriş noktası kurar.
---

:::callout 🎭 blue
**Bir bağımlılığı testte değiştirmenin en kısa yolu çoğu zaman onu hiç soyutlamamaktır: değeri parametre olarak vermek.** Gerçek uygulama hızlı ve belirlenimciyse test onu kullanır. Gerçekten değişken ya da yavaş bir dış kaynak varsa küçük bir trait tanımlanır, testte de elle yazılmış bir sahte uygulama (fake) kullanılır. Kapalı bir küme `enum` olarak kalır. Bağımlılıkları programın giriş noktası kurar ve içeriye verir.
:::

Clean Code serisinin [Soyutlamanın arkasından test](clean_code/07-soyutlamanin-arkasindan-test.html) notu kitabın görüşünü anlattı: değişken bir ayrıntı küçük bir soyutlamanın arkasına alınır, testte yerine sabit davranan bir uygulama konur, ama her sınıfa arayüz yazılmaz. Bu not aynı kararların Rust'ta nasıl verildiğini anlatıyor: değer, trait, `enum`, generic ve `dyn`.

Notun kod örnekleri 8 Kasım 2026'da Rust 1.98.1 ile denendi. Rust Book atıfları kitabın 2 Kasım 2026 tarihli sürümüne dayanıyor.

## Önce değer

Test ikizine (test double) uzanmadan önce sorulacak soru, bağımlılığın gerçekten bir nesne olup olmadığıdır. Çoğu zaman kodun ihtiyacı bir değerdir: bir süre, bir tarih, bir yol, bir metin.

*Software Engineering at Google* kitabının *Test Doubles* bölümünü Andrew Trenk ile Dillon Bly yazdı. Bölümün *How to Decide When to Use a Real Implementation* kısmına göre gerçek uygulama hızlı, belirlenimci ve bağımlılıkları basitse tercih edilir. Yazarların örnekleri değer nesneleridir: bir para tutarı, bir tarih, bir adres, bir liste. Aynı kısmın *Determinism* alt başlığı, belirlenimsizliğin yaygın nedeni olarak dış servislere bağlı, hermetik olmayan kodu gösterir. Sistem saatine dayanan kodu da bir başka örnek olarak anar ve testte belirli bir zamanı sabit döndüren bir ikiz önerir.

Bizce Rust'ta saat için çoğu zaman ikize de gerek kalmaz. Rust geliştirme rehberimizin *Clean Code'u Rust'ta uygulamak* sayfasının 4 Kasım 2026 tarihli sürümü bunun koşulunu da yazar. Bir hesaplama yalnızca tarih istiyorsa tarihi değer olarak geçirmek yeterli olabilir. Saat birden çok işlem boyunca okunacaksa ya da bir iş akışı gerçek depoya bağlanmadan sınanacaksa, küçük bir arayüz işe yarayabilir. Değerle çözülen durumda saati okuyan çağırandır, karar veren fonksiyon yalnızca değeri görür:

```rust
const AUTOSAVE_DELAY: Duration = Duration::from_secs(2);

/// Takes the elapsed time as a value; the caller reads the clock.
pub fn autosave_due(since_last_change: Duration) -> bool {
    since_last_change >= AUTOSAVE_DELAY
}

#[test]
fn autosave_waits_two_seconds() {
    assert!(!autosave_due(Duration::from_millis(1999)));
    assert!(autosave_due(Duration::from_secs(2)));
}
```

Serinin [Her ortamda çalışma](03-her-ortamda-calisma.html) notu aynı düzeni ortam değişkenleri için anlatmıştı: değişken programın sınırında bir kez okunur, içeriye değer olarak verilir.

## Gerçek uygulama yetiyorsa onu kullan

Trenk ve Bly, *Real Implementations* başlığında Google'ın ilk tercihinin gerçek uygulama olduğunu söyler. Gerekçeleri, testin kodu üretimde çalışacağı gibi çalıştırmasıdır. Gerçek uygulamayı tercih eden bu tarza klasik test, mock çerçevelerini tercih eden tarza ise mockist test denir. Yazarlara göre Google'da mockist tarzı ölçeklendirmek zor olmuştur. *Prefer Realism Over Isolation* başlığı bir adım daha atar: gerçek uygulamadaki bir hata testi kırıyorsa bu iyidir, çünkü kod üretimde de çalışmayacaktır.

Google'ın Comprehensive Rust kursu da aynı yöne bakar. Kursun *Android* bölümünün *Testing* kısmındaki *Mocking* sayfası Mockall'ı yaygın kullanılan bir kütüphane olarak tanıtır. Android için önerilen kütüphane olduğu bilgisi sayfanın konuşmacı notlarındadır. Aynı notlar mock kullanımının biraz tartışmalı olduğunu söyler ve iki yanını da sayar. Mock testi bağımlılıklarından tümüyle yalıtır, böylece testler daha hızlı ve kararlı çalışır. Ama yanlış ayarlanabilir ve gerçek bağımlılıktan farklı sonuç döndürebilir. Notlar bu yüzden mümkünse gerçek bağımlılığın kullanılmasını önerir. Örnekleri, bellekte çalışabilen veritabanları ve `localhost` üzerinde rastgele bir portta açılan süreç içi sunuculardır.

## Trait mi, `enum` mı?

Clean Code serisinin 07 notu bir çıkarımı "Bizce" diye işaretlemişti: testte bir bağımlılığın yerine sahtesini koymak, o soyutlamayı uygulayan türlere yeni bir tür eklemektir. Rust'ta bu soru doğrudan trait ile `enum` arasındaki seçime bağlanır.

Rust Book'un *Object-Oriented Programming Features* bölümündeki *Using Trait Objects to Abstract over Shared Behavior* başlığı bu seçimin ölçüsünü verir. Kullanılacak türler derleme anında bilinen sabit bir kümeyse `enum` iyi bir çözümdür. Kütüphaneyi kullananın türlere yenisini ekleyebilmesi gerekiyorsa trait gerekir. Aynı bölümün *Implementing an Object-Oriented Design Pattern* başlığı bir durum desenini trait nesneleriyle kurar. Başlığın *Why Not An Enum?* kısmı `enum`'u da olası bir çözüm sayar, ama bir bedelini yazar: değeri denetleyen her yerde her varyantı ele alan bir `match` gerekir ve bu tekrara dönüşebilir.

Bizce test açısından sonuç şu:

- **Yerine sahtesi konacak bir dış kaynak** trait olur. Test, o trait'in yeni bir uygulamasını yazar.
- **Kapalı bir küme** `enum` kalır. Testin ona yeni bir tür eklemesi gerekmez, testler her işlemi gerçek varyantlarla sınar. `_` kolu olmayan kapsamlı bir `match` kullanıldığında derleyici, yeni bir varyantın işlenmediği her yeri gösterir.

Rust geliştirme rehberimizin aynı sayfası açık-kapalı ilkesini de bu ayrımla anlatmıştı. Değişen şey yeni bir durumsa genişleme noktası `enum`'dur. Değişen şey yeni bir kayıt yöntemiyse genişleme noktası depolama sınırıdır.

## Trait'i tüketicinin ihtiyacından tarif et

Trait bir sınır olarak kurulacaksa, onu kullanan kodun ihtiyacı kadar küçük tutmak gerekir. Rehberimizin aynı sayfası bunu bir kayıt örneğiyle anlatmıştı: uygulama yalnızca bir kaydın alınmasını istiyorsa trait de yalnızca o işlemi anlatır.

Trenk ve Bly'nin *When Should Fakes Be Written?* başlığı sahte uygulamanın bir bedeli olduğunu hatırlatır. Gerçeğe benzer davranması için emek ve alan bilgisi ister, gerçek uygulama değiştikçe de güncellenmesi gerekir. Yazarlar bu yüzden onu gerçek uygulamanın sahibi olan ekibe bırakır. Bakımı gereken sahte sayısını az tutmak için de onu genellikle, testte kullanılamayan kodun en altındaki API için yazmayı önerir. Örneğin veritabanı testte kullanılamıyorsa sahte, veritabanını çağıran her sınıf için değil, veritabanı API'sinin kendisi için yazılır.

API'nin sahibi sahte uygulama sunmuyorsa ne olacağını *What to Do If a Fake Is Not Available* başlığı anlatır. İlk adım, sahiplerinden bir sahte uygulama istemektir. Bu olmazsa yazarların gösterdiği yollardan biri, API'ye yapılan bütün çağrıları tek bir sınıfta toplayıp o sınıfın sahtesini yazmaktır. Kod çoğu zaman API'nin yalnızca bir kısmını kullandığı için bu, bütün API'yi taklit etmekten çok daha kolay olabilir.

Aşağıdaki örnekte yeniden adlandırma işi depodan yalnızca üç şey ister: okumak, yazmak ve silmek. Testteki sahte depo bu üç işi bellekte yapar:

```rust
/// What renaming needs from storage, and nothing more.
pub trait ModuleStore {
    fn read(&self, name: &str) -> Option<String>;
    fn write(&mut self, name: &str, text: String);
    fn remove(&mut self, name: &str);
}

pub fn rename_module(store: &mut impl ModuleStore, from: &str, to: &str) -> Result<(), String> {
    if store.read(to).is_some() {
        return Err(format!("{to} zaten var"));
    }
    let text = store.read(from).ok_or_else(|| format!("{from} bulunamadı"))?;
    store.write(to, text);
    store.remove(from);
    Ok(())
}

/// An in-memory store that behaves like the real one for these three calls.
#[derive(Default)]
struct MemoryStore(HashMap<String, String>);

impl ModuleStore for MemoryStore {
    fn read(&self, name: &str) -> Option<String> {
        self.0.get(name).cloned()
    }

    fn write(&mut self, name: &str, text: String) {
        self.0.insert(name.to_owned(), text);
    }

    fn remove(&mut self, name: &str) {
        self.0.remove(name);
    }
}

#[test]
fn renaming_onto_an_existing_module_is_refused() {
    let mut store = MemoryStore::default();
    store.write("dm-1.xml", "bir".into());
    store.write("dm-2.xml", "iki".into());

    let result = rename_module(&mut store, "dm-1.xml", "dm-2.xml");

    assert_eq!(result, Err("dm-2.xml zaten var".into()));
    assert_eq!(store.read("dm-2.xml").as_deref(), Some("iki"));
}
```

Test, deponun son durumuna bakar: hangi yöntemin kaç kez çağrıldığını değil, ortaya çıkan sonucu denetler. Trenk ve Bly'nin *Prefer State Testing Over Interaction Testing* başlığında önerdiği de budur.

## Generic mi, `dyn` mı?

Sahte uygulamayı koda vermenin iki yolu var. Yukarıdaki `impl ModuleStore` parametresi bir generic'tir. Rust Book'un *Generic Types, Traits, and Lifetimes* bölümündeki *Generic Data Types* başlığının *Performance of Code Using Generics* kısmına göre derleyici generic kodu kullanılan her somut tür için ayrı ayrı üretir. Kitap bu işleme monomorphization der. Çalışma anında bir bedeli yoktur.

`&dyn ModuleStore` gibi bir trait nesnesi ise hangi uygulamanın çağrılacağına çalışma anında karar verir. Rust Book'un *Using Trait Objects to Abstract over Shared Behavior* başlığındaki *Performing Dynamic Dispatch* kısmı bunun bedelini sayar: çalışma anında bir arama yapılır, derleyici çağrıyı satır içine alamaz ve trait'in `dyn` ile kullanılabilmesi için uyması gereken kurallar vardır. Bizce bunun karşılığında fonksiyonun her uygulama türü için ayrı bir kopyası üretilmez.

```rust
/// Dynamic dispatch: one copy, the store is chosen at run time.
pub fn module_count(store: &dyn ModuleStore, names: &[&str]) -> usize {
    names.iter().filter(|name| store.read(name).is_some()).count()
}
```

Bizce test için yalnızca bir dikiş (seam) gerekiyorsa generic yeterlidir. Uygulamanın çalışma anında seçilmesi gerekiyorsa ya da generic parametre kod boyunca yayılmaya başlıyorsa `dyn` düşünülür. Denemede aynı sahte depo, generic `rename_module`'e de `&dyn` alan `module_count`'a da sorunsuz verildi.

## Kayıt tutan mock nesne ve `RefCell`

Bazen sınanan kodun bir sonucu yoktur, yalnızca bir yan etkisi vardır: bir ileti gönderir, bir uyarı gösterir. Bunu sınamak için çağrıları kaydeden bir mock nesne kullanılır. Rust Book'un *Smart Pointers* bölümündeki *RefCell<T> and the Interior Mutability Pattern* başlığının *Testing with Mock Objects* kısmı, bunun Rust'ta bir engele takıldığını gösterir. Kitabın örneğinde ileti gönderen trait'in yöntemi `&self` alır, yani yöntem nesneyi değiştiremez. İletileri bir listeye yazmak isteyen mock nesne bu yüzden derlenmez. Kitap, trait'in tasarımını testin rahatı için bozmak istemez ve listeyi bir `RefCell` içine koyar: liste dışarıdan değişmez görünür, içeride değiştirilebilir. Aynı başlığın *Tracking Borrows at Runtime* kısmı bu kolaylığın bedelini yazar: ödünç alma kuralları artık derleme anında değil çalışma anında denetlenir, kural çiğnenirse program panik yapar. `RefCell` yalnızca tek iş parçacıklı kullanım içindir.

Denemede aynı durumu kurduk. `RefCell` olmadan yazılan kayıt tutucu derlenmedi, derleyici `&` başvurusunun arkasındaki alanın değiştirilemeyeceğini söyledi. `RefCell` ile şu hali çalıştı:

```rust
pub trait Notifier {
    fn notify(&self, message: &str);
}

pub fn warn_if_unsaved(changes: usize, notifier: &impl Notifier) {
    if changes > 0 {
        notifier.notify(&format!("{changes} kaydedilmemiş değişiklik"));
    }
}

/// Records messages; `notify` takes `&self`, so the list sits in a `RefCell`.
#[derive(Default)]
struct RecordingNotifier {
    messages: RefCell<Vec<String>>,
}

impl Notifier for RecordingNotifier {
    fn notify(&self, message: &str) {
        self.messages.borrow_mut().push(message.to_owned());
    }
}

#[test]
fn unsaved_changes_are_announced_once() {
    let notifier = RecordingNotifier::default();

    warn_if_unsaved(3, &notifier);

    assert_eq!(*notifier.messages.borrow(), ["3 kaydedilmemiş değişiklik"]);
}
```

Bu bir etkileşim testidir: sonucu değil, mock nesneye ne söylendiğini denetler. Trenk ve Bly, *When Is Interaction Testing Appropriate?* başlığında bunun iki durumda yerinde olduğunu söyler. Birincisi, gerçek uygulama ya da sahte uygulama kullanılamadığı için durum testi yapılamıyorsa. İkincisi, bir çağrının sayısı ya da sırası davranışı değiştiriyorsa. *Best Practices for Interaction Testing* başlığı bir ölçü daha verir: etkileşim testi yalnızca dış dünyayı değiştiren çağrılar için yapılmalıdır. Yazarların bu türden örnekleri arasında e-posta göndermek de vardır. Bizce kullanıcıya bir uyarı iletmek de bu türden bir çağrıdır.

## Mock çerçeveleri

Rust'ın standart kütüphanesinde bir mock aracı yoktur. Rust Book'un aynı kısmı da bunu söyler, ama bir yapının aynı işi görebileceğini ekler. Comprehensive Rust'ın *Mocking* sayfası Mockall'ı tanıtır: kodun önce trait'lerle yazılması gerekir, ardından Mockall o trait için bir mock üretir. Sayfaya göre çağrılar için argümana bağlı beklentiler kurulabilir, `.times(n)` ile de bir yöntemin kaç kez çağrılacağı sınırlanabilir.

Trenk ve Bly mock çerçevelerinin aşırı kullanımına karşı uyarır. *Case Study: @DoNotMock* başlığına göre Google'da bir API, kod tabanında binlerce kez mock'lanmış olabilir. Bu mock'ların API'nin sözleşmesini çiğneyen davranışlar göstermesi büyük olasılıktır, API'nin sahibi de uygulamasını değiştirmeden önce bu testleri düzeltmek zorunda kalır. Google bu yüzden Java'da, bir türün mock'lanmasını engelleyen bir ek açıklama (annotation) geliştirmiştir.

Mockall'ı bu not için denemedik. Projede kullanılmıyordu ve indirmek gerekecekti. Bu kısım yalnızca Comprehensive Rust'a dayanıyor.

## Bağımlılıkları giriş noktası kurar

Clean Code serisinin 07 notu, Wampler'ın *Sistemler* bölümündeki kuralı anlatmıştı: nesneleri kurmak ile kullanmak ayrı işlerdir, kurma işi programın girişine toplanır. Trenk ve Bly'nin *Seams* başlığı da aynı fikre test açısından bakar: bağımlılık enjeksiyonunda bir sınıf ihtiyaç duyduğu nesneleri kendisi kurmaz, dışarıdan alır, böylece testte başkası verilebilir.

Bizce Rust'ta bunun için bir çerçeveye gerek yok. Programın giriş noktası gerçek uygulamaları kurar ve fonksiyonlara ya da yapılara parametre olarak verir. Test aynı yere sahtesini verir. Rehberimizin sayfası da veritabanı havuzunu başlangıçta kurup uygulamaya vermeyi bu bağı ayıran karar olarak anlatmıştı.

## Bu projede yaşadığımız

- **Kendi trait'i ve kendi sahte nesnesi yoktu.** 7 Kasım 2026'nın sonunda proje kendi trait'ini tanımlamamıştı. Kodda `dyn` kullanılmıyordu, projenin kendi yazdığı bir sahte, stub ya da mock nesne de yoktu. Testler gerçek parçaları kullanıyordu: önizleme testleri Typst'i çok küçük bir sayfa şablonuyla çalıştırıyor, testler gerçek dosyalara geçici klasörlerde yazıyordu. ACP bağlantı denemesinin bazı testleri de gerçek alt süreçler başlatıyordu.
- **Değerler parametre olarak verildi.** 7 Kasım 2026'nın sonunda belge modeli bir dosya yolu değil, XML metni alıyordu. Bu yüzden testleri modeli testin içinde yazılmış küçük XML metinleriyle kurabiliyordu. ACP bağlantı denemesinin asıl fonksiyonu da komutu, çalışma klasörünü ve zaman aşımını parametre olarak alıyordu.
- **Bloklar `enum`'du.** 6 Kasım 2026'da belge modelinin blok türleri bir `enum` olarak tanımlandı. Küme, editörün gösterdiği içerikle sınırlıydı. Editörün henüz göstermediği tablo gibi adım içerikleri tek bir `Other` varyantında olduğu gibi tutuluyordu. Testler gerçek varyantlarla yazıldı.
- **Typst'in sınırı Typst'in trait'iydi.** 6 Kasım 2026'daki ilk sürümde proje, Typst'in dosyalara ve fontlara ulaşmak için istediği `World` trait'ini kendi yapısıyla uyguladı. Yani bu sınırı proje değil, kütüphane tanımlamıştı.
- **Fontlar dışarıdan verilmiyordu.** 7 Kasım 2026'nın sonunda aynı yapı fontları, sistem fontlarını bir kez tarayan global bir `static` değerden alıyordu. Bizce bu, testte fontları değiştirmeyi olanaksız kılıyordu. Font deposu yapıya dışarıdan verilseydi, testler yalnızca Typst'in gömülü fontlarıyla çalışabilirdi.
- **Arayüz kütüphanesinin sahtesi.** 7 Kasım 2026'nın sonunda GPUI testleri, GPUI'nin kendi ekibinin yazdığı test platformunu kullanıyordu. Serinin [GPUI testleri](04-gpui-testleri.html) notu bunu bir sahte uygulama olarak anlatıyor.

## Belgelerin söylemedikleri

- Rust Book'un tek test ikizi örneği bir mock nesnedir. Kitap, sahte uygulama ile mock arasında nasıl seçim yapılacağından söz etmez.
- Rust Book, trait ile `enum` arasındaki seçimi testlerle ilişkilendirmez.
