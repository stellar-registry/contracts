use crate::test::contracts::{hw_bytes, hw_bytes_v2, hw_bytes_v3, hw_hash_v2};
use crate::{
    error::Error,
    test::registry::{to_string, Registry},
};
use soroban_sdk::{self, testutils::Address as _, Address, String};

/// Convenience: three ascending versions.
fn versions(env: &soroban_sdk::Env) -> (String, String, String) {
    (
        to_string(env, "0.0.0"),
        to_string(env, "0.0.1"),
        to_string(env, "0.0.2"),
    )
}

#[test]
fn preauthorized_author_can_publish_next_version() {
    let registry = &Registry::new_unverified();
    let env = registry.env();
    let client = registry.client();
    let name = &to_string(env, "widget");
    let (v0, v1, _) = versions(env);

    let alice = &Address::generate(env);
    let bob = &Address::generate(env);

    // Alice claims the name.
    registry.mock_auth_with_addresses_for_publish(
        name,
        alice,
        &Some(v0.clone()),
        &hw_bytes(env),
        &[alice],
    );
    client.publish(name, alice, &hw_bytes(env), &v0);

    // Alice pre-authorizes Bob.
    registry.mock_auths_for(&[alice], "preauthorize_author_transfer", (name, bob));
    client.preauthorize_author_transfer(name, bob);

    // Bob publishes the next version signing only for himself — succeeds.
    registry.mock_auth_with_addresses_for_publish(
        name,
        bob,
        &Some(v1.clone()),
        &hw_bytes_v2(env),
        &[bob],
    );
    assert!(
        client
            .try_publish(name, bob, &hw_bytes_v2(env), &v1)
            .is_ok(),
        "pre-authorized author must be able to publish the next version"
    );
    assert_eq!(client.fetch_hash(name, &None), hw_hash_v2(env));
}

#[test]
fn transfer_makes_new_author_the_owner() {
    let registry = &Registry::new_unverified();
    let env = registry.env();
    let client = registry.client();
    let name = &to_string(env, "widget");
    let (v0, v1, _) = versions(env);

    let alice = &Address::generate(env);
    let bob = &Address::generate(env);
    let carol = &Address::generate(env);

    registry.mock_auth_with_addresses_for_publish(
        name,
        alice,
        &Some(v0.clone()),
        &hw_bytes(env),
        &[alice],
    );
    client.publish(name, alice, &hw_bytes(env), &v0);

    registry.mock_auths_for(&[alice], "preauthorize_author_transfer", (name, bob));
    client.preauthorize_author_transfer(name, bob);

    registry.mock_auth_with_addresses_for_publish(
        name,
        bob,
        &Some(v1.clone()),
        &hw_bytes_v2(env),
        &[bob],
    );
    client.publish(name, bob, &hw_bytes_v2(env), &v1);

    // Bob is now the author: only the author may pre-authorize, so Bob signing
    // for himself must succeed.
    registry.mock_auths_for(&[bob], "preauthorize_author_transfer", (name, carol));
    assert!(
        client.try_preauthorize_author_transfer(name, carol).is_ok(),
        "after transfer the new author must be able to pre-authorize"
    );

    // ...and Alice, the former author, must no longer be able to.
    registry.mock_auths_for(&[alice], "preauthorize_author_transfer", (name, alice));
    assert!(
        client
            .try_preauthorize_author_transfer(name, alice)
            .is_err(),
        "former author must lose the right to pre-authorize after transfer"
    );
}

#[test]
fn former_author_cannot_publish_after_transfer() {
    let registry = &Registry::new_unverified();
    let env = registry.env();
    let client = registry.client();
    let name = &to_string(env, "widget");
    let (v0, v1, v2) = versions(env);

    let alice = &Address::generate(env);
    let bob = &Address::generate(env);

    registry.mock_auth_with_addresses_for_publish(
        name,
        alice,
        &Some(v0.clone()),
        &hw_bytes(env),
        &[alice],
    );
    client.publish(name, alice, &hw_bytes(env), &v0);

    registry.mock_auths_for(&[alice], "preauthorize_author_transfer", (name, bob));
    client.preauthorize_author_transfer(name, bob);

    registry.mock_auth_with_addresses_for_publish(
        name,
        bob,
        &Some(v1.clone()),
        &hw_bytes_v2(env),
        &[bob],
    );
    client.publish(name, bob, &hw_bytes_v2(env), &v1);

    // Alice, no longer the author and with no outstanding pre-authorization for
    // herself, cannot publish again with only her own signature.
    registry.mock_auth_with_addresses_for_publish(
        name,
        alice,
        &Some(v2.clone()),
        &hw_bytes_v3(env),
        &[alice],
    );
    assert!(
        client
            .try_publish(name, alice, &hw_bytes_v3(env), &v2)
            .is_err(),
        "former author must not be able to publish after transferring authorship"
    );
}

#[test]
fn non_preauthorized_address_cannot_take_over() {
    let registry = &Registry::new_unverified();
    let env = registry.env();
    let client = registry.client();
    let name = &to_string(env, "widget");
    let (v0, v1, _) = versions(env);

    let alice = &Address::generate(env);
    let bob = &Address::generate(env);
    let carol = &Address::generate(env);

    registry.mock_auth_with_addresses_for_publish(
        name,
        alice,
        &Some(v0.clone()),
        &hw_bytes(env),
        &[alice],
    );
    client.publish(name, alice, &hw_bytes(env), &v0);

    // Alice pre-authorizes Bob — Carol is a stranger to this transfer.
    registry.mock_auths_for(&[alice], "preauthorize_author_transfer", (name, bob));
    client.preauthorize_author_transfer(name, bob);

    // Carol tries to publish signing only for herself — must be rejected even
    // though a (different) pre-authorization is outstanding.
    registry.mock_auth_with_addresses_for_publish(
        name,
        carol,
        &Some(v1.clone()),
        &hw_bytes_v2(env),
        &[carol],
    );
    assert!(
        client
            .try_publish(name, carol, &hw_bytes_v2(env), &v1)
            .is_err(),
        "a non-pre-authorized address must not be able to take over authorship"
    );

    // Alice is still the author: she can still pre-authorize.
    registry.mock_auths_for(&[alice], "preauthorize_author_transfer", (name, bob));
    assert!(
        client.try_preauthorize_author_transfer(name, bob).is_ok(),
        "original author must remain the author after a failed takeover"
    );
}

#[test]
fn stranger_cannot_take_over_without_any_preauthorization() {
    let registry = &Registry::new_unverified();
    let env = registry.env();
    let client = registry.client();
    let name = &to_string(env, "widget");
    let (v0, v1, _) = versions(env);

    let alice = &Address::generate(env);
    let mallory = &Address::generate(env);

    registry.mock_auth_with_addresses_for_publish(
        name,
        alice,
        &Some(v0.clone()),
        &hw_bytes(env),
        &[alice],
    );
    client.publish(name, alice, &hw_bytes(env), &v0);

    // No pre-authorization exists; Mallory signing for herself must fail.
    registry.mock_auth_with_addresses_for_publish(
        name,
        mallory,
        &Some(v1.clone()),
        &hw_bytes_v2(env),
        &[mallory],
    );
    assert!(
        client
            .try_publish(name, mallory, &hw_bytes_v2(env), &v1)
            .is_err(),
        "without a pre-authorization no other address may publish"
    );
}

#[test]
fn preauthorize_requires_current_author_auth() {
    let registry = &Registry::new_unverified();
    let env = registry.env();
    let client = registry.client();
    let name = &to_string(env, "widget");
    let (v0, _, _) = versions(env);

    let alice = &Address::generate(env);
    let mallory = &Address::generate(env);

    registry.mock_auth_with_addresses_for_publish(
        name,
        alice,
        &Some(v0.clone()),
        &hw_bytes(env),
        &[alice],
    );
    client.publish(name, alice, &hw_bytes(env), &v0);

    // Mallory tries to pre-authorize herself, signing only for herself.
    registry.mock_auths_for(&[mallory], "preauthorize_author_transfer", (name, mallory));
    assert!(
        client
            .try_preauthorize_author_transfer(name, mallory)
            .is_err(),
        "only the current author may pre-authorize a transfer"
    );
}

#[test]
fn preauthorize_unpublished_wasm_fails() {
    let registry = &Registry::new_unverified();
    let env = registry.env();
    let client = registry.client();
    let name = &to_string(env, "never_published");

    let alice = &Address::generate(env);
    let bob = &Address::generate(env);

    registry.mock_auths_for(&[alice], "preauthorize_author_transfer", (name, bob));
    assert_eq!(
        client
            .try_preauthorize_author_transfer(name, bob)
            .unwrap_err(),
        Ok(Error::NoSuchWasmPublished),
        "cannot pre-authorize a transfer for a name that was never published"
    );
}

#[test]
fn reauthorization_overwrites_previous_target() {
    let registry = &Registry::new_unverified();
    let env = registry.env();
    let client = registry.client();
    let name = &to_string(env, "widget");
    let (v0, v1, _) = versions(env);

    let alice = &Address::generate(env);
    let bob = &Address::generate(env);
    let carol = &Address::generate(env);

    registry.mock_auth_with_addresses_for_publish(
        name,
        alice,
        &Some(v0.clone()),
        &hw_bytes(env),
        &[alice],
    );
    client.publish(name, alice, &hw_bytes(env), &v0);

    // Alice pre-authorizes Bob, then changes her mind and pre-authorizes Carol.
    registry.mock_auths_for(&[alice], "preauthorize_author_transfer", (name, bob));
    client.preauthorize_author_transfer(name, bob);
    registry.mock_auths_for(&[alice], "preauthorize_author_transfer", (name, carol));
    client.preauthorize_author_transfer(name, carol);

    // Bob is no longer authorized and cannot publish.
    registry.mock_auth_with_addresses_for_publish(
        name,
        bob,
        &Some(v1.clone()),
        &hw_bytes_v2(env),
        &[bob],
    );
    assert!(
        client
            .try_publish(name, bob, &hw_bytes_v2(env), &v1)
            .is_err(),
        "re-authorizing to Carol must revoke Bob's pending authorization"
    );

    // Carol, the current target, can take over.
    registry.mock_auth_with_addresses_for_publish(
        name,
        carol,
        &Some(v1.clone()),
        &hw_bytes_v2(env),
        &[carol],
    );
    assert!(
        client
            .try_publish(name, carol, &hw_bytes_v2(env), &v1)
            .is_ok(),
        "the most recently pre-authorized address must be able to take over"
    );
}
