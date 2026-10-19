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
        account_address: &Address,
    ) -> Result<(), Error> {
        Self::authorize_account_naming(env, account_admin, account_name, account_address);
        let is_available = !Storage::new(env).account.has(account_name);
        is_available
            .then_some(())
            .ok_or(Error::AccountNameAlreadyTaken)
    }

    /// Requires the owner's auth (or the manager's, if set) plus the named
    /// address's consent: either it signs this call, or it signed
    /// `preauthorize_account` earlier (needed when a manager executes the call
    /// and can't collect its signature). A matching preauthorization is always
    /// consumed, so none outlives the naming. Returns the operator.
    fn authorize_account_naming(
        env: &Env,
        owner: &Address,
        account_name: &NormalizedName,
        address: &Address,
    ) -> Address {
        let operator = Self::require_owner_or_manager(env, owner);
        let preauths = Storage::new(env).preauth_accounts;
        if preauths.get(account_name).as_ref() == Some(address) {
            preauths.remove(account_name);
        } else if *address != operator {
            // The host rejects a second `require_auth` for the operator.
            address.require_auth();
        }
        operator
    }

    /// Accounts may be classic G-addresses or smart accounts (C-addresses
    /// backed by a Wasm contract). Stellar Asset Contracts are not accounts.
    fn assert_valid_account_address(address: &Address) -> Result<(), Error> {
        match address.executable() {
            Some(Executable::Account | Executable::Wasm(_)) => Ok(()),
            Some(Executable::StellarAsset) => Err(Error::NotAccountAddress),
            None => Err(Error::ContractIdAddressDoesNotExist),
        }
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
        Self::assert_valid_account_address(account_address)?;
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
            owner: account_admin.clone(),
        }
        .publish(env);
        Ok(())
    }
}

#[contracttrait]
pub trait AccountRegistrable {
    /// Register a name for an existing G-address (Stellar account) or
    /// C-address (smart account). Requires `account_address`'s auth, or its
    /// unexpired `preauthorize_account` for this name.
    fn register_account(
        env: &Env,
        account_name: soroban_sdk::String,
        account_address: soroban_sdk::Address,
        owner: soroban_sdk::Address,
    ) -> Result<(), Error> {
        let account_name = account_name.try_into()?;
        Contract::assert_no_account_entry_and_authorize(
            env,
            &owner,
            &account_name,
            &account_address,
        )?;
        Contract::register_account_name(env, &account_name, &account_address, &owner)?;
        Ok(())
    }

    /// Let `account_address` be named `account_name` by a later
    /// `register_account` or `update_account_address` that it doesn't sign,
    /// e.g. one executed by a manager. Expires after the network's max
    /// temporary-entry TTL. Replaces any pending preauthorization for this
    /// name.
    fn preauthorize_account(
        env: &Env,
        account_name: soroban_sdk::String,
        account_address: soroban_sdk::Address,
    ) -> Result<(), Error> {
        let account_name: NormalizedName = account_name.try_into()?;
        account_address.require_auth();
        let preauths = Storage::new(env).preauth_accounts;
        preauths.set(&account_name, &account_address);
        // Temporary entries cannot outlive the network's max_entry_ttl; unlike
        // persistent entries the host traps instead of clamping.
        let max_ttl = env.storage().max_ttl();
        preauths.extend_ttl(&account_name, max_ttl, max_ttl);
        Ok(())
    }

    /// Look up the address of a registered account name
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

    /// Update the address of a registered account name.
    /// Requires current owner auth, or manager auth if manager is set, plus
    /// `new_address`'s consent as in `register_account`.
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

        let operator =
            Contract::authorize_account_naming(env, &entry.owner, &account_name, &new_address);

        Contract::assert_valid_account_address(&new_address)?;

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
