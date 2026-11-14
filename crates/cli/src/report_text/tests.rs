use std::path::Path;

use law_of_demeter_core::application::{Finding, Report};
use law_of_demeter_core::domain::method::{Reason, Verdict};

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
