extern crate std;

use soroban_sdk::{self, xdr, Address, Bytes, BytesN, Env, TryFromVal};
use std::rc::Rc;

/// Writes a classic Stellar account ledger entry directly (mirroring the
/// approach `Env::register_stellar_asset_contract_v2` uses internally to
/// fund its issuer account) and returns the corresponding G-address.
///
/// `Address::generate` (the usual testutils helper) always produces a
/// *contract* address, never an account one — there's no public testutils
/// helper for a funded G-address, so this constructs the ledger entry
/// directly. `seed` just needs to be distinct per call within a test to get
/// distinct addresses.
pub fn fund_account(env: &Env, seed: u8) -> Address {
    let account_id = xdr::AccountId(xdr::PublicKey::PublicKeyTypeEd25519(xdr::Uint256(
        [seed; 32],
    )));
    let key = Rc::new(xdr::LedgerKey::Account(xdr::LedgerKeyAccount {
        account_id: account_id.clone(),
    }));
    if env.host().get_ledger_entry(&key).unwrap().is_none() {
        let entry = Rc::new(xdr::LedgerEntry {
            data: xdr::LedgerEntryData::Account(xdr::AccountEntry {
                account_id: account_id.clone(),
                balance: 0,
                flags: 0,
                home_domain: xdr::String32::default(),
                inflation_dest: None,
                num_sub_entries: 0,
                seq_num: xdr::SequenceNumber(0),
                thresholds: xdr::Thresholds([1; 4]),
                signers: xdr::VecM::default(),
                ext: xdr::AccountEntryExt::V0,
            }),
            last_modified_ledger_seq: 0,
            ext: xdr::LedgerEntryExt::V0,
        });
        env.host().add_ledger_entry(&key, &entry, None).unwrap();
    }
    Address::try_from_val(env, &xdr::ScAddress::Account(account_id)).unwrap()
}

pub mod hello_world {
    soroban_sdk::contractimport!(file = "../../target/stellar/local/hello_world.wasm");
}

pub mod hello_world_v2 {
    soroban_sdk::contractimport!(file = "../../target/stellar/local/hello_world_v2.wasm");
}

pub mod hello_world_v3 {
    soroban_sdk::contractimport!(file = "../../target/stellar/local/hello_world_v3.wasm");
}

pub fn hw_hash(env: &Env) -> BytesN<32> {
    env.deployer().upload_contract_wasm(hello_world::WASM)
}
pub fn hw_client<'a>(env: &Env, address: &Address) -> hello_world::Client<'a> {
    hello_world::Client::new(env, address)
}
pub fn hw_bytes(env: &Env) -> Bytes {
    Bytes::from_slice(env, hello_world::WASM)
}

pub fn hw_hash_v2(env: &Env) -> BytesN<32> {
    env.deployer().upload_contract_wasm(hello_world_v2::WASM)
}
pub fn hw_client_v2<'a>(env: &Env, address: &Address) -> hello_world_v2::Client<'a> {
    hello_world_v2::Client::new(env, address)
}
pub fn hw_bytes_v2(env: &Env) -> Bytes {
    Bytes::from_slice(env, hello_world_v2::WASM)
}

pub fn hw_hash_v3(env: &Env) -> BytesN<32> {
    env.deployer().upload_contract_wasm(hello_world_v3::WASM)
}
pub fn hw_client_v3<'a>(env: &Env, address: &Address) -> hello_world_v3::Client<'a> {
    hello_world_v3::Client::new(env, address)
}
pub fn hw_bytes_v3(env: &Env) -> Bytes {
    Bytes::from_slice(env, hello_world_v3::WASM)
}
