use crate::test::contracts::{self, hw_bytes, immutable_bytes, immutable_hash};
use crate::test::registry::{to_string, Registry};
use soroban_sdk::{self, testutils::Address as _, Address};

/// `deploy_stateless` salts with the wasm hash, so the deployed address is a
/// pure function of `(deployer, wasm_hash)` and matches the id an offline
/// caller derives with `with_address(registry, hash).deployed_address()`. A
/// second call is an idempotent no-op returning the same address. The
/// `immutable_hello` fixture has a zero-arg `__constructor`, proving `init = ()`
/// is enough.
#[test]
fn deploy_stateless_is_content_addressed_and_idempotent() {
    let registry = &Registry::new_unverified();
    let env = registry.env();
    let client = registry.client();
    let version = registry.default_version();

    let author = &Address::generate(env);
    let wasm_name = &to_string(env, "immutable");

    // Publish the immutable wasm (its `__constructor` takes no args).
    registry.mock_auth_with_addresses_for_publish(
        wasm_name,
        author,
        &Some(version.clone()),
        &immutable_bytes(env),
        &[author],
    );
    client.publish(wasm_name, author, &immutable_bytes(env), &version);

    let hash = immutable_hash(env);
    assert_eq!(client.fetch_hash(wasm_name, &None), hash);

    // salt == wasm hash, deployer defaults to the registry contract itself.
    let address = client.deploy_stateless(wasm_name, &None, &None);

    // The returned id equals the address derived offline from
    // `(registry_address, wasm_hash)` — i.e. the salt really is the wasm hash.
    let expected = env
        .deployer()
        .with_address(client.address.clone(), hash.clone())
        .deployed_address();
    assert_eq!(address, expected);

    // The contract actually deployed and runs (so `init = ()` was accepted).
    let hw = contracts::immutable_client(env, &address);
    assert_eq!(hw.hello(), to_string(env, "stateless!"));

    // A second call is a no-op success returning the same address — never
    // `AlreadyDeployed`.
    let again = client.deploy_stateless(wasm_name, &None, &None);
    assert_eq!(again, address);

    // And `try_` form confirms the repeat is `Ok`, not a trap/error.
    let again_try = client.try_deploy_stateless(wasm_name, &None, &None);
    assert_eq!(again_try, Ok(Ok(address)));
}

/// `deploy_stateless` never forwards constructor arguments. Deploying a wasm
/// whose `__constructor` *requires* an argument (`hello_world` needs `admin`)
/// therefore traps — the self-enforcing "deployer-independent" guarantee.
#[test]
fn deploy_stateless_never_passes_init_args() {
    let registry = &Registry::new_unverified();
    let env = registry.env();
    let client = registry.client();
    let version = registry.default_version();

    let author = &Address::generate(env);
    let wasm_name = &to_string(env, "needs-args");

    // `hello_world`'s `__constructor(env, admin)` requires an argument.
    registry.mock_auth_with_addresses_for_publish(
        wasm_name,
        author,
        &Some(version.clone()),
        &hw_bytes(env),
        &[author],
    );
    client.publish(wasm_name, author, &hw_bytes(env), &version);

    // Because no init args are ever passed, the arg-requiring constructor traps.
    let res = client.try_deploy_stateless(wasm_name, &None, &None);
    assert!(
        res.is_err(),
        "expected the arg-requiring constructor to trap"
    );
}

/// An explicit deployer is honored: the derived id is namespaced by that
/// deployer's address, and re-deploying from the same deployer is idempotent.
#[test]
fn deploy_stateless_honors_explicit_deployer() {
    let registry = &Registry::new_unverified();
    let env = registry.env();
    let client = registry.client();
    let version = registry.default_version();

    let author = &Address::generate(env);
    let deployer = &Address::generate(env);
    let wasm_name = &to_string(env, "immutable");

    registry.mock_auth_with_addresses_for_publish(
        wasm_name,
        author,
        &Some(version.clone()),
        &immutable_bytes(env),
        &[author],
    );
    client.publish(wasm_name, author, &immutable_bytes(env), &version);

    let hash = immutable_hash(env);

    // A custom (account) deployer authorizes the deploy in its own namespace.
    env.mock_all_auths();
    let address = client.deploy_stateless(wasm_name, &None, &Some(deployer.clone()));

    let expected = env
        .deployer()
        .with_address(deployer.clone(), hash)
        .deployed_address();
    assert_eq!(address, expected);

    // Idempotent for the same deployer.
    let again = client.deploy_stateless(wasm_name, &None, &Some(deployer.clone()));
    assert_eq!(again, address);
}
