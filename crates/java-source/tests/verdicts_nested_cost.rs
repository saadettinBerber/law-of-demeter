mod judging;
mod nesting;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use judging::verdicts_in_c;
use nesting::{
    IMPORTS, collected_chain, nested_collectors, nested_lambda_collectors, snippet, world,
};

const THREE_SECONDS: Duration = Duration::from_secs(3);

fn judges_within(limit: Duration, statement: String) -> bool {
    let (done, finished) = mpsc::channel();
    thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || {
            verdicts_in_c(IMPORTS, "", &snippet(&statement), "group", &world());
            let _ = done.send(());
        })
        .unwrap();
    finished.recv_timeout(limit).is_ok()
}

#[test]
fn collectors_nested_twenty_four_levels_deep_with_references_are_judged_within_three_seconds() {
    assert!(judges_within(THREE_SECONDS, nested_collectors(24)));
}

#[test]
fn collectors_nested_twenty_four_levels_deep_with_lambdas_are_judged_within_three_seconds() {
    assert!(judges_within(THREE_SECONDS, nested_lambda_collectors(24)));
}

#[test]
fn a_long_chain_of_collected_streams_is_judged_in_linear_time() {
    assert!(judges_within(THREE_SECONDS, collected_chain(300)));
}
