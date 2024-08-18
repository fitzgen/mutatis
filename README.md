<div align="center">
  <h1><code>mutatis</code></h1>
  <p>
    <strong><code>mutatis</code> is a library for writing custom, structure-aware test-case mutators for fuzzers in Rust.</strong>
  </p>
  <p>
    <a href="https://crates.io/crates/mutatis"><img src="https://img.shields.io/crates/v/mutatis.svg" alt="crates.io"></a>
    <a href="https://docs.rs/mutatis"><img src="https://docs.rs/mutatis/badge.svg" alt="docs.rs"></a>
  </p>
  <h3>
    <a href="https://github.com/fitzgen/mutatis">Repository</a>
    <span> | </span>
    <a href="https://github.com/fitzgen/mutatis/blob/main/CONTRIBUTING.md">Contributing</a>
  </h3>
</div>

## About

Many popular fuzzers — including [`libfuzzer`][libfuzzer], [AFL], and more — are
*coverage-guided* and *mutation-based*.

[libfuzzer]: https://crates.io/crates/libfuzzer-sys
[AFL]: https://crates.io/crates/afl

*Coverage-guided* means that the fuzzer observes which code is dynamically
executed while running an input through the system under test. When creating new
inputs, it will try to make inputs that execute new code paths, maximizing the
amount of code that's been explored. If a new input triggers new code paths to
be executed, then it is added to the corpus. If a new input only exercises code
paths that have already been discovered, then it is thrown away.

*Mutation-based* means that, when creating a new input, the fuzzer modifies an
existing input from its corpus. The idea is that, if the existing input
triggered interesting behavior in the system under test, then a modification of
that input probably will as well, but might additionally trigger some new
behavior as well. Consider the scenario where we are fuzzing programming
language's parser: if some input made it deep inside the parser, rather than
bouncing off early due to an invalid token, then a new input derived from this
one is also likely to go deep into the parser. At least it is more likely to do
so than a new random string.

But what happens when we aren't fuzzing a language parser, or something else
that the fuzzer's built-in mutation strategies are pretty good at supporting?
When we have our own custom, structured input type? In this case, some fuzzers
will expose a hook for customizing the routine for mutating an existing input
from its corpus to create a new candidate input, for example the
[`libfuzzer::fuzz_mutator!`][fuzz-mutator] hook. And `mutatis` exists to
simplify writing these custom mutation hooks.

## Usage

There are two primary components to this library:

1. **[The `mutatis::Mutate`
   trait.](https://docs.rs/mutatis/latest/mutatis/trait.Mutate.html)** A trait
   that is implemented by types which can mutate other types. The
   `mutatis::Mutate::mutate` trait method takes a value and chooses one of many
   mutations to apply. You can think of a `mutatis::Mutate` implementation like
   a random streaming iterator that takes an item as input and modifies it into
   the next value, rather than generating and returning them.

2. **[The `mutatis::mutators`
   module.](https://docs.rs/mutatis/latest/mutatis/mutators/index.html)** This
   module, idiomatically imported via `use mutatis::mutators as m`, provides
   types and combinators for building custom mutators.

### Using Default Mutators

Here's a simple example of using `mutatis` and its default mutators to randomly
mutate a value:

```rust
# fn foo() -> mutatis::Result<()> {
let mut point = (42, 36);

let mut session = mutatis::Session::new();
for _ in 0..3 {
    session.mutate(&mut point)?;
    println!("mutated point is {point:?}");
}

// Example output:
//
//     mutated point is (-565504428, 36)
//     mutated point is (-565504428, 49968845)
//     mutated point is (-1854163941, 49968845)
# Ok(())
# }
# foo().unwrap()
```

### Using Mutator Combinators

Here's an example of using `mutatis` to define a custom mutator for a simple
data structure:

```rust
# fn foo() -> mutatis::Result<()> {
use mutatis::{mutators as m, Mutate, Session};

/// A silly monster type.
#[derive(Debug)]
pub struct Monster {
    pos: [i32; 2],
    hp: u16,

    // Invariant: ghost's are already dead, so when `is_ghost = true` it must
    // always be the case that `hp = 0`.
    is_ghost: bool,
}

/// A mutator that mutates one of a monster's fields, while maintaining our
/// invariant that ghosts always have zero HP.
let mut mutator =
    // Mutate the `pos` field...
    m::array(m::i32()).proj(|x: &mut Monster| &mut x.pos)
        // ...or mutate the `hp` field...
        .or(
            m::u16()
                .proj(|x: &mut Monster| &mut x.hp)
                .map(|_ctx, monster| {
                    // If we mutated the `hp` such that it is non-zero, then the
                    // monster cannot be a ghost.
                    if monster.hp > 0 {
                        monster.is_ghost = false;
                    }
                    Ok(())
                }),
        )
        // ...or mutate the `is_ghost` field.
        .or(
            m::bool()
                .proj(|x: &mut Monster| &mut x.is_ghost)
                .map(|_ctx, monster| {
                    // If we turned this monster into a ghost, then its `hp`
                    // must be zero.
                    if monster.is_ghost {
                        monster.hp = 0;
                    }
                    Ok(())
                }),
        );

// Define a monster...
let mut monster = Monster {
    hp: 36,
    is_ghost: false,
    pos: [-8, 9000],
};

// ...and mutate it a bunch of times!
let mut session = Session::new();
for _ in 0..5 {
    session.mutate_with(&mut mutator, &mut monster)?;
    println!("mutated monster is {monster:?}");
}

// Example output:
//
//     mutated monster is Monster { pos: [-8, -1647191276], hp: 36, is_ghost: false }
//     mutated monster is Monster { pos: [-8, -1062708247], hp: 36, is_ghost: false }
//     mutated monster is Monster { pos: [-8, -1062708247], hp: 61401, is_ghost: false }
//     mutated monster is Monster { pos: [-8, -1062708247], hp: 0, is_ghost: true }
//     mutated monster is Monster { pos: [-8, 1487274938], hp: 0, is_ghost: true }
# Ok(())
# }
# foo().unwrap()
```

### Automatically Deriving Mutators with `#[derive(Mutate)]`

If you enable this crate's `derive` feature, then you can automatically derive
mutators for your type definitions.

First, enable the `derive` feature in `Cargo.toml`:

```toml
[dependencies]
mutatis = { ..., features = ["derive"] }
```

Then simply slap `#[derive(Mutate)]` onto your type definitions:

```rust
# fn foo() -> mutatis::Result<()> {
#![cfg(feature = "derive")]
use mutatis::{Mutate, Session};

// The derive macro will automatically generate a `Vec2Mutator` type that
// implements `Mutate<Vec2>` and register it as the default mutator for
// `Vec2`s.
#[derive(Debug, Mutate)]
pub struct Vec2 {
    pub x: i32,
    pub y: i32,
}

// Define a vec2...
let mut v = Vec2 {
    x: 99,
    y: 1,
};

// ...and mutate it a bunch of times!
let mut session = Session::new();
for _ in 0..5 {
    session.mutate(&mut v)?;
    println!("mutated v is {v:?}");
}

// Example output:
//
//     mutated v is Vec2 { x: 99, y: 49968845 }
//     mutated v is Vec2 { x: 99, y: 1848354087 }
//     mutated v is Vec2 { x: -1320835063, y: 1848354087 }
//     mutated v is Vec2 { x: -1320835063, y: 1443194178 }
//     mutated v is Vec2 { x: -437057104, y: 1443194178 }
# Ok(())
# }
# #[cfg(feature = "derive")] foo().unwrap()
```

Derived mutators also have a `new` constructor that takes sub-mutators for each
field of their input type, which allows you to customize how each field is
mutated:

```rust
# fn foo() -> mutatis::Result<()> {
#![cfg(feature = "derive")]
use mutatis::{mutators as m, Mutate, Session};

#[derive(Debug, Mutate)]
pub struct MyType(u32, bool, u32);

// A `MyType` mutator that will only mutate values such that the following
// invariants hold true:
//
// * `10 <= value.0 <= 100`
// * `value.1 = true`
// * `0 <= value.2 <= 42`
let mut mutator = MyTypeMutator::new(
    m::range(10..=100),
    m::just(true),
    m::range(0..=42),
);

let mut value = MyType(1, false, 2);

let mut session = Session::new();
for _ in 0..5 {
    session.mutate_with(&mut mutator, &mut value)?;
    println!("mutated value is {value:?}");
}

// Example output:
//
//     mutated value is MyType(1, true, 2)
//     mutated value is MyType(1, true, 0)
//     mutated value is MyType(1, true, 18)
//     mutated value is MyType(73, true, 18)
//     mutated value is MyType(73, true, 14)
# Ok(())
# }
# #[cfg(feature = "derive")] foo().unwrap();
```

#### Container Attributes

The `#[derive(Mutate)]` macro supports the following attributes on `struct`s
and `enum`s:

* `#[mutatis(mutator_name = MyCoolName)]`: Generate a mutator type named
  `MyCoolName` instead of appending `Mutator` to the input type's name.

* `#[mutatis(mutator_doc = "my documentation")]`: Generate a custom doc comment
  for the generated mutator type. This may be repeated multiple times. The
  resulting doc comment is a concatenation of all occurrences.

* `#[mutatis(default_mutate = false)]`: Do not implement the `DefaultMutate`
  trait for the generated mutator type.

#### Field Attributes

The `#[derive(Mutate)]` macro suports the following attributes on fields within
`struct`s and `enum` variants:

* `#[mutatis(ignore)]`: Do not mutate this field.

* `#[mutatis(default_mutate)]`: Always use this field's type's `DefaultMutate`
  implementation to mutate this field. Do not generate a generic type parameter
  or argument to the generated mutator's constructor for mutating this field.

### Integrating Your Mutators with a Fuzzer

These are general steps for integrating your custom, `mutatis`-based mutators
into your fuzzing or property-based testing framework of choice:

* Identify the framework's mechanism for customizing mutations, for example the
  [`libfuzzer_sys::fuzz_mutator!`][fuzz-mutator] macro.

* Implement that mechanism by:

  1. Converting the framework's raw bytes for the test case into your structured
     test case type, if necessary.

     Most fuzzers, including `libfuzzer`, don't know anything about your
     structured types, they just manipulate byte buffers. So you'll need to
     convert the raw bytes into your structured test case type. If you don't
     otherwise have a natural way of doing that, like if you're fuzzing a parser
     and could just run the parser on the raw data, then a quick-and-easy trick
     to to use `serde` and `bincode` to deserialize the raw bytes into your
     structured type.

  2. Run your `mutatis`-based custom mutator on the structured test case.

  3. Convert the structured test case back into raw bytes for the framework, if
     necessary.

     This is the inverse of step (i). If you used `serde` and `bincode` in step
     (i) you would also want to use them here in step (iii).

[fuzz-mutator]: https://docs.rs/libfuzzer-sys/latest/libfuzzer_sys/macro.fuzz_mutator.html

#### Example with `libfuzzer-sys`

While `mutatis` is agnostic of which fuzzing engine or property-testing
framework you use, here's an example of using `mutatis` to define a custom
mutator for [`libfuzzer-sys`][libfuzzer]. Integrating `mutatis` with other
fuzzing engines' APIs should look pretty similar, mutatis mutandis.

This example defines two different color representations, RGB and HSL, as well
as conversions between them. The fuzz target asserts that roundtripping an RGB
color to HSL and back to RGB correctly results in the original RGB color. The
fuzz mutator converts the fuzzer's bytes into an RGB color, mutates the RGB
color, and then updates the fuzzer's test case based on the mutated RGB color.

```rust,no_run
#[cfg(feature = "derive")]
# mod example {
use libfuzzer_sys::{fuzzer_mutate, fuzz_mutator, fuzz_target};
use mutatis::{mutators as m, Mutate, Session};

/// A red-green-blue color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Mutate)]
pub struct Rgb([u8; 3]);

impl Rgb {
    /// Create an RGB color from 3 bytes.
    pub fn from_bytes(bytes: [u8; 3]) -> Self {
        Rgb(bytes)
    }

    /// Get the underlying bytes for an RGB color.
    pub fn to_bytes(self) -> [u8; 3] {
        self.0
    }

    /// Convert this color from RGB to HSL.
    pub fn to_hsl(self) -> Hsl {
        todo!()
    }
}

/// A hue-saturation-lightness color.
pub struct Hsl {
    // ...
}

impl Hsl {
    /// Convert this color from HSL to RGB.
    pub fn to_rgb(self) -> Rgb {
        todo!()
    }
}

// The fuzz target: assert that RGB-to-HSL-to-RGB is the identity function.
fuzz_target!(|data| {
    let bytes = match data.first_chunk::<3>() {
        Some(b) => *b,
        None => return,
    };

    let rgb = Rgb::from_bytes(bytes);
    let hsl = rgb.to_hsl();
    let rgb2 = hsl.to_rgb();

    assert_eq!(rgb, rgb2);
});

// The custom mutator: create an RGB color from the fuzzer's raw data, mutate
// it, update the fuzzer's raw data based on that mutated RGB color.
fuzz_mutator!(|data: &mut [u8], size: usize, max_size: usize, seed: u32| {
    let bytes = match data.first_chunk::<3>() {
        Some(b) => *b,
        // If we don't have enough bytes to mutate ourselves, use the fuzzer's
        // default mutation strategies.
        None => return fuzzer_mutate(data, size, max_size),
    };

    let mut rgb = Rgb::from_bytes(bytes);

    // Configure the mutation with the seed that libfuzzer gave us.
    let mut session = Session::new().seed(seed.into());

    // Mutate `rgb` using its default, derived mutator!
    match session.mutate(&mut rgb) {
        Ok(()) => {
            // Update the fuzzer's raw data based on the mutated RGB color.
            let new_bytes = rgb.to_bytes();
            let new_size = std::cmp::min(max_size, new_bytes.len());
            data[..new_size].copy_from_slice(&bytes[..new_size]);
            new_size
        }
        Err(_) => {
            // If we failed to mutate the test case, fall back to the fuzzer's
            // default mutation strategies.
            return fuzzer_mutate(data, size, max_size);
        }
    }

});
# }
```

### Shrinking Test Cases

You can configure a `Session` to only perform mutations that "shrink"
their given values. When paired with a property or predicate function, doing so
lets you easily build test-case reducers that find the smallest input that
triggers a bug.

```rust
# fn foo() -> mutatis::Result<()> {
use mutatis::{mutators as m, Mutate, Session};

// Configure mutation to only shrink the input.
let mut session = Session::new().shrink(true);

let mut value = u32::MAX;
for _ in 0..10 {
    session.mutate(&mut value)?;
    println!("shrunken value is {value}");
}

// Example output:
//
//     shrunken value is 1682887620
//     shrunken value is 363687628
//     shrunken value is 259482126
//     shrunken value is 49968845
//     shrunken value is 12933345
//     shrunken value is 9334495
//     shrunken value is 124077
//     shrunken value is 12325
//     shrunken value is 9732
//     shrunken value is 3837
# Ok(())
# }
# foo().unwrap()
```

When implementing `Mutate` by hand, rather than relying on the
`mutatis::mutator` module's combinators or the `derive(Mutate)` macro, be sure
to check whether you're inside a shrinking session and adjust your potential
mutations appropriately. Compared to the original input, a shrunken value should
be simpler and less complex, have fewer members inside its inner `Vec`s and
other container types, serialize to fewer bytes, etc... In general, checking for
shrinking is only something that collections and "leaf" `Mutate` implementations
need to worry about.

```rust
# fn foo() -> mutatis::Result<()> {
use mutatis::{
    mutators as m, Error, Generate, Mutate, Session, Candidates,
    Result, ResultExt,
};

/// A mutator that mutates `u32`s into powers of two.
pub struct PowersOfTwo;

impl Mutate<u32> for PowersOfTwo {
    fn mutate(
        &mut self,
        mutations: &mut Candidates,
        value: &mut u32,
    ) -> Result<()> {
        // If we should only shrink the value, then only generate powers of two
        // less than the input. Otherwise, generate any power of two `u32`.
        let max_log2 = if mutations.shrink() {
            match value.ilog2().checked_sub(value.is_power_of_two() as u32) {
                Some(s) => s,
                // There are no more powers of two to shrink to, so early return
                // and do not register any candidate mutations.
                None => return Ok(()),
            }
        } else {
            31
        };

        mutations.mutation(|context| {
            // Choose a random `log2(value)` between 0 and `max_log2`, inclusive.
            let log2 = m::range(0..=max_log2).generate(context)?;

            // value = 2^log2(value) = 1 << log2(value)
            *value = 1 << log2;

            Ok(())
        })?;

        Ok(())
    }
}

let mut value = u32::MAX;

// Configure mutation to only shrink the input.
let mut session = Session::new().shrink(true).seed(19);

for _ in 0..10 {
    session.mutate_with(&mut PowersOfTwo, &mut value).ignore_exhausted()?;
    println!("shrunken value is {value}");
}

// Example output:
//
//     shrunken value is 8388608
//     shrunken value is 8192
//     shrunken value is 128
//     shrunken value is 16
//     shrunken value is 1
//     shrunken value is 1
//     shrunken value is 1
//     shrunken value is 1
//     shrunken value is 1
//     shrunken value is 1
# Ok(())
# }
# foo().unwrap()
```

### Writing Smoke Tests with `mutatis::check`

When you enable the `check` feature in `Cargo.toml`, [the `mutatis::check`
module](https://docs.rs/mutatis/latest/mutatis/check/index.html) provides a tiny
property-based testing framework that is suitable for writing smoke tests. It is
not intended to replace a full-fledged fuzzing engine that you'd use for
in-depth, 24/7 fuzzing.

```rust
#[cfg(all(test, feature = "check"))]
mod tests {
    use mutatis::check::Check;

    #[test]
    fn test_that_addition_commutes() {
        Check::new()
            .iters(1000)
            .shrink_iters(1000)
            .run_with_defaults(|(a, b): &(i32, i32)| {
                if a + b == b + a {
                    Ok(())
                } else {
                    Err("addition is not commutative!")
                }
            })
            .unwrap();
    }
}
```

See [the module
documentation](https://docs.rs/mutatis/latest/mutatis/check/index.html) for more
details.

## Cargo Features

**Note: none of this crate's features are enabled by default. You most likely
want to enable `std`.**

* `alloc`: Enable mutators for types in Rust's `alloc` crate and internally
  use features that the `alloc` crate provides.

* `std`: Enable mutators for types in Rust's `std` crate and internally use
  features that the `std` crate provides.

* `log`: Enable logging with [the `log` crate](https://docs.rs/log).

* `check`: Enable the `mutatis::check` module for writing property-based smoke
  tests with `mutatis`.

* `derive`: Enable the `#[derive(Mutate)]` macro for automatically deriving
  mutators for your types

## Minimum Supported Rust Version

<!-- XXX: Keep this documented MSRV in sync with the `rust-version` in `Cargo.toml` -->

The minimum supported Rust version (MSRV) is currently **1.80.0**.

The MSRV will never be increased in a patch release, but may be increased in a
minor release. We will aim to avoid doing so without good reason.

## License

Licensed under dual MIT or Apache-2.0 at your choice.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
