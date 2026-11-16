mod judging;
mod nesting;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use judging::verdicts_in_c;
use nesting::{
    IMPORTS, collected_chain, nested_collectors, nested_lambda_collectors, snippet, world,
};

const BUDGET: Duration = Duration::from_secs(3);

fn judges_within_budget(statement: String) -> bool {
    let (done, finished) = mpsc::channel();
    thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || {
            verdicts_in_c(IMPORTS, "", &snippet(&statement), "group", &world());
            let _ = done.send(());
        })
        .unwrap();
    finished.recv_timeout(BUDGET).is_ok()
}

#[test]
fn collectors_nested_twelve_levels_deep_with_references_are_judged_within_budget() {
    assert!(judges_within_budget(nested_collectors(12)));
}

#[test]
fn collectors_nested_twelve_levels_deep_with_lambdas_are_judged_within_budget() {
    assert!(judges_within_budget(nested_lambda_collectors(12)));
}

#[test]
fn a_long_chain_of_collected_streams_is_judged_in_linear_time() {
    assert!(judges_within_budget(collected_chain(300)));
}
