use std::cell::Cell;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

pub fn generated_balance() -> i32 {
    let wallet = Wallet::new(50);
    generated_charge(&wallet);
    wallet.balance()
}

#[cfg(not(feature = "alternate"))]
pub fn selected_value() -> i32 {
    17
}

#[cfg(feature = "alternate")]
pub fn selected_value() -> i32 {
    23
}

struct Wallet {
    balance: Cell<i32>,
    charges: Cell<usize>,
}

impl Wallet {
    fn new(balance: i32) -> Self {
        Self {
            balance: Cell::new(balance),
            charges: Cell::new(0),
        }
    }

    fn charge(&self, amount: i32) {
        self.balance.set(self.balance.get() - amount);
        self.charges.set(self.charges.get() + 1);
    }

    fn balance(&self) -> i32 {
        self.balance.get()
    }
}

#[cfg(test)]
mod tests {
    use super::{Wallet, generated_balance, selected_value};

    #[test]
    #[cfg(not(build_bonus))]
    fn generated_base_code_uses_the_declared_input() {
        let requested = env!("LOD_FIXTURE_MODE");

        let balance = generated_balance();

        assert_eq!(requested, "base");
        assert_eq!(balance, 43);
    }

    #[test]
    #[cfg(build_bonus)]
    fn generated_bonus_code_uses_the_build_cfg() {
        let requested = env!("LOD_FIXTURE_MODE");

        let balance = generated_balance();

        assert_eq!(requested, "bonus");
        assert_eq!(balance, 39);
    }

    #[test]
    #[cfg(not(feature = "alternate"))]
    fn the_default_feature_body_is_selected() {
        let expected = 17;

        let selected = selected_value();

        assert_eq!(selected, expected);
    }

    #[test]
    #[cfg(feature = "alternate")]
    fn the_alternate_feature_body_is_selected() {
        let expected = 23;

        let selected = selected_value();

        assert_eq!(selected, expected);
    }

    #[test]
    fn target_cfg_selects_the_declared_host_body() {
        let expected = "macos-arm64";

        let selected = selected_target();

        assert_eq!(selected, expected);
        assert_eq!(usize::BITS, 64);
    }

    #[test]
    fn one_declarative_invocation_generates_two_distinct_charges() {
        let order = Order {
            wallet: Wallet::new(50),
            reads: Cell::new(0),
        };

        let balances = expansion_legacy::charge_twice!(&order);

        assert_eq!(balances, (43, 32));
        assert_eq!(order.reads.get(), 2);
        assert_eq!(order.wallet.charges.get(), 2);
    }

    #[test]
    fn a_function_like_proc_macro_generates_a_charge_in_the_consumer() {
        let wallet = Wallet::new(50);

        let balance = expansion_generator::generated_charge!(wallet);

        assert_eq!(balance, 37);
        assert_eq!(wallet.charges.get(), 1);
    }

    #[test]
    #[allow(array_into_iter, clippy::into_iter_on_ref)]
    fn an_exported_macro_keeps_the_method_tokens_edition() {
        let borrowed = [String::from("first"), String::from("second")];
        let consumed = [String::from("first"), String::from("second")];

        let legacy: std::slice::Iter<'_, String> = expansion_legacy::definition_items!(borrowed);
        let modern: std::array::IntoIter<String, 2> =
            expansion_legacy::callsite_items!(consumed, into_iter);
        let references: Vec<&String> = legacy.collect();
        let values: Vec<String> = modern.collect();

        assert_eq!(references.len(), 2);
        assert!(std::ptr::eq(references[0], &borrowed[0]));
        assert!(std::ptr::eq(references[1], &borrowed[1]));
        assert_eq!(values, ["first", "second"]);
    }

    use std::cell::Cell;

    struct Order {
        wallet: Wallet,
        reads: Cell<usize>,
    }

    impl Order {
        fn wallet(&self) -> &Wallet {
            self.reads.set(self.reads.get() + 1);
            &self.wallet
        }
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    fn selected_target() -> &'static str {
        "macos-arm64"
    }

    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    fn selected_target() -> &'static str {
        "other"
    }
}
