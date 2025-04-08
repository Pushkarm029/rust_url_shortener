use core::cmp::Ordering;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::errors::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnixNanos(pub u128);

impl FromStr for UnixNanos {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(UnixNanos(
            u128::from_str(s).map_err(|e| AppError::InvalidInput(e.to_string()))?,
        ))
    }
}

impl fmt::Display for UnixNanos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u128> for UnixNanos {
    fn from(nanos: u128) -> Self {
        UnixNanos(nanos)
    }
}

impl From<UnixNanos> for u128 {
    fn from(nanos: UnixNanos) -> Self {
        nanos.0
    }
}

impl PartialOrd for UnixNanos {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UnixNanos {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}
