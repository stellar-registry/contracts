#![no_std]
//! Shared types extracted from the on-chain Stellar Registry contract so that
//! other Soroban contracts can reuse them without depending on the registry
//! contract crate itself (which is a `cdylib`, `publish = false`).
//!
//! Currently this exposes [`NormalizedName`] — the registry's crate-name
//! validation/normalization type — and its [`NameError`].

mod error;
mod name;

pub use error::NameError;
pub use name::NormalizedName;
