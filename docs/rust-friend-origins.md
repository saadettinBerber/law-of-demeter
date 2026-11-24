# Temel Rust arkadaş kökleri

Bu dilim `01e238b` tabanındaki onaylı kaynak adaptörünü genişletir. Üretim değişiklikleri yalnız `crates/rust-source` içindedir; domain, Java, CLI, vendor/probe ve kök Cargo dosyaları korunur. Geniş karar satırları tamamlandı sayılmaz.

İlk kaynak sözleşmesi gerçek Cargo projesini `RustProject` → `CheckProject` üzerinden okur: `order.charge(amount)` izinli; `order.customer().wallet().withdraw(amount)` üzerinde `customer` izinli, `wallet` için `return_of("customer")`, `withdraw` için `return_of("wallet")` ihlali vardır. Dosya/satır ve tam verdict bağımsız beklentiyle sınanır. Customer ve Wallet açık davranış metotları taşır; sınıflandırma etiketi enjekte edilmez.

Tam zincir ilk çalıştırmada yeşildi. Dönüş tipi köprüsünü geçici unit tipe indirmek iki ihlali yanlış izinli yaptı; test gerçek assertion'da kırıldı ve kaynak byte eş geri alındı. `flow-first.log`, `flow-fault.log`, `flow-fault.json` ve `flow-verification.json`, worktree'nin `target/rust-friends-journey` dizinindedir. Başlangıçtaki 17 test, tam zincir ve typed RA alan kimliği testiyle 19 test geçti; fmt ve workspace Clippy temizdi.

Typed alan öğrenme testi, `self.wallet` ve başka bir PaymentService örneğinin aynı özel `wallet` alanına çözülmesini doğrular. Aynı bildirim/erişim yetkisi, aynı arkadaş kökü demek değildir; iş kararı mevcut domain alan köprüleriyle aktarılacaktır.
