use std::cell::Cell;

#[test]
fn option_projections_and_consumption_keep_the_boxed_wallet() {
    let wallet = Box::new(Wallet { balance: 50 });
    let address = &*wallet as *const Wallet;
    let mut optional = Some(wallet);

    let shared: Option<&Box<Wallet>> = optional.as_ref();
    let shared = shared.expect("the wallet is present");
    let shared_address = &**shared as *const Wallet;
    let before_write = shared.balance;
    let mutable: Option<&mut Box<Wallet>> = optional.as_mut();
    mutable.expect("the wallet is present").balance = 30;
    let owned: Box<Wallet> = optional.expect("the wallet is present");

    assert_eq!(shared_address, address);
    assert!(std::ptr::eq(&*owned, address));
    assert_eq!((before_write, owned.balance), (50, 30));
}

#[test]
fn a_typed_none_skips_its_mapping_callback() {
    let optional: Option<&Wallet> = None;
    let calls = Cell::new(0);

    let mapped: Option<i32> = optional.map(|wallet| {
        calls.set(calls.get() + 1);
        wallet.balance
    });

    assert_eq!(mapped, None);
    assert_eq!(calls.get(), 0);
}

#[test]
fn result_projections_keep_success_and_error_targets_separate() {
    let success_wallet = Box::new(Wallet { balance: 50 });
    let error_wallet = Box::new(Wallet { balance: 50 });
    let success_address = &*success_wallet as *const Wallet;
    let error_address = &*error_wallet as *const Wallet;
    let mut success: Result<Box<Wallet>, Box<Wallet>> = Ok(success_wallet);
    let mut failure: Result<Box<Wallet>, Box<Wallet>> = Err(error_wallet);

    let shared_success: Result<&Box<Wallet>, &Box<Wallet>> = success.as_ref();
    let shared_error: Result<&Box<Wallet>, &Box<Wallet>> = failure.as_ref();
    let borrowed_success = &**shared_success.expect("success has a wallet") as *const Wallet;
    let borrowed_error = &**shared_error.expect_err("failure has a wallet") as *const Wallet;
    let mutable_success: Result<&mut Box<Wallet>, &mut Box<Wallet>> = success.as_mut();
    mutable_success.expect("success has a wallet").balance = 20;
    let after_success_write = failure.as_ref().expect_err("failure has a wallet").balance;
    let mutable_error: Result<&mut Box<Wallet>, &mut Box<Wallet>> = failure.as_mut();
    mutable_error.expect_err("failure has a wallet").balance = 30;

    assert_eq!(borrowed_success, success_address);
    assert_eq!(borrowed_error, error_address);
    assert_ne!(borrowed_success, borrowed_error);
    assert_eq!(after_success_write, 50);
    assert_eq!(success.expect("success has a wallet").balance, 20);
    assert_eq!(failure.expect_err("failure has a wallet").balance, 30);
}

#[test]
fn map_and_map_err_run_only_for_their_respective_result_roles() {
    let success_wallet = Wallet { balance: 50 };
    let error_wallet = Wallet { balance: 50 };
    let success: Result<&Wallet, &Wallet> = Ok(&success_wallet);
    let failure: Result<&Wallet, &Wallet> = Err(&error_wallet);
    let success_calls = Cell::new(0);
    let error_calls = Cell::new(0);

    let mapped_success = success
        .map(|wallet| {
            success_calls.set(success_calls.get() + 1);
            (wallet, wallet.balance)
        })
        .map_err(|wallet| {
            error_calls.set(error_calls.get() + 1);
            (wallet, wallet.balance)
        });
    let after_success = (success_calls.get(), error_calls.get());
    let mapped_error = failure
        .map(|wallet| {
            success_calls.set(success_calls.get() + 1);
            (wallet, wallet.balance)
        })
        .map_err(|wallet| {
            error_calls.set(error_calls.get() + 1);
            (wallet, wallet.balance)
        });

    assert_eq!(after_success, (1, 0));
    assert_eq!((success_calls.get(), error_calls.get()), (1, 1));
    let (kept_success, success_balance) = mapped_success.expect("success stays successful");
    let (kept_error, error_balance) = mapped_error.expect_err("failure stays an error");
    assert!(std::ptr::eq(kept_success, &success_wallet));
    assert!(std::ptr::eq(kept_error, &error_wallet));
    assert_eq!((success_balance, error_balance), (50, 50));
}

#[test]
fn and_then_can_replace_success_and_keeps_an_existing_error() {
    let success_wallet = Wallet { balance: 50 };
    let incoming_error = Wallet { balance: 50 };
    let replacement_error = Wallet { balance: 50 };
    let success: Result<&Wallet, &Wallet> = Ok(&success_wallet);
    let failure: Result<&Wallet, &Wallet> = Err(&incoming_error);
    let calls = Cell::new(0);

    let replaced: Result<&Wallet, &Wallet> = success.and_then(|_| {
        calls.set(calls.get() + 1);
        Err(&replacement_error)
    });
    let after_success = calls.get();
    let retained: Result<&Wallet, &Wallet> = failure.and_then(|_| {
        calls.set(calls.get() + 1);
        Err(&replacement_error)
    });

    assert_eq!((after_success, calls.get()), (1, 1));
    assert!(std::ptr::eq(
        replaced.expect_err("callback replaces success"),
        &replacement_error
    ));
    assert!(std::ptr::eq(
        retained.expect_err("existing error is retained"),
        &incoming_error
    ));
}

#[test]
fn an_error_has_no_success_items_and_keeps_its_known_error() {
    let error_wallet = Wallet { balance: 50 };
    let failure: Result<Wallet, &Wallet> = Err(&error_wallet);

    let iterator: std::result::Iter<'_, Wallet> = failure.iter();
    let success_items: Vec<&Wallet> = iterator.collect();
    let error: &Wallet = failure.as_ref().expect_err("the result has an error");

    assert!(success_items.is_empty());
    assert!(std::ptr::eq(error, &error_wallet));
    assert_eq!(error.balance, 50);
}

#[derive(Debug)]
struct Wallet {
    balance: i32,
}
