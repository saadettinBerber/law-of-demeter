use std::num::NonZero;
use std::panic::resume_unwind;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

pub(crate) fn map<T: Sync, R: Send>(items: &[T], work: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let available = thread::available_parallelism().map_or(1, NonZero::get);
    let workers = available.min(items.len());
    let next = AtomicUsize::new(0);
    let mut slots: Vec<Option<R>> = items.iter().map(|_| None).collect();
    thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                scope.spawn(|| {
                    let mut finished = Vec::new();
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(item) = items.get(index) else {
                            return finished;
                        };
                        finished.push((index, work(item)));
                    }
                })
            })
            .collect();
        for handle in handles {
            match handle.join() {
                Ok(finished) => {
                    for (index, result) in finished {
                        slots[index] = Some(result);
                    }
                }
                Err(panic) => resume_unwind(panic),
            }
        }
    });
    slots.into_iter().flatten().collect()
}
