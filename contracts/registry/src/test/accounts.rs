use crate::{
    error::Error,
    test::contracts::{fund_account, hello_world, hw_bytes, hw_hash},
    test::registry::{to_string, Registry},
};
use soroban_sdk::{self, testutils::Address as _, testutils::Register, Address};

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
