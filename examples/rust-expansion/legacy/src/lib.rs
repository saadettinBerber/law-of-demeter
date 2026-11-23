#[macro_export]
macro_rules! charge_twice {
    ($owner:expr) => {{
        let first = $owner.wallet();
        first.charge(7);
        let after_first = first.balance();
        let second = $owner.wallet();
        second.charge(11);
        (after_first, second.balance())
    }};
}

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
