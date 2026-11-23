fn main() {
    let result: Result<u32, String> = Ok(50);

    let item: Option<&String> = result.iter().next();

    assert!(item.is_some());
}
