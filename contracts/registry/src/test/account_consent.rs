extern crate std;

use crate::{
    error::Error,
    events::RegisterAccount,
    test::contracts::fund_account,
    test::registry::{to_string, Registry},
};
use soroban_sdk::{
    self,
    auth::Context,
    auth::CustomAccountInterface,
    contract, contracterror, contractimpl,
    crypto::Hash,
    testutils::{Address as _, Events as _, Ledger as _},
    xdr::{
        InvokeContractArgs, ScAddress, ScSymbol, ScVal, SorobanAddressCredentials,
        SorobanAuthorizationEntry, SorobanAuthorizedFunction, SorobanAuthorizedInvocation,
        SorobanCredentials, VecM,
    },
    Address, Env, Event, IntoVal, TryFromVal, Val, Vec,
};

/// Smart-account fixture whose signature is a bool: `true` approves.
#[contract]
pub struct SmartAccount;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SmartAccountError {
    Denied = 1,
}

#[contractimpl]
impl CustomAccountInterface for SmartAccount {
    type Signature = bool;
    type Error = SmartAccountError;

    #[allow(non_snake_case, unused_variables)]
    fn __check_auth(
        _env: Env,
        signature_payload: Hash<32>,
        approve: bool,
        auth_contexts: Vec<Context>,
    ) -> Result<(), SmartAccountError> {
        if approve {
            Ok(())
        } else {
            Err(SmartAccountError::Denied)
        }
    }
}

/// Auth entry signed by a `SmartAccount`, so `require_auth` runs its
/// `__check_auth` instead of a mock.
fn smart_account_auth(
    env: &Env,
    registry: &Registry,
    signer: &Address,
    approve: bool,
    nonce: i64,
    fn_name: &str,
    args: impl IntoVal<Env, Vec<Val>>,
) -> SorobanAuthorizationEntry {
    let args: Vec<Val> = args.into_val(env);
    let args: std::vec::Vec<ScVal> = args
        .iter()
        .map(|v| ScVal::try_from_val(env, &v).unwrap())
        .collect();
    SorobanAuthorizationEntry {
        credentials: SorobanCredentials::Address(SorobanAddressCredentials {
            address: ScAddress::from(signer),
            nonce,
            signature_expiration_ledger: env.ledger().sequence() + 100,
            signature: ScVal::Bool(approve),
        }),
        root_invocation: SorobanAuthorizedInvocation {
            function: SorobanAuthorizedFunction::ContractFn(InvokeContractArgs {
                contract_address: ScAddress::from(&registry.client().address),
                function_name: ScSymbol(fn_name.try_into().unwrap()),
                args: args.try_into().unwrap(),
            }),
            sub_invocations: VecM::default(),
        },
    }
}

fn name(env: &Env) -> soroban_sdk::String {
    to_string(env, "my-account")
}

fn preauthorize(registry: &Registry, account: &Address) {
    let env = registry.env();
    // `mock_auths` can't mock a G-address, so mock everything and check who signed.
    env.mock_all_auths();
    registry.client().preauthorize_account(&name(env), account);
    assert!(env.auths().iter().any(|(a, _)| a == account));
}

#[test]
fn unmanaged_register_requires_named_address_auth() {
    let registry = Registry::new_non_root_unmanaged();
    let env = registry.env();
    let client = registry.client();
    let owner = Address::generate(env);
    let account = fund_account(env, 40);
    let args = (name(env), &account, &owner);

    registry.mock_auths_for(&[&owner], "register_account", args.clone());
    assert!(client
        .try_register_account(&name(env), &account, &owner)
        .is_err());

    env.mock_all_auths();
    client.register_account(&name(env), &account, &owner);
    assert!(env.auths().iter().any(|(a, _)| *a == account));
    assert_eq!(client.fetch_account_id(&name(env)), account);
}

#[test]
fn unmanaged_register_own_address_needs_one_signature() {
    let registry = Registry::new_non_root_unmanaged();
    let env = registry.env();
    let client = registry.client();
    let account = env.register(SmartAccount, ());

    env.set_auths(&[smart_account_auth(
        env,
        &registry,
        &account,
        true,
        1,
        "register_account",
        (name(env), &account, &account),
    )]);
    client.register_account(&name(env), &account, &account);
    assert_eq!(client.fetch_account_owner(&name(env)), account);
}

#[test]
fn managed_register_without_preauth_fails() {
    let registry = Registry::new_non_root_managed();
    let env = registry.env();
    let client = registry.client();
    let manager = client.manager().unwrap();
    let owner = Address::generate(env);
    let account = fund_account(env, 42);

    registry.mock_auths_for(
        &[&manager],
        "register_account",
        (name(env), &account, &owner),
    );
    assert!(client
        .try_register_account(&name(env), &account, &owner)
        .is_err());
}

#[test]
fn managed_register_consumes_preauth() {
    let registry = Registry::new_non_root_managed();
    let env = registry.env();
    let client = registry.client();
    let manager = client.manager().unwrap();
    let owner = Address::generate(env);
    let account = fund_account(env, 43);
    let args = (name(env), &account, &owner);

    preauthorize(&registry, &account);
    registry.mock_auths_for(&[&manager], "register_account", args.clone());
    client.register_account(&name(env), &account, &owner);
    assert_eq!(client.fetch_account_id(&name(env)), account);
    assert!(env.auths().iter().all(|(a, _)| *a != account));

    // Free the name again; the used preauth must not register it a second time.
    env.mock_all_auths();
    client.rename_account(&name(env), &to_string(env, "elsewhere"));
    registry.mock_auths_for(&[&manager], "register_account", args);
    assert!(client
        .try_register_account(&name(env), &account, &owner)
        .is_err());
}

#[test]
fn self_registration_consumes_preauth() {
    let registry = Registry::new_non_root_unmanaged();
    let env = registry.env();
    let client = registry.client();
    let account = fund_account(env, 54);

    preauthorize(&registry, &account);
    env.mock_all_auths();
    client.register_account(&name(env), &account, &account);
    client.rename_account(&name(env), &to_string(env, "elsewhere"));

    // The preauth was spent on the self-registration, so a stranger can't
    // reuse it to name `account` without a fresh signature.
    let stranger = Address::generate(env);
    registry.mock_auths_for(
        &[&stranger],
        "register_account",
        (name(env), &account, &stranger),
    );
    assert!(client
        .try_register_account(&name(env), &account, &stranger)
        .is_err());
}

#[test]
fn preauth_is_scoped_to_name_and_address() {
    let registry = Registry::new_non_root_managed();
    let env = registry.env();
    let client = registry.client();
    let manager = client.manager().unwrap();
    let owner = Address::generate(env);
    let account = fund_account(env, 44);
    let other = fund_account(env, 45);

    preauthorize(&registry, &account);

    registry.mock_auths_for(&[&manager], "register_account", (name(env), &other, &owner));
    assert!(client
        .try_register_account(&name(env), &other, &owner)
        .is_err());

    let other_name = to_string(env, "other-name");
    registry.mock_auths_for(
        &[&manager],
        "register_account",
        (&other_name, &account, &owner),
    );
    assert!(client
        .try_register_account(&other_name, &account, &owner)
        .is_err());
}

#[test]
fn expired_preauth_is_rejected() {
    let registry = Registry::new_non_root_managed();
    let env = registry.env();
    env.ledger().with_mut(|li| li.max_entry_ttl = 10_000);
    let client = registry.client();
    let manager = client.manager().unwrap();
    let owner = Address::generate(env);
    let account = fund_account(env, 46);

    preauthorize(&registry, &account);
    env.ledger().with_mut(|li| li.sequence_number += 10_000);

    // A fresh preauth still works, so the failure below is the expiry.
    let fresh = fund_account(env, 53);
    let fresh_name = to_string(env, "fresh");
    env.mock_all_auths();
    client.preauthorize_account(&fresh_name, &fresh);
    registry.mock_auths_for(
        &[&manager],
        "register_account",
        (&fresh_name, &fresh, &owner),
    );
    client.register_account(&fresh_name, &fresh, &owner);

    registry.mock_auths_for(
        &[&manager],
        "register_account",
        (name(env), &account, &owner),
    );
    assert!(client
        .try_register_account(&name(env), &account, &owner)
        .is_err());
}

#[test]
fn preauthorize_requires_named_address_auth() {
    let registry = Registry::new_non_root_managed();
    let env = registry.env();
    let account = fund_account(env, 47);
    let mallory = Address::generate(env);

    registry.mock_auths_for(&[&mallory], "preauthorize_account", (name(env), &account));
    assert!(registry
        .client()
        .try_preauthorize_account(&name(env), &account)
        .is_err());
}

#[test]
fn managed_update_address_requires_preauth() {
    let registry = Registry::new_non_root_managed();
    let env = registry.env();
    let client = registry.client();
    let manager = client.manager().unwrap();
    env.mock_all_auths();
    client.register_account(&name(env), &fund_account(env, 48), &Address::generate(env));
    let new_address = fund_account(env, 49);

    registry.mock_auths_for(
        &[&manager],
        "update_account_address",
        (name(env), &new_address),
    );
    assert!(client
        .try_update_account_address(&name(env), &new_address)
        .is_err());

    preauthorize(&registry, &new_address);
    registry.mock_auths_for(
        &[&manager],
        "update_account_address",
        (name(env), &new_address),
    );
    client.update_account_address(&name(env), &new_address);
    assert_eq!(client.fetch_account_id(&name(env)), new_address);
}

#[test]
fn unmanaged_update_address_requires_new_address_auth() {
    let registry = Registry::new_non_root_unmanaged();
    let env = registry.env();
    let client = registry.client();
    let owner = Address::generate(env);
    env.mock_all_auths();
    client.register_account(&name(env), &fund_account(env, 50), &owner);
    let new_address = fund_account(env, 51);
    let args = (name(env), &new_address);

    registry.mock_auths_for(&[&owner], "update_account_address", args.clone());
    assert!(client
        .try_update_account_address(&name(env), &new_address)
        .is_err());

    env.mock_all_auths();
    client.update_account_address(&name(env), &new_address);
    assert!(env.auths().iter().any(|(a, _)| *a == new_address));
    assert_eq!(client.fetch_account_id(&name(env)), new_address);
}

#[test]
fn smart_account_consents_via_check_auth() {
    let registry = Registry::new_non_root_unmanaged();
    let env = registry.env();
    let client = registry.client();
    let owner = env.register(SmartAccount, ());
    let account = env.register(SmartAccount, ());
    let args = (name(env), &account, &owner);

    env.set_auths(&[
        smart_account_auth(
            env,
            &registry,
            &owner,
            true,
            1,
            "register_account",
            args.clone(),
        ),
        smart_account_auth(
            env,
            &registry,
            &account,
            false,
            1,
            "register_account",
            args.clone(),
        ),
    ]);
    assert_eq!(
        client
            .try_register_account(&name(env), &account, &owner)
            .unwrap_err(),
        Err(soroban_sdk::InvokeError::Abort),
        "a smart account that rejects in __check_auth must not be named"
    );

    env.set_auths(&[
        smart_account_auth(
            env,
            &registry,
            &owner,
            true,
            2,
            "register_account",
            args.clone(),
        ),
        smart_account_auth(env, &registry, &account, true, 2, "register_account", args),
    ]);
    client.register_account(&name(env), &account, &owner);
    assert_eq!(client.fetch_account_id(&name(env)), account);
}

#[test]
fn smart_account_preauthorizes_managed_register() {
    let registry = Registry::new_non_root_managed();
    let env = registry.env();
    let client = registry.client();
    let manager = client.manager().unwrap();
    let owner = Address::generate(env);
    let account = env.register(SmartAccount, ());

    env.set_auths(&[smart_account_auth(
        env,
        &registry,
        &account,
        true,
        1,
        "preauthorize_account",
        (name(env), &account),
    )]);
    client.preauthorize_account(&name(env), &account);

    registry.mock_auths_for(
        &[&manager],
        "register_account",
        (name(env), &account, &owner),
    );
    client.register_account(&name(env), &account, &owner);
    assert_eq!(client.fetch_account_id(&name(env)), account);
}

#[test]
fn register_account_event_includes_owner() {
    let registry = Registry::new_non_root_unmanaged();
    let env = registry.env();
    let client = registry.client();
    let owner = Address::generate(env);
    let account = fund_account(env, 52);

    env.mock_all_auths();
    client.register_account(&name(env), &account, &owner);

    let expected = RegisterAccount {
        account_name: name(env),
        account_id: account,
        owner,
    }
    .to_xdr(env, &client.address);
    assert!(env.events().all().events().contains(&expected));
}

#[test]
fn preauthorize_invalid_name_fails() {
    let registry = Registry::new_non_root_unmanaged();
    let env = registry.env();
    assert_eq!(
        registry
            .client()
            .try_preauthorize_account(&to_string(env, ""), &Address::generate(env))
            .unwrap_err(),
        Ok(Error::InvalidName)
    );
}
