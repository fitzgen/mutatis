//! `Mutate` implementations for `std` types.

use super::*;
use std::hash::Hash;

mod hash_map;
mod hash_set;

pub use hash_map::*;
pub use hash_set::*;
