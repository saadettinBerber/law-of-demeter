#[allow(array_into_iter, clippy::into_iter_on_ref)]
pub fn definition_count() -> usize {
    let values = [String::from("first"), String::from("second")];
    let reference = (&values).into_iter();
    let iterator = edition_provider::definition_items!(values);

    assert_eq!(reference.count(), 2);
    iterator.count()
}

pub fn callsite_count() -> usize {
    let reference_values = [String::from("first"), String::from("second")];
    let reference = reference_values.into_iter();
    let values = [String::from("first"), String::from("second")];
    let iterator = edition_provider::callsite_items!(values, into_iter);

    assert_eq!(reference.count(), 2);
    iterator.count()
}

#[cfg(test)]
mod tests {
    #[test]
    #[allow(array_into_iter)]
    fn a_definition_token_keeps_reference_identity() {
        let values = [String::from("first"), String::from("second")];
        let mut iterator = edition_provider::definition_items!(values);

        let first = iterator.next().expect("first item is missing");

        assert!(std::ptr::eq(first, &values[0]));
    }

    #[test]
    fn a_consumer_token_yields_an_owned_string() {
        let values = [String::from("first"), String::from("second")];
        let mut iterator = edition_provider::callsite_items!(values, into_iter);

        let first = iterator.next().expect("first item is missing");
        let bytes = first.into_bytes();

        assert_eq!(bytes, b"first");
    }
}
