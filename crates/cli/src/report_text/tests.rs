use std::path::Path;

use law_of_demeter_core::application::{Finding, Report};
use law_of_demeter_core::domain::method::{Reason, Verdict};
use law_of_demeter_core::domain::type_shape::TypeRef;

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
