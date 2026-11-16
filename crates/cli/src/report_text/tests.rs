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
        section(&text, "Violations"),
        [
            "src/OrderService.java:11  OrderService.pay  getWallet()  Violation: the object returned by getCustomer() is not a friend of f"
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
        section(&text, "Violations"),
        [
            "src/OrderService.java:4  OrderService.pay  withdraw()  Violation: local variable customer: the object returned by getCustomer() is not a friend of f",
            "src/OrderService.java:5  OrderService.pay  applyDiscount()  Violation: lambda parameter line: the object returned by getLines() is not a friend of f",
            "src/OrderService.java:6  OrderService.pay  applyDiscount()  Violation: loop variable line: the object returned by getLines() is not a friend of f",
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
        section(&text, "Violations"),
        [
            "src/OrderService.java:8  OrderService.pay  getWallet()  Violation: method reference Customer::getWallet: the object returned by getCustomer() is not a friend of f"
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

    assert_eq!(
        section(&text, "Reasons for allowed calls"),
        ["  1  Argument"]
    );
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
        section(&text, "Reasons for allowed calls"),
        ["  1  Function argument"]
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
        section(&text, "Violations"),
        [
            "src/OrderService.java:7  OrderService.pay  add()  Violation: collection modified from outside (java.util.List<OrderLine>): the object returned by getLines() is not a friend of f"
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
            section(&text, "Violations").len(),
            section(&text, "Unresolved types")
        ),
        (
            1,
            vec![
                "src/OrderService.java:4  OrderService.pay  getCity()  Unresolved: type not resolved (Address): the object returned by getAddress() is not a friend of f",
                "src/OrderService.java:5  OrderService.pay  trim()  Unresolved: type not resolved: the object returned by getName() is not a friend of f",
            ]
        )
    );
}

fn allowed(line: usize, reason: Reason) -> Finding {
    finding(Verdict::allowed("call", line, reason))
}

fn allowed_lines(findings: Vec<Finding>) -> Vec<String> {
    let text = render(&report_of(findings), true);
    section(&text, "Allowed calls")
        .into_iter()
        .map(|line| {
            line.split_once("  call()  Allowed: ")
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
        section(&text, "Allowed calls"),
        ["src/OrderService.java:9  OrderService.pay  call()  Allowed: order is an argument of f"]
    );
}

#[test]
fn the_reasons_that_make_a_receiver_a_friend_read_in_english() {
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
            "f's own object",
            "wallet is a field of the class",
            "object created in f (java.lang.StringBuilder)",
            "static call on type Order",
            "literal value",
            "local variable current: order is an argument of f",
            "lambda parameter o: orders is a field of the class",
            "loop variable o: orders is a field of the class",
            "the object returned by static call builder() is a friend of f",
            "the object returned by factory call create() is a friend of f",
            "the object returned by repository query findById() is a friend of f",
            "method reference Order::getCustomer: object passed to a function as an argument",
        ]
    );
}

#[test]
fn a_static_constant_reads_as_a_constant_not_a_call() {
    let lines = allowed_lines(vec![allowed(1, Reason::static_field("PAID"))]);

    assert_eq!(lines, ["static constant PAID is a friend of f"]);
}

#[test]
fn the_reasons_that_make_a_stranger_allowed_read_in_english() {
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
            "Server is a data structure",
            "Server is a value",
            "Server is a container",
            "fluent call on Server",
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
        section(&text, "Hybrid classes"),
        [
            "src/Account.java  Account  Hybrid class: behavior: withdraw, deposit; setters: setBalance"
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
            ReadError::at_line(Path::new("src/Broken.java"), 3, "unexpected token"),
            ReadError::new(Path::new("src/Locked.java"), "permission denied"),
        ],
        vec![ReadError::new(
            Path::new("lib/broken.jar"),
            "broken archive",
        )],
    );

    let text = render(&report, false);

    assert_eq!(
        (
            section(&text, "Source errors"),
            section(&text, "Type reading errors")
        ),
        (
            vec![
                "src/Broken.java:3  Error: unexpected token",
                "src/Locked.java  Error: permission denied",
            ],
            vec!["lib/broken.jar  Error: broken archive"]
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
        vec![ReadError::new(
            Path::new("lib/broken.jar"),
            "broken archive",
        )],
    )
}

#[test]
fn the_summary_counts_files_methods_calls_and_findings() {
    let text = render(&mixed_report(), false);

    assert_eq!(
        section(&text, "Summary"),
        [
            "  Files: 2",
            "  Methods: 3",
            "  Calls: 7 (allowed 3, violations 3, unresolved 1)",
            "  Hybrid classes: 1",
            "  Source errors: 1",
            "  Type reading errors: 1",
        ]
    );
}

#[test]
fn violations_are_counted_per_reason_and_unresolved_types_are_left_out() {
    let text = render(&mixed_report(), false);

    assert_eq!(
        section(&text, "Reasons for violations"),
        [
            "  2  Returned object is not a friend of f",
            "  1  Returned collection is modified",
        ]
    );
}

#[test]
fn allowed_calls_are_counted_per_reason_when_they_are_shown() {
    let text = render(&mixed_report(), true);

    assert_eq!(
        section(&text, "Reasons for allowed calls"),
        ["  2  Argument", "  1  Data structure"]
    );
}

#[test]
fn a_report_without_violations_says_so() {
    let report = report_of(vec![allowed(1, Reason::parameter("order"))]);

    let text = render(&report, false);

    assert_eq!(text.lines().next(), Some("No violations found."));
}

#[test]
fn allowed_calls_stay_out_of_the_default_report() {
    let text = render(&mixed_report(), false);

    assert_eq!(
        (
            text.contains("Allowed calls"),
            text.contains("Reasons for allowed calls"),
            text.contains("  Allowed: ")
        ),
        (false, false, false)
    );
}

#[test]
fn the_whole_default_report_reads_top_to_bottom() {
    let text = render(&mixed_report(), false);

    assert_eq!(
        text,
        "Violations (3):
src/OrderService.java:4  OrderService.pay  a()  Violation: the object returned by getCustomer() is not a friend of f
src/OrderService.java:5  OrderService.pay  b()  Violation: local variable customer: the object returned by getCustomer() is not a friend of f
src/OrderService.java:6  OrderService.pay  add()  Violation: collection modified from outside (java.util.List): the object returned by getLines() is not a friend of f

Unresolved types (1):
src/OrderService.java:7  OrderService.pay  c()  Unresolved: type not resolved: the object returned by getName() is not a friend of f

Hybrid classes (1):
src/Account.java  Account  Hybrid class: behavior: withdraw, deposit; setters: setBalance

Source errors (1):
src/Broken.java  Error: hata

Type reading errors (1):
lib/broken.jar  Error: broken archive

Summary:
  Files: 2
  Methods: 3
  Calls: 7 (allowed 3, violations 3, unresolved 1)
  Hybrid classes: 1
  Source errors: 1
  Type reading errors: 1

Reasons for violations:
  2  Returned object is not a friend of f
  1  Returned collection is modified
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
        section(&text, "Reasons for allowed calls"),
        [
            "  2  Field of the class",
            "  1  Argument",
            "  1  Container",
            "  1  Data structure",
            "  1  Fluent call",
            "  1  Literal",
            "  1  Object created in the method",
            "  1  Own object",
            "  1  Result of a factory",
            "  1  Result of a repository query",
            "  1  Result of a static call",
            "  1  Static call",
            "  1  Static constant",
            "  1  Value",
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
            section(&text, "Allowed calls"),
            section(&text, "Reasons for allowed calls")
        ),
        (
            vec![
                "src/OrderService.java:3  OrderService.pay  call()  Allowed: library type (java.util.concurrent.locks.ReentrantReadWriteLock$WriteLock)"
            ],
            vec!["  1  Library type"]
        )
    );
}
