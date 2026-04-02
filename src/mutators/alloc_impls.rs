//! `Mutate` implementations for `alloc` types.

use super::*;

mod binary_heap;
mod boxed;
mod btree_map;
mod btree_set;
mod c_string;
mod cow;
mod linked_list;
mod string;
mod vec;
mod vec_deque;

pub use binary_heap::*;
pub use boxed::*;
pub use btree_map::*;
pub use btree_set::*;
pub use c_string::*;
pub use cow::*;
pub use linked_list::*;
pub use string::*;
pub use vec::*;
pub use vec_deque::*;
