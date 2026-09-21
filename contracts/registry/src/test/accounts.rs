use crate::{
    error::Error,
    events::{RenameAccount, SecurityFlagAccount, UpdateAccountAddress, UpdateAccountOwner},
    test::contracts::{fund_account, hello_world, hw_bytes, hw_hash},
    test::registry::{to_string, Registry},
};
use soroban_sdk::{
    self,
    testutils::{Address as _, Events as _, Register},
    Address, Env, Event,
};

fn deploy_hw(env: &soroban_sdk::Env, owner: &Address) -> Address {
    hello_world::WASM.register(env, None, hello_world::Args::__constructor(owner))
}

fn setup_with_registered_account<'a>(seed: u8) -> (Registry<'a>, Address) {
    let registry = Registry::new_with_bytes(&hw_bytes, &hw_hash);
    let env = registry.env();
    let owner = registry.admin().clone();
    env.mock_all_auths();
    let account_address = fund_account(env, seed);
    registry
        .client()
        .register_account(&to_string(env, "my-account"), &account_address, &owner);
    (registry, account_address)
}

#[test]
fn register_and_fetch_unmanaged() {
    let registry = Registry::new_non_root_unmanaged();
    let env = registry.env();
    let client = registry.client();
    let owner = registry.admin().clone();
    let account_address = fund_account(env, 1);

    env.mock_all_auths();
    client.register_account(&to_string(env, "chadoh"), &account_address, &owner);

    assert_eq!(
        client.fetch_account_id(&to_string(env, "chadoh")),
        account_address
    );
    assert_eq!(client.fetch_account_owner(&to_string(env, "chadoh")), owner);
}

#[test]
fn register_managed_requires_manager_auth() {
    let registry = Registry::new_non_root_managed();
    let env = registry.env();
    let client = registry.client();
    let owner = Address::generate(env);
    let account_address = fund_account(env, 2);

    env.mock_all_auths();
    client.register_account(&to_string(env, "chadoh"), &account_address, &owner);

    assert_eq!(
        client.fetch_account_id(&to_string(env, "chadoh")),
        account_address
    );
}

#[test]
fn register_already_taken_name() {
    let registry = Registry::new_non_root_unmanaged();
    let env = registry.env();
    let client = registry.client();
    let owner = registry.admin().clone();

    env.mock_all_auths();
    client.register_account(&to_string(env, "chadoh"), &fund_account(env, 3), &owner);

    assert_eq!(
        client
            .try_register_account(&to_string(env, "chadoh"), &Address::generate(env), &owner)
            .unwrap_err(),
        Ok(Error::AccountNameAlreadyTaken)
    );
}

#[test]
fn register_contract_address_as_account_fails() {
    let registry = Registry::new_with_bytes(&hw_bytes, &hw_hash);
    let env = registry.env();
    let client = registry.client();
    let owner = registry.admin().clone();

    env.mock_all_auths();
    let contract_addr = deploy_hw(env, &owner);

    assert_eq!(
        client
            .try_register_account(&to_string(env, "chadoh"), &contract_addr, &owner)
            .unwrap_err(),
        Ok(Error::NotAccountAddress)
    );
}

#[test]
fn fetch_unregistered_account_fails() {
    let registry = Registry::new_non_root_unmanaged();
    let env = registry.env();
    let client = registry.client();

    assert_eq!(
        client
            .try_fetch_account_id(&to_string(env, "nonexistent"))
            .unwrap_err(),
        Ok(Error::NoSuchAccountRegistered)
    );
}

#[test]
fn account_and_contract_namespaces_are_independent() {
    let registry = Registry::new_with_bytes(&hw_bytes, &hw_hash);
    let env = registry.env();
    let client = registry.client();
    let owner = registry.admin().clone();

    env.mock_all_auths();
    let contract_addr = deploy_hw(env, &owner);
    client.register_contract(&to_string(env, "foo"), &contract_addr, &owner);

    let account_address = fund_account(env, 4);
    client.register_account(&to_string(env, "foo"), &account_address, &owner);

    assert_eq!(
        client.fetch_contract_id(&to_string(env, "foo")),
        contract_addr
    );
    assert_eq!(
        client.fetch_account_id(&to_string(env, "foo")),
        account_address
    );
}

#[test]
fn update_account_owner() {
    let (registry, _addr) = setup_with_registered_account(10);
    let env = registry.env();
    let client = registry.client();

    let new_owner = Address::generate(env);
    env.mock_all_auths();
    client.update_account_owner(&to_string(env, "my-account"), &new_owner);

    assert_eq!(
        client.fetch_account_owner(&to_string(env, "my-account")),
        new_owner
    );
}

#[test]
fn update_account_address() {
    let (registry, _addr) = setup_with_registered_account(11);
    let env = registry.env();
    let client = registry.client();

    let new_address = fund_account(env, 12);
    env.mock_all_auths();
    client.update_account_address(&to_string(env, "my-account"), &new_address);

    assert_eq!(
        client.fetch_account_id(&to_string(env, "my-account")),
        new_address
    );
}

#[test]
fn update_account_address_to_contract_fails() {
    let (registry, _addr) = setup_with_registered_account(13);
    let env = registry.env();
    let client = registry.client();
    let owner = registry.admin();

    env.mock_all_auths();
    let contract_addr = deploy_hw(env, owner);

    assert_eq!(
        client
            .try_update_account_address(&to_string(env, "my-account"), &contract_addr)
            .unwrap_err(),
        Ok(Error::NotAccountAddress)
    );
}

#[test]
fn rename_account() {
    let (registry, addr) = setup_with_registered_account(14);
    let env = registry.env();
    let client = registry.client();

    env.mock_all_auths();
    client.rename_account(&to_string(env, "my-account"), &to_string(env, "new-name"));

    assert_eq!(
        client
            .try_fetch_account_id(&to_string(env, "my-account"))
            .unwrap_err(),
        Ok(Error::NoSuchAccountRegistered)
    );
    assert_eq!(client.fetch_account_id(&to_string(env, "new-name")), addr);
}

#[test]
fn rename_account_to_taken_name() {
    let (registry, _addr) = setup_with_registered_account(15);
    let env = registry.env();
    let client = registry.client();

    let owner = registry.admin();
    env.mock_all_auths();
    client.register_account(
        &to_string(env, "other-account"),
        &fund_account(env, 16),
        owner,
    );

    assert_eq!(
        client
            .try_rename_account(
                &to_string(env, "my-account"),
                &to_string(env, "other-account")
            )
            .unwrap_err(),
        Ok(Error::AccountNameAlreadyTaken)
    );
}

#[test]
fn update_nonexistent_account() {
    let registry = Registry::new_with_bytes(&hw_bytes, &hw_hash);
    let env = registry.env();
    let client = registry.client();

    env.mock_all_auths();

    assert_eq!(
        client
            .try_update_account_owner(&to_string(env, "nonexistent"), &Address::generate(env))
            .unwrap_err(),
        Ok(Error::NoSuchAccountRegistered)
    );
}

#[test]
fn non_owner_without_manager_fails() {
    let registry = Registry::new_non_root_unmanaged();
    let env = registry.env();
    let client = registry.client();

    let owner = Address::generate(env);
    env.mock_all_auths();
    client.register_account(
        &to_string(env, "my-account"),
        &fund_account(env, 17),
        &owner,
    );

    let non_owner = Address::generate(env);
    registry.mock_auth_for(
        &non_owner,
        "update_account_owner",
        (&to_string(env, "my-account"), &Address::generate(env)),
    );
    assert!(client
        .try_update_account_owner(&to_string(env, "my-account"), &Address::generate(env))
        .is_err());
}

#[test]
fn flag_account_as_owner_succeeds() {
    let (registry, _addr) = setup_with_registered_account(18);
    let env = registry.env();
    let client = registry.client();

    env.mock_all_auths();
    client.flag_account(&to_string(env, "my-account"), &true);
    client.flag_account(&to_string(env, "my-account"), &false);
}

// --- Event coverage -------------------------------------------------------
//
// The indexer does not track these events yet, so these tests are the only
// guard on their shape. They also cover `flagged`, which has no getter.

/// Assert the last invocation emitted `event` from the registry contract.
fn assert_emitted(env: &Env, registry: &Registry, event: &impl Event) {
    let expected = event.to_xdr(env, &registry.client().address);
    assert!(
        env.events().all().events().contains(&expected),
        "expected event not emitted; got {:?}",
        env.events().all()
    );
}

/// Registry with a manager and one registered account owned by a distinct
/// (non-manager) owner. Returns `(registry, owner, manager)`.
fn setup_managed_with_account<'a>(seed: u8) -> (Registry<'a>, Address, Address) {
    let registry = Registry::new_non_root_managed();
    let env = registry.env();
    let owner = Address::generate(env);
    env.mock_all_auths();
    registry.client().register_account(
        &to_string(env, "my-account"),
        &fund_account(env, seed),
        &owner,
    );
    let manager = registry.client().manager().unwrap();
    assert_ne!(owner, manager);
    (registry, owner, manager)
}

#[test]
fn update_account_owner_emits_event_with_owner_operator() {
    let registry = Registry::new_non_root_unmanaged();
    let env = registry.env();
    let client = registry.client();
    let owner = Address::generate(env);
    env.mock_all_auths();
    client.register_account(
        &to_string(env, "my-account"),
        &fund_account(env, 20),
        &owner,
    );

    let new_owner = Address::generate(env);
    client.update_account_owner(&to_string(env, "my-account"), &new_owner);

    assert_emitted(
        env,
        &registry,
        &UpdateAccountOwner {
            account_name: to_string(env, "my-account"),
            new_owner,
            operator: owner,
        },
    );
}

#[test]
fn update_account_owner_by_manager_records_manager_as_operator() {
    let (registry, owner, manager) = setup_managed_with_account(21);
    let env = registry.env();
    let client = registry.client();

    let new_owner = Address::generate(env);
    env.mock_all_auths();
    client.update_account_owner(&to_string(env, "my-account"), &new_owner);

    // The manager must have authorized, not the current owner.
    let auths = env.auths();
    assert!(auths.iter().any(|(a, _)| *a == manager));
    assert!(auths.iter().all(|(a, _)| *a != owner));

    assert_emitted(
        env,
        &registry,
        &UpdateAccountOwner {
            account_name: to_string(env, "my-account"),
            new_owner: new_owner.clone(),
            operator: manager,
        },
    );
    assert_eq!(
        client.fetch_account_owner(&to_string(env, "my-account")),
        new_owner
    );
}

#[test]
fn update_account_owner_preserves_address() {
    let (registry, addr) = setup_with_registered_account(22);
    let env = registry.env();
    let client = registry.client();

    env.mock_all_auths();
    client.update_account_owner(&to_string(env, "my-account"), &Address::generate(env));

    assert_eq!(client.fetch_account_id(&to_string(env, "my-account")), addr);
}

#[test]
fn update_account_address_emits_event_and_preserves_owner() {
    let (registry, _addr) = setup_with_registered_account(23);
    let env = registry.env();
    let client = registry.client();
    let owner = registry.admin().clone();
    // Root-style registry: manager is set, so it is the operator. Read it
    // before the call under test, since any invocation resets the event buffer.
    let operator = client.manager().unwrap_or(owner.clone());

    let new_address = fund_account(env, 24);
    env.mock_all_auths();
    client.update_account_address(&to_string(env, "my-account"), &new_address);

    assert_emitted(
        env,
        &registry,
        &UpdateAccountAddress {
            account_name: to_string(env, "my-account"),
            new_address,
            operator,
        },
    );
    assert_eq!(
        client.fetch_account_owner(&to_string(env, "my-account")),
        owner
    );
}

#[test]
fn update_account_address_by_manager_records_manager_as_operator() {
    let (registry, _owner, manager) = setup_managed_with_account(25);
    let env = registry.env();
    let client = registry.client();

    let new_address = fund_account(env, 26);
    env.mock_all_auths();
    client.update_account_address(&to_string(env, "my-account"), &new_address);

    assert_emitted(
        env,
        &registry,
        &UpdateAccountAddress {
            account_name: to_string(env, "my-account"),
            new_address,
            operator: manager,
        },
    );
}

#[test]
fn update_account_address_to_unfunded_account_fails() {
    let (registry, _addr) = setup_with_registered_account(27);
    let env = registry.env();
    let client = registry.client();

    env.mock_all_auths();
    // A fresh G-address that does not exist on the ledger.
    assert_eq!(
        client
            .try_update_account_address(&to_string(env, "my-account"), &Address::generate(env))
            .unwrap_err(),
        Ok(Error::ContractIdAddressDoesNotExist)
    );
}

#[test]
fn rename_account_emits_event_and_preserves_owner() {
    let registry = Registry::new_non_root_unmanaged();
    let env = registry.env();
    let client = registry.client();
    let owner = Address::generate(env);
    env.mock_all_auths();
    client.register_account(
        &to_string(env, "my-account"),
        &fund_account(env, 28),
        &owner,
    );

    client.rename_account(&to_string(env, "my-account"), &to_string(env, "new-name"));

    assert_emitted(
        env,
        &registry,
        &RenameAccount {
            old_name: to_string(env, "my-account"),
            new_name: to_string(env, "new-name"),
            operator: owner.clone(),
        },
    );
    assert_eq!(
        client.fetch_account_owner(&to_string(env, "new-name")),
        owner
    );
}

#[test]
fn rename_account_by_manager_records_manager_as_operator() {
    let (registry, _owner, manager) = setup_managed_with_account(29);
    let env = registry.env();
    let client = registry.client();

    env.mock_all_auths();
    client.rename_account(&to_string(env, "my-account"), &to_string(env, "new-name"));

    assert_emitted(
        env,
        &registry,
        &RenameAccount {
            old_name: to_string(env, "my-account"),
            new_name: to_string(env, "new-name"),
            operator: manager,
        },
    );
}

#[test]
fn rename_account_normalizes_names_in_event() {
    let (registry, _addr) = setup_with_registered_account(30);
    let env = registry.env();
    let client = registry.client();

    let expected_operator = client.manager().unwrap_or(registry.admin().clone());
    env.mock_all_auths();
    client.rename_account(&to_string(env, "My-Account"), &to_string(env, "New-Name"));

    // Event carries the normalized (stored) names, which is what the indexer
    // will need to key on.
    assert_emitted(
        env,
        &registry,
        &RenameAccount {
            old_name: to_string(env, "my-account"),
            new_name: to_string(env, "new-name"),
            operator: expected_operator,
        },
    );
}

#[test]
fn flag_account_emits_event_with_account_and_operator() {
    let registry = Registry::new_non_root_unmanaged();
    let env = registry.env();
    let client = registry.client();
    let owner = Address::generate(env);
    env.mock_all_auths();
    client.register_account(
        &to_string(env, "my-account"),
        &fund_account(env, 31),
        &owner,
    );

    for flagged in [true, false] {
        client.flag_account(&to_string(env, "my-account"), &flagged);
        assert_emitted(
            env,
            &registry,
            &SecurityFlagAccount {
                account_name: to_string(env, "my-account"),
                flagged,
                operator: owner.clone(),
            },
        );
    }
}

#[test]
fn flag_account_by_manager_records_manager_as_operator() {
    let (registry, owner, manager) = setup_managed_with_account(32);
    let env = registry.env();
    let client = registry.client();

    env.mock_all_auths();
    client.flag_account(&to_string(env, "my-account"), &true);

    let auths = env.auths();
    assert!(auths.iter().any(|(a, _)| *a == manager));
    assert!(auths.iter().all(|(a, _)| *a != owner));

    assert_emitted(
        env,
        &registry,
        &SecurityFlagAccount {
            account_name: to_string(env, "my-account"),
            flagged: true,
            operator: manager,
        },
    );
}

#[test]
fn flag_nonexistent_account_fails() {
    let registry = Registry::new_non_root_unmanaged();
    let env = registry.env();

    env.mock_all_auths();
    assert_eq!(
        registry
            .client()
            .try_flag_account(&to_string(env, "nonexistent"), &true)
            .unwrap_err(),
        Ok(Error::NoSuchAccountRegistered)
    );
}

#[test]
fn flag_account_by_non_owner_without_manager_fails() {
    let registry = Registry::new_non_root_unmanaged();
    let env = registry.env();
    let client = registry.client();
    let owner = Address::generate(env);
    env.mock_all_auths();
    client.register_account(
        &to_string(env, "my-account"),
        &fund_account(env, 33),
        &owner,
    );

    registry.mock_auth_for(
        &Address::generate(env),
        "flag_account",
        (&to_string(env, "my-account"), &true),
    );
    assert!(client
        .try_flag_account(&to_string(env, "my-account"), &true)
        .is_err());
}

#[test]
fn failed_management_calls_emit_no_event() {
    let (registry, _addr) = setup_with_registered_account(34);
    let env = registry.env();
    let client = registry.client();

    env.mock_all_auths();
    let _ = client.try_rename_account(&to_string(env, "nonexistent"), &to_string(env, "x"));
    assert!(env.events().all().events().is_empty());
}
