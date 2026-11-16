mod nesting;

use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use law_of_demeter_java_source::map_source;
use nesting::{
    IMPORTS, collected_chain, nested_collectors, nested_lambda_collectors, snippet, world,
};

const BUDGET: Duration = Duration::from_secs(3);

fn maps_within_budget(statement: String) -> bool {
    let (done, finished) = mpsc::channel();
    thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || {
            let source = format!("{IMPORTS} class C {{\n{}\n}}", snippet(&statement));
            map_source(Path::new("C.java"), &source, &world()).unwrap();
            let _ = done.send(());
        })
        .unwrap();
    finished.recv_timeout(BUDGET).is_ok()
}

#[test]
fn collectors_nested_with_references_map_in_linear_time() {
    assert!(maps_within_budget(nested_collectors(300)));
}

#[test]
fn collectors_nested_with_lambdas_map_in_linear_time() {
    assert!(maps_within_budget(nested_lambda_collectors(300)));
}

#[test]
fn a_long_chain_of_collected_streams_maps_in_linear_time() {
    assert!(maps_within_budget(collected_chain(300)));
}
