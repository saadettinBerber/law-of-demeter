#[test]
#[allow(array_into_iter)]
fn array_method_iteration_yields_owned_values_from_edition_2021() {
    let wallets = [Wallet { balance: 50 }, Wallet { balance: 90 }];

    let iterator: std::array::IntoIter<Wallet, 2> = wallets.into_iter();
    let balances: Vec<i32> = iterator.map(|wallet| wallet.balance).collect();

    assert_eq!(balances, [50, 90]);
}

struct Wallet {
    balance: i32,
}
