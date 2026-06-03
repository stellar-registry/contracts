use crate::error::NameError;

pub(crate) trait AsStr {
    fn as_mut_str(&mut self) -> Result<&mut str, NameError>;
    fn as_str(&self) -> Result<&str, NameError>;
}

impl AsStr for [u8] {
    fn as_mut_str(&mut self) -> Result<&mut str, NameError> {
        core::str::from_utf8_mut(self).map_err(|_| NameError::InvalidName)
    }

    fn as_str(&self) -> Result<&str, NameError> {
        core::str::from_utf8(self).map_err(|_| NameError::InvalidName)
    }
}
