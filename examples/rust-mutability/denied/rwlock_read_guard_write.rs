use std::sync::RwLock;

fn main() {
    let balance = RwLock::new(50);
    let Ok(guard) = balance.read() else {
        return;
    };

    *guard = 30;
}
