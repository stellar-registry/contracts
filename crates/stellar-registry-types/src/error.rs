use soroban_sdk::contracterror;

/// Errors produced while validating/normalizing a [`crate::NormalizedName`].
///
/// Kept as a small, self-contained error (built on `soroban_sdk`'s native
/// `contracterror` so it stays portable across SDK versions) so consumers — and
/// the registry contract — can map it onto their own richer error enums via
/// `From`.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum NameError {
    /// Invalid name.
    /// Must be at most 64 characters and non-empty;
    /// ascii alphanumeric, '-', or '_';
    /// start with a ascii alphabetic character;
    /// and not be a Rust keyword
    InvalidName = 1,
}
