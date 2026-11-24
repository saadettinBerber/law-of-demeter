#[test]
#[allow(array_into_iter)]
fn a_legacy_token_does_not_change_the_following_modern_token() {
    let original = [String::from("legacy-first"), String::from("legacy-second")];
    let consumed = [String::from("modern-first"), String::from("modern-second")];
    let legacy = edition_provider::definition_items!(original);
    let modern = edition_provider::callsite_items!(consumed, into_iter);

    let first_reference = legacy.into_iter().next().expect("legacy item is missing");
    let bytes = modern
        .into_iter()
        .next()
        .expect("modern item is missing")
        .into_bytes();

    assert!(std::ptr::eq(first_reference, &original[0]));
    assert_eq!(bytes, b"modern-first");
}

#[test]
#[allow(array_into_iter)]
fn a_modern_token_does_not_change_the_following_legacy_token() {
    let original = [String::from("legacy-first"), String::from("legacy-second")];
    let consumed = [String::from("modern-first"), String::from("modern-second")];
    let modern = edition_provider::callsite_items!(consumed, into_iter);
    let legacy = edition_provider::definition_items!(original);

    let bytes = modern
        .into_iter()
        .next()
        .expect("modern item is missing")
        .into_bytes();
    let first_reference = legacy.into_iter().next().expect("legacy item is missing");

    assert_eq!(bytes, b"modern-first");
    assert!(std::ptr::eq(first_reference, &original[0]));
}
