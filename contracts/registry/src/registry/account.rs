use crate::name::NormalizedName;
use crate::storage::AccountEntry;
use crate::storage::Storage;

use soroban_sdk::contracttrait;
use soroban_sdk::Executable;
use soroban_sdk::{self, Address, Env};

use crate::{error::Error, Contract};

impl Contract {
    pub(crate) fn assert_no_account_entry_and_authorize(
        env: &Env,
        account_admin: &Address,
        account_name: &NormalizedName,
    ) -> Result<(), Error> {
        if let Some(manager) = Storage::manager(env) {
            manager.require_auth();
        } else {
            account_admin.require_auth();
        }
        let is_available = !Storage::new(env).account.has(account_name);
        is_available
            .then_some(())
            .ok_or(Error::AccountNameAlreadyTaken)
    }

    fn get_account_entry(env: &Env, account_name: &NormalizedName) -> Result<AccountEntry, Error> {
        Storage::new(env)
            .account
            .get(account_name)
            .ok_or(Error::NoSuchAccountRegistered)
    }

    pub(crate) fn get_account_id(
        env: &Env,
        account_name: &NormalizedName,
    ) -> Result<Address, Error> {
        Ok(Self::get_account_entry(env, account_name)?.address)
    }

    pub(crate) fn get_account_owner(
        env: &Env,
        account_name: &NormalizedName,
    ) -> Result<Address, Error> {
        Ok(Self::get_account_entry(env, account_name)?.owner)
    }

    pub(crate) fn register_account_name(
        env: &Env,
        account_name: &NormalizedName,
        account_address: &Address,
        account_admin: &Address,
    ) -> Result<(), Error> {
        match account_address.executable() {
            Some(Executable::Account) => {}
            Some(_) => return Err(Error::NotAccountAddress),
            None => return Err(Error::ContractIdAddressDoesNotExist),
        }
        let mut account_map = Storage::new(env).account;
        account_map.set(
            account_name,
            &AccountEntry {
                owner: account_admin.clone(),
                address: account_address.clone(),
                flagged: false,
            },
        );
        crate::events::RegisterAccount {
            account_name: account_name.to_string(),
            account_id: account_address.clone(),
        }
        .publish(env);
        Ok(())
    }
}

#[contracttrait]
pub trait AccountRegistrable {
    /// Register a name for an existing G-address (Stellar account).
    fn register_account(
        env: &Env,
        account_name: soroban_sdk::String,
        account_address: soroban_sdk::Address,
        owner: soroban_sdk::Address,
    ) -> Result<(), Error> {
        let account_name = account_name.try_into()?;
        Contract::assert_no_account_entry_and_authorize(env, &owner, &account_name)?;
        Contract::register_account_name(env, &account_name, &account_address, &owner)?;
        Ok(())
    }

    /// Look up the G-address of a registered account name
    fn fetch_account_id(
        env: &Env,
        account_name: soroban_sdk::String,
    ) -> Result<soroban_sdk::Address, Error> {
        Contract::get_account_id(env, &account_name.try_into()?)
    }

    /// Look up the owner of a registered account name
    fn fetch_account_owner(
        env: &Env,
        account_name: soroban_sdk::String,
    ) -> Result<soroban_sdk::Address, Error> {
        Contract::get_account_owner(env, &account_name.try_into()?)
    }
}
