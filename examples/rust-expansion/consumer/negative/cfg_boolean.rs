fn main() {
    if cfg!(feature = "alternate") {
        missing_function();
    }
}
