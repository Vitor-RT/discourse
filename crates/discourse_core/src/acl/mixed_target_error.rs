use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct MixedTargetError;

impl fmt::Display for MixedTargetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mixed Target Error")
    }
}

impl Error for MixedTargetError {}
