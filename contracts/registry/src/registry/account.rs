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

#[contracttrait]
pub trait AccountManageable {
    /// Update the owner of a registered account name.
    /// Requires current owner auth, or manager auth if manager is set.
    fn update_account_owner(
        env: &Env,
        account_name: soroban_sdk::String,
        new_owner: soroban_sdk::Address,
    ) -> Result<(), Error> {
        let account_name: NormalizedName = account_name.try_into()?;
        let mut storage = Storage::new(env);
        let entry = storage
            .account
            .get(&account_name)
            .ok_or(Error::NoSuchAccountRegistered)?;

        let operator = Contract::require_owner_or_manager(env, &entry.owner);

        storage.account.extend_ttl_max(&account_name);
        storage.account.set(
            &account_name,
            &AccountEntry {
                owner: new_owner.clone(),
                address: entry.address,
                flagged: entry.flagged,
            },
        );
        crate::events::UpdateAccountOwner {
            account_name: account_name.to_string(),
            new_owner,
            operator,
        }
        .publish(env);
        Ok(())
    }

    /// Update the G-address of a registered account name.
    /// Requires current owner auth, or manager auth if manager is set.
    fn update_account_address(
        env: &Env,
        account_name: soroban_sdk::String,
        new_address: soroban_sdk::Address,
    ) -> Result<(), Error> {
        let account_name: NormalizedName = account_name.try_into()?;
        let mut storage = Storage::new(env);
        let entry = storage
            .account
            .get(&account_name)
            .ok_or(Error::NoSuchAccountRegistered)?;

        let operator = Contract::require_owner_or_manager(env, &entry.owner);

        match new_address.executable() {
            Some(Executable::Account) => {}
            Some(_) => return Err(Error::NotAccountAddress),
            None => return Err(Error::ContractIdAddressDoesNotExist),
        }

        storage.account.extend_ttl_max(&account_name);
        storage.account.set(
            &account_name,
            &AccountEntry {
                owner: entry.owner,
                address: new_address.clone(),
                flagged: entry.flagged,
            },
        );
        crate::events::UpdateAccountAddress {
            account_name: account_name.to_string(),
            new_address,
            operator,
        }
        .publish(env);
        Ok(())
    }

    /// Rename a registered account name.
    /// Requires current owner auth, or manager auth if manager is set.
    fn rename_account(
        env: &Env,
        old_name: soroban_sdk::String,
        new_name: soroban_sdk::String,
    ) -> Result<(), Error> {
        let old_name: NormalizedName = old_name.try_into()?;
        let new_name: NormalizedName = new_name.try_into()?;

        let mut storage = Storage::new(env);
        let entry = storage
            .account
            .get(&old_name)
            .ok_or(Error::NoSuchAccountRegistered)?;

        let operator = Contract::require_owner_or_manager(env, &entry.owner);

        if storage.account.has(&new_name) {
            return Err(Error::AccountNameAlreadyTaken);
        }

        storage.account.remove(&old_name);
        storage.account.set(&new_name, &entry);
        storage.account.extend_ttl_max(&new_name);

        crate::events::RenameAccount {
            old_name: old_name.to_string(),
            new_name: new_name.to_string(),
            operator,
        }
        .publish(env);
        Ok(())
    }

    /// Flag account, marking it as compromised or un-marking it as being
    /// compromised.
    fn flag_account(
        env: &Env,
        account_name: soroban_sdk::String,
        flagged: bool,
    ) -> Result<(), Error> {
        let account_name: NormalizedName = account_name.try_into()?;

        let mut storage = Storage::new(env);
        let entry = Contract::get_account_entry(env, &account_name)?;

        let operator = Contract::require_owner_or_manager(env, &entry.owner);

        storage.account.extend_ttl_max(&account_name);
        storage.account.set(
            &account_name,
            &AccountEntry {
                owner: entry.owner,
                address: entry.address,
                flagged,
            },
        );

        crate::events::SecurityFlagAccount {
            account_name: account_name.to_string(),
            flagged,
            operator,
        }
        .publish(env);
        Ok(())
    }
}
