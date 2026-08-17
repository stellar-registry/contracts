use flux_rs::attrs::{invariant, opts, refined_by};
use soroban_sdk::{Env, String};

use super::to_str::AsStr;
use crate::Error;

/// Longest accepted name, in bytes. Module-level (not an associated const)
/// so it can appear in the flux refinement below.
pub(crate) const MAX_NAME_LENGTH: usize = 64;

/// Flux-refined: `len` is an index of the type with the invariant
/// `len <= MAX_NAME_LENGTH`, so every slice of `internal` by `len` is proven
/// in-bounds at compile time rather than by inspection.
#[refined_by(n: int)]
#[invariant(n <= MAX_NAME_LENGTH)]
pub(crate) struct Normalized {
    #[field(usize[n])]
    len: usize,
    internal: [u8; MAX_NAME_LENGTH],
}

impl Normalized {
    pub const MAX_NAME_LENGTH: usize = MAX_NAME_LENGTH;

    pub fn canonicalize(s: &String) -> Result<String, Error> {
        Normalized::new(s)?.to_string(s.env())
    }

    #[opts(check_overflow = "strict")]
    pub fn new(s: &String) -> Result<Self, Error> {
        let len = s.len() as usize;
        // `len == 0` is exactly `s.is_empty()`; phrased on the local so the
        // non-empty fact is visible to flux too.
        if len > MAX_NAME_LENGTH || len == 0 {
            return Err(Error::InvalidName);
        }
        let mut internal = [0u8; MAX_NAME_LENGTH];
        let (first, _) = internal.split_at_mut(len);
        s.copy_into_slice(first);
        Self { len, internal }.normalize()?.validate()
    }

    /// Validates the crates.io-style charset and canonicalizes in place
    /// (`_` → `-`, uppercase → lowercase).
    ///
    /// Operates byte-wise rather than on `chars()`: every accepted byte is
    /// ASCII (anything `>= 0x80` — i.e. any byte of a multi-byte UTF-8
    /// character — fails `is_ascii_alphanumeric` exactly like its `char`
    /// counterpart did), so the byte walk accepts and rewrites precisely the
    /// same strings, while dropping the old `unsafe as_bytes_mut` and the
    /// deferred `chars_to_change` buffer, and giving flux an index loop it
    /// can bound.
    #[opts(check_overflow = "strict")]
    fn normalize(mut self) -> Result<Self, Error> {
        // Index `internal` directly (not through the `as_mut_bytes` slice):
        // the struct invariant `len <= MAX_NAME_LENGTH` is what proves every
        // access below in-bounds, and it lives on the struct fields.
        if self.len == 0 || !self.internal[0].is_ascii_alphabetic() {
            return Err(Error::InvalidName);
        }
        let mut i: usize = 0;
        while i < self.len {
            let b = self.internal[i];
            if !(b.is_ascii_alphanumeric() || b == b'_' || b == b'-') {
                return Err(Error::InvalidName);
            }
            if b == b'_' {
                self.internal[i] = b'-';
            } else {
                self.internal[i] = b.to_ascii_lowercase();
            }
            i += 1;
        }
        Ok(self)
    }

    fn validate(self) -> Result<Self, Error> {
        if is_keyword(self.as_str()?) {
            return Err(Error::InvalidName);
        }
        Ok(self)
    }

    fn as_bytes(&self) -> &[u8] {
        let (first, _) = self.internal.split_at(self.len);
        first
    }

    pub fn to_string(&self, env: &Env) -> Result<String, Error> {
        let s = self.as_str()?;
        Ok(String::from_str(env, s))
    }
}

impl AsStr for Normalized {
    fn as_str(&self) -> Result<&str, Error> {
        self.as_bytes().as_str()
    }
}

/// from crate `check_keyword`
/// <https://github.com/JoelCourtney/check_keyword/blob/68486cbfa368070fdbfd383fc5840aa380bb1e6f/src/lib.rs#L120>
fn is_keyword(s: &str) -> bool {
    match s {
    "as" |
    "break" |
    "const" |
    "continue" |
    "crate" |
    "else" |
    "enum" |
    "extern" |
    "false" |
    "fn" |
    "for" |
    "if" |
    "impl" |
    "in" |
    "let" |
    "loop" |
    "match" |
    "mod" |
    "move" |
    "mut" |
    "pub" |
    "ref" |
    "return" |
    "self" |
    "Self" |
    "static" |
    "struct" |
    "super" |
    "trait" |
    "true" |
    "type" |
    "unsafe" |
    "use" |
    "where" |
    "while" |

    // STRICT, 2018

    "async"|
    "await"|

    // DYN

    "dyn" |

    // RESERVED, 2015

    "abstract" |
    "become" |
    "box" |
    "do" |
    "final" |
    "macro" |
    "override" |
    "priv" |
    "typeof" |
    "unsized" |
    "virtual" |
    "yield" |

    // RESERVED, 2018

    "try" |

    // RESERVED, 2024
    "gen" |

    // WEAK

    "macro_rules" |
    "union" |
    "'static" |

    // Windows keywords
    "nul" => true,
    _ => false
    }
}
