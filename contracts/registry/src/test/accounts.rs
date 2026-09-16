use crate::{
    error::Error,
    test::contracts::{fund_account, hello_world, hw_bytes, hw_hash},
    test::registry::{to_string, Registry},
};
use soroban_sdk::{self, testutils::Address as _, testutils::Register, Address};

fn deploy_hw(env: &soroban_sdk::Env, owner: &Address) -> Address {
    hello_world::WASM.register(env, None, hello_world::Args::__constructor(owner))
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
