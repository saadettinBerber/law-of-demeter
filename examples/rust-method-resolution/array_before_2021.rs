#[test]
#[allow(array_into_iter, clippy::into_iter_on_ref)]
fn array_method_iteration_yields_references_before_edition_2021() {
    let wallets = [Wallet { balance: 50 }, Wallet { balance: 90 }];

    let iterator: std::slice::Iter<'_, Wallet> = wallets.into_iter();
    let returned: Vec<&Wallet> = iterator.collect();

    assert!(std::ptr::eq(returned[0], &wallets[0]));
    assert!(std::ptr::eq(returned[1], &wallets[1]));
    assert_eq!((returned[0].balance, returned[1].balance), (50, 90));
}

struct Wallet {
    balance: i32,
}
