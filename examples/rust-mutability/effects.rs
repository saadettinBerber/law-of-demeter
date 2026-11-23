use std::cell::{Cell, RefCell};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockWriteGuard};

#[test]
fn acquiring_a_mutex_guard_can_preserve_the_protected_value() -> Result<(), &'static str> {
    let wallet = LockedWallet {
        balance: Mutex::new(50),
    };

    let while_held = {
        let guard = wallet.balance_guard()?;
        *guard
    };
    let after_release = *wallet.balance_guard()?;

    assert_eq!(while_held, 50);
    assert_eq!(after_release, 50);
    Ok(())
}

#[test]
fn a_mutex_guard_can_change_balance_seen_through_an_arc() -> Result<(), &'static str> {
    let wallet = Arc::new(LockedWallet {
        balance: Mutex::new(50),
    });
    let alias = Arc::clone(&wallet);

    let before_write = {
        let mut guard = alias.balance_guard()?;
        let before = *guard;
        *guard = 30;
        before
    };
    let after_write = *wallet.balance_guard()?;

    assert!(Arc::ptr_eq(&wallet, &alias));
    assert_eq!(before_write, 50);
    assert_eq!(after_write, 30);
    Ok(())
}

#[test]
fn acquiring_an_rwlock_write_guard_can_preserve_the_value() -> Result<(), &'static str> {
    let wallet = ReadWriteWallet {
        balance: RwLock::new(50),
    };

    let while_held = {
        let guard = wallet.balance_guard()?;
        *guard
    };
    let after_release = wallet.balance()?;

    assert_eq!(while_held, 50);
    assert_eq!(after_release, 50);
    Ok(())
}

#[test]
fn a_returned_rwlock_write_guard_can_change_shared_balance() -> Result<(), &'static str> {
    let wallet = ReadWriteWallet {
        balance: RwLock::new(50),
    };
    let shared = &wallet;

    let before_write = {
        let mut guard = shared.balance_guard()?;
        let before = *guard;
        *guard = 30;
        before
    };
    let after_write = wallet.balance()?;

    assert_eq!(before_write, 50);
    assert_eq!(after_write, 30);
    Ok(())
}

#[test]
fn cloning_an_rc_changes_the_owner_count_without_changing_the_stored_integer() {
    let balance = Rc::new(50);
    let owners_before = Rc::strong_count(&balance);

    let alias = Rc::clone(&balance);
    let owners_after = Rc::strong_count(&balance);
    let stored_values = (*balance, *alias);

    assert!(Rc::ptr_eq(&balance, &alias));
    assert_eq!(owners_before, 1);
    assert_eq!(owners_after, 2);
    assert_eq!(stored_values, (50, 50));
}

#[test]
fn cloning_an_arc_changes_the_owner_count_without_changing_the_stored_integer() {
    let balance = Arc::new(50);
    let owners_before = Arc::strong_count(&balance);

    let alias = Arc::clone(&balance);
    let owners_after = Arc::strong_count(&balance);
    let stored_values = (*balance, *alias);

    assert!(Arc::ptr_eq(&balance, &alias));
    assert_eq!(owners_before, 1);
    assert_eq!(owners_after, 2);
    assert_eq!(stored_values, (50, 50));
}

#[test]
fn a_returned_mutable_reference_allows_writing_the_owners_state() {
    let mut wallet = Wallet { balance: 50 };

    let before_write = {
        let balance = wallet.balance_mut();
        let before = *balance;
        *balance = 30;
        before
    };
    let after_write = wallet.balance();

    assert_eq!(before_write, 50);
    assert_eq!(after_write, 30);
}

#[test]
fn a_returned_shared_handle_exposes_mutation_without_changing_the_balance_on_return() {
    let wallet = SharedWallet {
        balance: Rc::new(RefCell::new(50)),
    };
    let shared = &wallet;

    let handle = wallet.balance_handle();
    let after_return = shared.balance();
    *handle.borrow_mut() = 30;
    let after_write = shared.balance();

    assert_eq!(after_return, 50);
    assert_eq!(after_write, 30);
}

#[test]
fn warming_a_cache_changes_administrative_state_without_changing_the_logical_observation() {
    let name = CachedName::new("Ada");
    let expected = CachedName::new("Ada");
    let shared = &name;
    let cached_before = name.is_cached();

    let byte_count = shared.byte_count();
    let cached_after = name.is_cached();
    let cached_byte_count = shared.byte_count();
    let text_after = name.text();

    assert!(!cached_before);
    assert!(cached_after);
    assert_eq!(byte_count, 3);
    assert_eq!(cached_byte_count, 3);
    assert_eq!(text_after, "Ada");
    assert_eq!(name, expected);
    assert_eq!(hash_of(&name), hash_of(&expected));
    assert!(!expected.is_cached());
}

fn hash_of(value: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

struct LockedWallet {
    balance: Mutex<i32>,
}

impl LockedWallet {
    fn balance_guard(&self) -> Result<MutexGuard<'_, i32>, &'static str> {
        self.balance.lock().map_err(|_| "balance mutex poisoned")
    }
}

struct ReadWriteWallet {
    balance: RwLock<i32>,
}

impl ReadWriteWallet {
    fn balance_guard(&self) -> Result<RwLockWriteGuard<'_, i32>, &'static str> {
        self.balance.write().map_err(|_| "balance rwlock poisoned")
    }

    fn balance(&self) -> Result<i32, &'static str> {
        self.balance
            .read()
            .map(|guard| *guard)
            .map_err(|_| "balance rwlock poisoned")
    }
}

struct Wallet {
    balance: i32,
}

impl Wallet {
    fn balance_mut(&mut self) -> &mut i32 {
        &mut self.balance
    }

    fn balance(&self) -> i32 {
        self.balance
    }
}

struct SharedWallet {
    balance: Rc<RefCell<i32>>,
}

impl SharedWallet {
    fn balance_handle(&self) -> Rc<RefCell<i32>> {
        Rc::clone(&self.balance)
    }

    fn balance(&self) -> i32 {
        *self.balance.borrow()
    }
}

#[derive(Debug)]
struct CachedName {
    text: String,
    byte_count_cache: Cell<Option<usize>>,
}

impl CachedName {
    fn new(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            byte_count_cache: Cell::new(None),
        }
    }

    fn byte_count(&self) -> usize {
        if let Some(count) = self.byte_count_cache.get() {
            return count;
        }
        let count = self.text.len();
        self.byte_count_cache.set(Some(count));
        count
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn is_cached(&self) -> bool {
        self.byte_count_cache.get().is_some()
    }
}

impl PartialEq for CachedName {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
    }
}

impl Eq for CachedName {}

impl Hash for CachedName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text.hash(state);
    }
}
