#[path = "../support/macro_edition.rs"]
mod macro_edition;

#[test]
fn direct_and_forwarded_method_tokens_keep_their_editions() -> Result<(), Box<dyn std::error::Error>>
{
    macro_edition::the_model_preserves_editions_and_direct_or_forwarded_token_dispatch()
}

#[test]
fn a_legacy_method_token_selects_the_borrowed_array_iterator()
-> Result<(), Box<dyn std::error::Error>> {
    macro_edition::a_definition_token_uses_its_legacy_edition_for_array_method_lookup()
}

#[test]
fn successive_legacy_and_modern_tokens_keep_both_call_orders()
-> Result<(), Box<dyn std::error::Error>> {
    macro_edition::check_call_orders()
}
