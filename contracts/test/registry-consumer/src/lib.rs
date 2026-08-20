#![no_std]
// `deploy`/`deploy_with_subregistry` take 8 args by design.
#![allow(clippy::too_many_arguments)]
//! Compile-only smoke test proving `registry-traits`' `#[contracttrait]`s are
//! *importable* — a downstream contract, in a separate crate depending only on
//! `registry-traits` (exactly as perch will), can wire the whole registry
//! behavior set onto its own contract type with empty `impl` blocks. If the
//! trait default bodies still hard-referenced a concrete `Contract`, or if the
//! crate dragged in admin-sep / soroban-sdk-tools, this would not compile.
//!
//! There is no state or storage set-up here: the point is that the traits
//! *resolve and type-check* against a foreign `Self`.

// The registry trait signatures return `Result<_, registry_traits::Error>`; the
// `#[contracttrait]` glue expects that `Error` type in scope on the impl side.
use registry_traits::registry::contract::{
    Batchable, Deployable, Manageable, Proxyable, Redeployable, StatelessDeployable,
};
use registry_traits::registry::wasm::Publishable;
use registry_traits::Error;
use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct Consumer;

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
