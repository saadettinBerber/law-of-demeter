use std::rc::Rc;

fn main() {
    let balance = Rc::new(50);

    *balance = 30;
}
