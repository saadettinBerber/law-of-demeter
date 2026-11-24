#[path = "../support/macro_edition.rs"]
mod macro_edition;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    macro_edition::the_model_preserves_editions_and_direct_or_forwarded_token_dispatch()?;
    macro_edition::a_definition_token_uses_its_legacy_edition_for_array_method_lookup()?;
    macro_edition::check_call_orders()
}
