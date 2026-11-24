#[macro_export]
macro_rules! definition_items {
    ($array:expr) => {
        $array.into_iter()
    };
}

#[macro_export]
macro_rules! callsite_items {
    ($array:expr, $method:ident) => {
        $array.$method()
    };
}

#[allow(array_into_iter, clippy::into_iter_on_ref)]
pub fn direct_count() -> usize {
    let values = [String::from("first"), String::from("second")];
    let reference = (&values).into_iter();
    let iterator = values.into_iter();

    assert_eq!(reference.count(), 2);
    iterator.count()
}
