use soroban_sdk::{Env, String};

// `NormalizedName` (crate-name validation/normalization) now lives in the
// shared `stellar-registry-types` crate so other Soroban contracts can reuse it
// without depending on this `cdylib` contract crate. Re-exported here so the
// rest of the registry keeps using `crate::name::NormalizedName`.
pub use stellar_registry_types::NormalizedName;

pub(crate) const REGISTRY: &str = "registry";
pub(crate) const UNVERIFIED: &str = "unverified";
pub(crate) const ROOT: &str = "root";

#[must_use]
pub fn registry(env: &Env) -> NormalizedName {
    unsafe { NormalizedName::new_unchecked(String::from_str(env, REGISTRY)) }
}

#[must_use]
pub fn unverifed(env: &Env) -> NormalizedName {
    unsafe { NormalizedName::new_unchecked(String::from_str(env, UNVERIFIED)) }
}
