#![no_std]
use soroban_sdk::{Env, String, contract, contractimpl};

/// A minimal fixture for `StatelessDeployable`: its `__constructor` takes no
/// arguments and does not panic, so it can be content-addressed-deployed with
/// `init = ()`.
#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn __constructor(_env: &Env) {}

    pub fn hello(env: &Env) -> String {
        String::from_str(env, "stateless!")
    }
}
