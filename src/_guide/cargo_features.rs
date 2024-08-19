/*!

# Cargo Features

**Note: none of this crate's features are enabled by default. You most likely
want to enable `std`.**

* **`alloc`**: Enable mutators for types in Rust's `alloc` crate and internally
  use features that the `alloc` crate provides.

* **`std`**: Enable mutators for types in Rust's `std` crate and internally use
  features that the `std` crate provides.

* **`log`**: Enable logging with [the `log` crate](https://docs.rs/log).

* **`check`**: Enable the `mutatis::check` module for writing property-based
  smoke tests with `mutatis`.

* **`derive`**: Enable the `#[derive(Mutate)]` macro for automatically deriving
  mutators for your types

 */
