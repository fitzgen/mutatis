//! `Mutate` implementations for `alloc` types.

use super::*;

mod binary_heap;
mod btree_map;
mod btree_set;
mod linked_list;
mod vec;
mod vec_deque;

pub use binary_heap::*;
pub use btree_map::*;
pub use btree_set::*;
pub use linked_list::*;
pub use vec::*;
pub use vec_deque::*;
