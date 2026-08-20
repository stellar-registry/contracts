#![allow(
    clippy::ref_option,
    clippy::needless_pass_by_value,
    clippy::too_many_arguments
)]
extern crate std;

mod contracts;
mod registry;

mod batch;
mod manageable;
mod names;
mod stateless;
mod unverified;
mod verified;
