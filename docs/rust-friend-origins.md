# Temel Rust arkadaş kökleri

Bu dilim `01e238b` tabanındaki onaylı kaynak adaptörünü genişletir. Üretim değişiklikleri yalnız `crates/rust-source` içindedir; domain, Java, CLI, vendor/probe ve kök Cargo dosyaları korunur. Geniş karar satırları tamamlandı sayılmaz.

İlk kaynak sözleşmesi gerçek Cargo projesini `RustProject` → `CheckProject` üzerinden okur: `order.charge(amount)` izinli; `order.customer().wallet().withdraw(amount)` üzerinde `customer` izinli, `wallet` için `return_of("customer")`, `withdraw` için `return_of("wallet")` ihlali vardır. Dosya/satır ve tam verdict bağımsız beklentiyle sınanır. Customer ve Wallet açık davranış metotları taşır; sınıflandırma etiketi enjekte edilmez.

Tam zincir ilk çalıştırmada yeşildi. Dönüş tipi köprüsünü geçici unit tipe indirmek iki ihlali yanlış izinli yaptı; test gerçek assertion'da kırıldı ve kaynak byte eş geri alındı. `flow-first.log`, `flow-fault.log`, `flow-fault.json` ve `flow-verification.json`, worktree'nin `target/rust-friends-journey` dizinindedir. Başlangıçtaki 17 test, tam zincir ve typed RA alan kimliği testiyle 19 test geçti; fmt ve workspace Clippy temizdi.

Typed alan öğrenme testi, `self.wallet` ve başka bir PaymentService örneğinin aynı özel `wallet` alanına çözülmesini doğrular. Aynı bildirim/erişim yetkisi, aynı arkadaş kökü demek değildir; köken mevcut domain alan köprüleriyle aktarılır.

Doğrudan alan köprüsü eklendi. Çözülmüş alanın gerçek sahibi, alıcının yalnız referansları çıkarılmış tipiyle eşleşmelidir; başka nesneye custom projection açık `Unsupported` kalır. `self` ve kendi doğrudan alanı arkadaş, aynı modülde aynı özel bildirime ulaşan başka örneğin alanı yabancıdır. Alan karşılaştırması önce eksik iki verdict nedeniyle kırmızı, sonra tam beklenen dört verdict ile yeşil oldu. `fields-red.log` / `fields-green.log` davranış kanıtıdır; `fields-verification.json` ile 20 adaptör testi, fmt ve workspace Clippy geçti.

Gerçek named struct ve tuple struct oluşturma, initializer ifadeleriyle birlikte `Creation` köprüsüne aktarılır. Tuple çağrısı gerçek `CallableKind::TupleStruct` hedefiyle tanınır; `BalancePocket` takma adı da aynı bildirime çözülür. `Pocket::new` ise mevcut değeri döndüren sıradan bir fonksiyondur ve açık `Unsupported` kalır. Yeni Holder içindeki mevcut Wallet, alan politikası gereği oluşturulmuş arkadaş olmaz.

Basit `let` bağları gerçek HIR `Local` kimliğiyle tutulur. Domain'e okunabilir tekil bağ adı ile gerçek bildirilen tip verilir. Parametreyi veya önceki bağı gölgelemek, desenli bağlar ve `let else` henüz desteklenmez. Değerin orijinal ve ayarlanmış tipi yalnız normal referanslar çıkarılınca aynı olmalıdır; başka nesneye coercion sessiz köken değişikliği yapamaz. HIR sorguları aynı snapshot'ın bağlı veritabanı kapsamındadır.

Oluşturma testleri önce eksik verdict nedeniyle kırıldı, sonra initializer çağrıları ve oluşturulmuş local gerekçeleriyle geçti. Sıradan `new` fonksiyonunu constructor sayan geçici hata açık issue yerine yanlış izinli verdict üretti; test kırıldı ve kaynak byte eş geri alındı. `creation-red.log`, `constructor-fault.log` / `constructor-fault.json` davranış kanıtıdır. Eski desteklenmeyen gövde testi, artık tanınan struct/let satırı yerine sonraki desteklenmeyen associated function çağrısının gerçek 22. satırını bekler.

`creation-verification.json` ile 29 adaptör testi, workspace fmt ve Clippy geçti. Derleme/kurulum ve bağlı RA sorgu bağlamı hataları davranış kırmızısı olarak sayılmadı.
