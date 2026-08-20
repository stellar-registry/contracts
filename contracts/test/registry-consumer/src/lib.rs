#![no_std]
// `deploy`/`deploy_with_subregistry` take 8 args by design; the registry crate
// allows this crate-wide too.
#![allow(clippy::too_many_arguments)]
//! Compile-only smoke test proving the registry's `#[contracttrait]`s are
//! *importable* — a downstream contract, defined in a completely separate
//! crate, can wire the whole registry behavior set onto its own contract type
//! with empty `impl` blocks. If the trait default bodies still hard-referenced
//! `crate::Contract`, this crate would fail to compile.
//!
//! There is no state or storage set-up here: the point is that the traits
//! *resolve and type-check* against a foreign `Self`. Deploying/running this
//! contract is out of scope (the registry's own test suite covers behavior).

use admin_sep::{Administratable, Upgradable};
// The registry trait signatures return `Result<_, registry::Error>`; the
// `#[contracttrait]` glue expects that `Error` type in scope on the impl side.
use registry::registry::contract::{
    Batchable, Deployable, Manageable, Proxyable, Redeployable, StatelessDeployable,
};
use registry::registry::wasm::Publishable;
use registry::Error;
use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct Consumer;

// `Administratable`/`Upgradable` come from admin-sep; the registry traits below
// reuse `AdministratableExtension::require_admin` for their auth, so a consumer
// wires an admin backing too.
#[contractimpl(contracttrait)]
impl Administratable for Consumer {}

#[contractimpl(contracttrait)]
impl Upgradable for Consumer {}

// The registry's own behavior set, implemented on a *foreign* contract type
// purely from the imported trait defaults.
#[contractimpl(contracttrait)]
impl Deployable for Consumer {}

#[contractimpl(contracttrait)]
impl StatelessDeployable for Consumer {}

#[contractimpl(contracttrait)]
impl Redeployable for Consumer {}

#[contractimpl(contracttrait)]
impl Batchable for Consumer {}

#[contractimpl(contracttrait)]
impl Manageable for Consumer {}

#[contractimpl(contracttrait)]
impl Proxyable for Consumer {}

#[contractimpl(contracttrait)]
impl Publishable for Consumer {}
