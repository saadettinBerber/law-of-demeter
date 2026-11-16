use std::path::Path;

use law_of_demeter_core::application::{Finding, HybridFinding, ReadError, Report};
use law_of_demeter_core::domain::classification::{Classification, Types};
use law_of_demeter_core::domain::method::{Reason, Verdict};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

use super::render;

fn finding(verdict: Verdict) -> Finding {
    Finding::new(
        Path::new("src/OrderService.java"),
        "OrderService",
        "pay",
        verdict,
    )
}

fn report_of(findings: Vec<Finding>) -> Report {
    Report::new(1, 1, findings, Vec::new(), Vec::new(), Vec::new())
}

fn section<'a>(text: &'a str, title: &str) -> Vec<&'a str> {
    text.lines()
        .skip_while(|line| !line.starts_with(title))
        .skip(1)
        .take_while(|line| !line.is_empty())
        .collect()
}

#[test]
fn a_violation_is_listed_with_its_location_call_verdict_and_reason() {
    let report = report_of(vec![finding(Verdict::violation(
        "getWallet",
        11,
        Reason::return_of("getCustomer"),
    ))]);

    let text = render(&report, false);

    assert_eq!(
        section(&text, "İhlaller"),
        [
            "src/OrderService.java:11  OrderService.pay  getWallet()  İhlal: getCustomer() çağrısının döndürdüğü nesne f'nin arkadaşı değil"
        ]
    );
}

#[test]
fn a_violation_reached_through_a_variable_names_the_variable_and_the_origin() {
    let origin = || Reason::return_of("getLines");
    let report = report_of(vec![
        finding(Verdict::violation(
            "withdraw",
            4,
            Reason::local("customer", Reason::return_of("getCustomer")),
        )),
        finding(Verdict::violation(
            "applyDiscount",
            5,
            Reason::lambda_parameter("line", origin()),
        )),
        finding(Verdict::violation(
            "applyDiscount",
            6,
            Reason::loop_variable("line", origin()),
        )),
    ]);

    let text = render(&report, false);

    assert_eq!(
        section(&text, "İhlaller"),
        [
            "src/OrderService.java:4  OrderService.pay  withdraw()  İhlal: customer yerel değişkeni: getCustomer() çağrısının döndürdüğü nesne f'nin arkadaşı değil",
            "src/OrderService.java:5  OrderService.pay  applyDiscount()  İhlal: line lambda parametresi: getLines() çağrısının döndürdüğü nesne f'nin arkadaşı değil",
            "src/OrderService.java:6  OrderService.pay  applyDiscount()  İhlal: line döngü değişkeni: getLines() çağrısının döndürdüğü nesne f'nin arkadaşı değil",
        ]
    );
}

#[test]
fn a_violation_through_a_method_reference_names_its_type_method_and_origin() {
    let report = report_of(vec![finding(Verdict::violation(
        "getWallet",
        8,
        Reason::method_reference(
            TypeRef::class("shop.domain.Customer"),
            "getWallet",
            Reason::return_of("getCustomer"),
        ),
    ))]);

    let text = render(&report, false);

    assert_eq!(
        section(&text, "İhlaller"),
        [
            "src/OrderService.java:8  OrderService.pay  getWallet()  İhlal: Customer::getWallet metot referansı: getCustomer() çağrısının döndürdüğü nesne f'nin arkadaşı değil"
        ]
    );
}

#[test]
fn a_method_reference_is_counted_under_the_reason_that_feeds_it() {
    let reference = Reason::method_reference(
        TypeRef::class("shop.domain.Order"),
        "getCustomer",
        Reason::parameter("orders"),
    );

    let text = render(&report_of(vec![allowed(9, reference)]), true);

    assert_eq!(section(&text, "Uygun nedenleri"), ["  1  Argüman"]);
}

#[test]
fn an_object_a_function_receives_is_counted_under_its_own_reason() {
    let reference = Reason::method_reference(
        TypeRef::class("shop.domain.Order"),
        "getCustomer",
        Reason::FunctionArgument,
    );

    let text = render(&report_of(vec![allowed(9, reference)]), true);

    assert_eq!(
        section(&text, "Uygun nedenleri"),
        ["  1  Fonksiyonun argümanı"]
    );
}

#[test]
fn changing_a_returned_collection_names_the_collection_type() {
    let lines = TypeRef::generic("java.util.List", vec![TypeRef::class("OrderLine")]);
    let report = report_of(vec![finding(Verdict::violation(
        "add",
        7,
        Reason::container_mutation(lines, Reason::return_of("getLines")),
    ))]);

    let text = render(&report, false);

    assert_eq!(
        section(&text, "İhlaller"),
        [
            "src/OrderService.java:7  OrderService.pay  add()  İhlal: koleksiyon dışarıdan değiştiriliyor (java.util.List<OrderLine>): getLines() çağrısının döndürdüğü nesne f'nin arkadaşı değil"
        ]
    );
}

#[test]
fn unresolved_types_are_listed_apart_from_the_real_violations() {
    let report = report_of(vec![
        finding(Verdict::violation(
            "getWallet",
            3,
            Reason::return_of("getCustomer"),
        )),
        finding(Verdict::violation(
            "getCity",
            4,
            Reason::unresolved_type(
                Some(TypeRef::class("Address")),
                Reason::return_of("getAddress"),
            ),
        )),
        finding(Verdict::violation(
            "trim",
            5,
            Reason::unresolved_type(None, Reason::return_of("getName")),
        )),
    ]);

    let text = render(&report, false);

    assert_eq!(
        (
            section(&text, "İhlaller").len(),
            section(&text, "Çözülemeyen tipler")
        ),
        (
            1,
            vec![
                "src/OrderService.java:4  OrderService.pay  getCity()  Belirsiz: tip çözülemedi (Address): getAddress() çağrısının döndürdüğü nesne f'nin arkadaşı değil",
                "src/OrderService.java:5  OrderService.pay  trim()  Belirsiz: tip çözülemedi: getName() çağrısının döndürdüğü nesne f'nin arkadaşı değil",
            ]
        )
    );
}

fn allowed(line: usize, reason: Reason) -> Finding {
    finding(Verdict::allowed("call", line, reason))
}

fn allowed_lines(findings: Vec<Finding>) -> Vec<String> {
    let text = render(&report_of(findings), true);
    section(&text, "Uygun çağrılar")
        .into_iter()
        .map(|line| {
            line.split_once("  call()  Uygun: ")
                .map_or(line, |(_, reason)| reason)
                .to_owned()
        })
        .collect()
}

#[test]
fn allowed_calls_are_listed_with_their_reason_when_asked() {
    let report = report_of(vec![allowed(9, Reason::parameter("order"))]);

    let text = render(&report, true);

    assert_eq!(
        section(&text, "Uygun çağrılar"),
        [
            "src/OrderService.java:9  OrderService.pay  call()  Uygun: order f'ye argüman olarak verildi"
        ]
    );
}

#[test]
fn the_reasons_that_make_a_receiver_a_friend_read_in_turkish() {
    let lines = allowed_lines(vec![
        allowed(1, Reason::This),
        allowed(2, Reason::field("wallet")),
        allowed(
            3,
            Reason::created(TypeRef::class("java.lang.StringBuilder")),
        ),
        allowed(4, Reason::static_type(TypeRef::class("Order"))),
        allowed(5, Reason::Literal),
        allowed(6, Reason::local("current", Reason::parameter("order"))),
        allowed(7, Reason::lambda_parameter("o", Reason::field("orders"))),
        allowed(8, Reason::loop_variable("o", Reason::field("orders"))),
        allowed(9, Reason::static_result("builder")),
        allowed(10, Reason::factory_result("create")),
        allowed(11, Reason::query_result("findById")),
        allowed(
            12,
            Reason::method_reference(
                TypeRef::class("Order"),
                "getCustomer",
                Reason::FunctionArgument,
            ),
        ),
    ]);

    assert_eq!(
        lines,
        [
            "f'nin kendi nesnesi",
            "wallet sınıfın alanı",
            "f'nin içinde oluşturulan nesne (java.lang.StringBuilder)",
            "Order tipi üzerinde statik çağrı",
            "sabit değer",
            "current yerel değişkeni: order f'ye argüman olarak verildi",
            "o lambda parametresi: orders sınıfın alanı",
            "o döngü değişkeni: orders sınıfın alanı",
            "builder() statik çağrısının döndürdüğü nesne f'nin arkadaşı",
            "create() factory çağrısının döndürdüğü nesne f'nin arkadaşı",
            "findById() depo sorgusunun döndürdüğü nesne f'nin arkadaşı",
            "Order::getCustomer metot referansı: fonksiyona argüman olarak verilen nesne",
        ]
    );
}

#[test]
fn a_static_constant_reads_as_a_constant_not_a_call() {
    let lines = allowed_lines(vec![allowed(1, Reason::static_field("PAID"))]);

    assert_eq!(lines, ["PAID statik sabiti f'nin arkadaşı"]);
}

#[test]
fn the_reasons_that_make_a_stranger_allowed_read_in_turkish() {
    let server = TypeRef::class("Server");
    let lines = allowed_lines(vec![
        allowed(
            1,
            Reason::classified(Classification::DataStructure, server.clone()),
        ),
        allowed(2, Reason::classified(Classification::Value, server.clone())),
        allowed(
            3,
            Reason::classified(Classification::Container, server.clone()),
        ),
        allowed(4, Reason::fluent(server)),
    ]);

    assert_eq!(
        lines,
        [
            "Server bir veri yapısı",
            "Server bir değer",
            "Server bir konteyner",
            "Server üzerinde akıcı (fluent) çağrı",
        ]
    );
}

fn account_hybrid() -> HybridFinding {
    let account = TypeShape::new("Account", TypeKind::Class)
        .with_method(
            MethodShape::new("getBalance")
                .reading("balance")
                .returning(TypeRef::primitive("int")),
        )
        .with_method(
            MethodShape::new("setBalance")
                .writing("balance")
                .with_parameters(vec![TypeRef::primitive("int")]),
        )
        .with_method(MethodShape::new("withdraw"))
        .with_method(MethodShape::new("deposit"));
    let hybrid = Types::new()
        .with_shape(account)
        .hybrid_class("Account")
        .expect("Account is a hybrid");
    HybridFinding::new(Path::new("src/Account.java"), hybrid)
}

#[test]
fn a_hybrid_class_is_listed_with_its_behavior_and_its_setters() {
    let report = Report::new(
        1,
        0,
        Vec::new(),
        vec![account_hybrid()],
        Vec::new(),
        Vec::new(),
    );

    let text = render(&report, false);

    assert_eq!(
        section(&text, "Hibrit sınıflar"),
        [
            "src/Account.java  Account  Hibrit sınıf: davranış: withdraw, deposit; setter: setBalance"
        ]
    );
}

#[test]
fn read_errors_are_listed_with_their_line_when_they_have_one() {
    let report = Report::new(
        1,
        0,
        Vec::new(),
        Vec::new(),
        vec![
            ReadError::at_line(Path::new("src/Broken.java"), 3, "beklenmeyen simge"),
            ReadError::new(Path::new("src/Locked.java"), "izin reddedildi"),
        ],
        vec![ReadError::new(Path::new("lib/broken.jar"), "bozuk arşiv")],
    );

    let text = render(&report, false);

    assert_eq!(
        (
            section(&text, "Kaynak hataları"),
            section(&text, "Tip okuma hataları")
        ),
        (
            vec![
                "src/Broken.java:3  Hata: beklenmeyen simge",
                "src/Locked.java  Hata: izin reddedildi",
            ],
            vec!["lib/broken.jar  Hata: bozuk arşiv"]
        )
    );
}

fn mixed_report() -> Report {
    Report::new(
        2,
        3,
        vec![
            allowed(1, Reason::parameter("order")),
            allowed(2, Reason::parameter("order")),
            allowed(
                3,
                Reason::classified(Classification::DataStructure, TypeRef::class("Server")),
            ),
            finding(Verdict::violation("a", 4, Reason::return_of("getCustomer"))),
            finding(Verdict::violation(
                "b",
                5,
                Reason::local("customer", Reason::return_of("getCustomer")),
            )),
            finding(Verdict::violation(
                "add",
                6,
                Reason::container_mutation(
                    TypeRef::class("java.util.List"),
                    Reason::return_of("getLines"),
                ),
            )),
            finding(Verdict::violation(
                "c",
                7,
                Reason::unresolved_type(None, Reason::return_of("getName")),
            )),
        ],
        vec![account_hybrid()],
        vec![ReadError::new(Path::new("src/Broken.java"), "hata")],
        vec![ReadError::new(Path::new("lib/broken.jar"), "bozuk arşiv")],
    )
}

#[test]
fn the_summary_counts_files_methods_calls_and_findings() {
    let text = render(&mixed_report(), false);

    assert_eq!(
        section(&text, "Özet"),
        [
            "  Dosya: 2",
            "  Metot: 3",
            "  Çağrı: 7 (uygun 3, ihlal 3, belirsiz 1)",
            "  Hibrit sınıf: 1",
            "  Kaynak hatası: 1",
            "  Tip okuma hatası: 1",
        ]
    );
}

#[test]
fn violations_are_counted_per_reason_and_unresolved_types_are_left_out() {
    let text = render(&mixed_report(), false);

    assert_eq!(
        section(&text, "İhlal nedenleri"),
        [
            "  2  Döndürülen nesne f'nin arkadaşı değil",
            "  1  Döndürülen koleksiyon değiştiriliyor",
        ]
    );
}

#[test]
fn allowed_calls_are_counted_per_reason_when_they_are_shown() {
    let text = render(&mixed_report(), true);

    assert_eq!(
        section(&text, "Uygun nedenleri"),
        ["  2  Argüman", "  1  Veri yapısı"]
    );
}

#[test]
fn a_report_without_violations_says_so() {
    let report = report_of(vec![allowed(1, Reason::parameter("order"))]);

    let text = render(&report, false);

    assert_eq!(text.lines().next(), Some("İhlal bulunamadı."));
}

#[test]
fn allowed_calls_stay_out_of_the_default_report() {
    let text = render(&mixed_report(), false);

    assert_eq!(
        (
            text.contains("Uygun çağrılar"),
            text.contains("Uygun nedenleri"),
            text.contains("  Uygun: ")
        ),
        (false, false, false)
    );
}

#[test]
fn the_whole_default_report_reads_top_to_bottom() {
    let text = render(&mixed_report(), false);

    assert_eq!(
        text,
        "İhlaller (3):
src/OrderService.java:4  OrderService.pay  a()  İhlal: getCustomer() çağrısının döndürdüğü nesne f'nin arkadaşı değil
src/OrderService.java:5  OrderService.pay  b()  İhlal: customer yerel değişkeni: getCustomer() çağrısının döndürdüğü nesne f'nin arkadaşı değil
src/OrderService.java:6  OrderService.pay  add()  İhlal: koleksiyon dışarıdan değiştiriliyor (java.util.List): getLines() çağrısının döndürdüğü nesne f'nin arkadaşı değil

Çözülemeyen tipler (1):
src/OrderService.java:7  OrderService.pay  c()  Belirsiz: tip çözülemedi: getName() çağrısının döndürdüğü nesne f'nin arkadaşı değil

Hibrit sınıflar (1):
src/Account.java  Account  Hibrit sınıf: davranış: withdraw, deposit; setter: setBalance

Kaynak hataları (1):
src/Broken.java  Hata: hata

Tip okuma hataları (1):
lib/broken.jar  Hata: bozuk arşiv

Özet:
  Dosya: 2
  Metot: 3
  Çağrı: 7 (uygun 3, ihlal 3, belirsiz 1)
  Hibrit sınıf: 1
  Kaynak hatası: 1
  Tip okuma hatası: 1

İhlal nedenleri:
  2  Döndürülen nesne f'nin arkadaşı değil
  1  Döndürülen koleksiyon değiştiriliyor
"
    );
}

#[test]
fn every_kind_of_allowed_call_has_a_reason_group_in_the_summary() {
    let server = TypeRef::class("Server");
    let report = report_of(vec![
        allowed(1, Reason::This),
        allowed(2, Reason::field("wallet")),
        allowed(3, Reason::parameter("order")),
        allowed(4, Reason::created(server.clone())),
        allowed(5, Reason::static_type(server.clone())),
        allowed(6, Reason::Literal),
        allowed(7, Reason::local("c", Reason::field("wallet"))),
        allowed(
            8,
            Reason::classified(Classification::DataStructure, server.clone()),
        ),
        allowed(9, Reason::classified(Classification::Value, server.clone())),
        allowed(
            10,
            Reason::classified(Classification::Container, server.clone()),
        ),
        allowed(11, Reason::fluent(server)),
        allowed(12, Reason::static_result("builder")),
        allowed(13, Reason::factory_result("create")),
        allowed(14, Reason::query_result("findById")),
        allowed(15, Reason::static_field("PAID")),
    ]);

    let text = render(&report, true);

    assert_eq!(
        section(&text, "Uygun nedenleri"),
        [
            "  2  Sınıfın alanı",
            "  1  Akıcı çağrı",
            "  1  Argüman",
            "  1  Depo sorgusunun sonucu",
            "  1  Değer",
            "  1  Factory'nin sonucu",
            "  1  Kendi nesnesi",
            "  1  Konteyner",
            "  1  Metotta oluşturulan nesne",
            "  1  Sabit değer",
            "  1  Statik sabit",
            "  1  Statik çağrı",
            "  1  Statik çağrının sonucu",
            "  1  Veri yapısı",
        ]
    );
}

#[test]
fn a_library_type_reads_as_allowed_and_forms_its_own_reason_group() {
    let lock = TypeRef::class("java.util.concurrent.locks.ReentrantReadWriteLock$WriteLock");
    let report = report_of(vec![allowed(3, Reason::library(lock))]);

    let text = render(&report, true);

    assert_eq!(
        (
            section(&text, "Uygun çağrılar"),
            section(&text, "Uygun nedenleri")
        ),
        (
            vec![
                "src/OrderService.java:3  OrderService.pay  call()  Uygun: kütüphane tipi (java.util.concurrent.locks.ReentrantReadWriteLock$WriteLock)"
            ],
            vec!["  1  Kütüphane tipi"]
        )
    );
}
