use crate::Error;

pub(crate) trait AsStr {
    fn as_str(&self) -> Result<&str, Error>;
}

impl AsStr for [u8] {
    fn as_str(&self) -> Result<&str, Error> {
        core::str::from_utf8(self).map_err(|_| Error::InvalidName)
    }
}
