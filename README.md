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

1. **[The `mutatis::Mutator`
   trait.](https://docs.rs/mutatis/latest/mutatis/trait.Mutator.html)** A trait
   that is implemented by types which can mutate other types. The
   `mutatis::Mutator::mutate` trait method takes a value and mutates it. You can
   think of a `mutatis::Mutator` implementation like a streaming iterator that
   takes as input and modifies items, rather than generating them from scratch
   and returning them.

2. **[The `mutatis::mutators`
   module.](https://docs.rs/mutatis/latest/mutatis/mutators/index.html)** This
   module, idiomatically imported via `use mutatis::mutators as m`, provides a
   types and combinators for building custom mutators.

Here's an example of using `mutatis` to define a custom mutator for a simple
data structure:

```rust
# fn foo() -> mutatis::Result<()> {
use mutatis::{mutators as m, MutationContext, Mutator};

/// A silly monster type.
#[derive(Debug)]
pub struct Monster {
    hp: u16,
    is_ghost: bool,
    pos: [i32; 2],
}

/// A mutator that chooses one of a monster's fields to mutate.
let mut mutator = m::one_of((
    m::u16().proj(|x: &mut Monster| &mut x.hp),
    m::bool().proj(|x: &mut Monster| &mut x.is_ghost),
    m::array(m::i32()).proj(|x: &mut Monster| &mut x.pos),
));

// Define a monster...
let mut monster = Monster {
    hp: 36,
    is_ghost: true,
    pos: [42, -8],
};

// ...and mutate it a bunch of times!
let mut context = MutationContext::default();
for _ in 0..5 {
    mutator.mutate(&mut context, &mut monster)?;
    println!("mutated monster is {monster:?}");
}

// Example output:
//
//     mutated monster is Monster { hp: 25654, is_ghost: true, pos: [42, -8] }
//     mutated monster is Monster { hp: 25654, is_ghost: false, pos: [42, -8] }
//     mutated monster is Monster { hp: 61108, is_ghost: false, pos: [42, -8] }
//     mutated monster is Monster { hp: 61108, is_ghost: false, pos: [-1166784619, -8] }
# Ok(())
# }
```

### Automatically Deriving Mutators with `#[derive(Mutator)]`

If you enable this crate's `derive` feature, then you can automatically derive
mutators for your type definitions.

First, enable the `derive` feature in `Cargo.toml`:

```toml
[dependencies]
mutatis = { ..., features = ["derive"] }
```

Then simply slap `#[derive(Mutator)]` onto your type definitions:

```rust
# fn foo() -> mutatis::Result<()> {
#![cfg(feature = "derive")]
use mutatis::{mutators as m, MutationContext, Mutator};

/// A silly monster type that derives `Mutator`.
#[derive(Debug, Mutator)]
pub struct Monster {
    hp: u16,
    is_ghost: bool,
    pos: [i32; 2],
}

// The derive macro will automatically generate a `MonsterMutator` type that
// implements `Mutator<Monster>` and register it as the default mutator for
// `Monster`s.
let mut mutator = m::default::<Monster>();

// Define a monster...
let mut monster = Monster {
    hp: 36,
    is_ghost: true,
    pos: [42, -8],
};

// ...and mutate it a bunch of times!
let mut context = MutationContext::default();
for _ in 0..5 {
    mutator.mutate(&mut context, &mut monster)?;
    println!("mutated monster is {monster:?}");
}

// Example output:
//
//     mutated monster is Monster { hp: 36, is_ghost: false, pos: [42, -8] }
//     mutated monster is Monster { hp: 36, is_ghost: false, pos: [42, -982287921] }
//     mutated monster is Monster { hp: 36, is_ghost: false, pos: [42, 1443194178] }
//     mutated monster is Monster { hp: 36, is_ghost: true, pos: [42, 1443194178] }
//     mutated monster is Monster { hp: 37582, is_ghost: true, pos: [42, 1443194178] }
# Ok(())
# }
```

The generated mutator also has a constructor that takes sub-mutators for each
field of the input type, which allows you to customize how each field is
mutated:

```rust
# fn foo() -> mutatis::Result<()> {
#![cfg(feature = "derive")]
use mutatis::{mutators as m, MutationContext, Mutator};

#[derive(Debug, Mutator)]
pub struct MyType(u32, u32);

// A `MyType` mutator that will only generate inner values within the given
// ranges.
let mut mutator = MyTypeMutator::new(m::range(10..=100), m::range(0..=42));

let mut value = MyType(1, 2);
let mut context = MutationContext::default();
for _ in 0..5 {
    mutator.mutate(&mut context, &mut value)?;
    println!("mutated value is {value:?}");
}

// Example output:
//
//     mutated value is MyType(11, 2)
//     mutated value is MyType(38, 2)
//     mutated value is MyType(38, 41)
//     mutated value is MyType(35, 41)
//     mutated value is MyType(35, 24)
# Ok(())
# }
```

#### Container Attributes

The `#[derive(Mutator)]` macro supports the following attributes on `struct`s
and `enum`s:

* `#[mutatis(mutator_name = MyCoolName)]`: Generate a mutator type named
  `MyCoolName` instead of appending `Mutator` to the input type's name.

* `#[mutatis(mutator_doc = "my documentation")]`: Generate a custom doc comment
  for the generated mutator type. This may be repeated multiple times. The
  resulting doc comment is a concatenation of all occurrences.

* `#[mutatis(default_mutator = false)]`: Do not implement the `DefaultMutator`
  trait for the generated mutator type.

#### Field Attributes

The `#[derive(Mutator)]` macro suports the following attributes on fields within
`struct`s and `enum` variants:

* `#[mutatis(ignore)]`: Do not mutate this field.

* `#[mutatis(default_mutator)]`: Always use this field's type's `DefaultMutator`
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
use mutatis::{mutators as m, MutationContext, Mutator};

/// A red-green-blue color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Mutator)]
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

    // Create a mutator for the RGB color.
    let mut mutator = m::default::<Rgb>();

    // Configure the mutation with the seed that libfuzzer gave us.
    let mut context = MutationContext::builder()
        .seed(seed.into())
        .build();

    // Mutate the RGB color!
    mutator.mutate(&mut context, &mut rgb);

    // Update the fuzzer's raw data based on the mutated RGB color.
    let new_bytes = rgb.to_bytes();
    let new_size = std::cmp::min(max_size, new_bytes.len());
    data[..new_size].copy_from_slice(&bytes[..new_size]);
    new_size
});
# }
```

### Shrinking Test Cases

You can configure a `MutationContext` to only perform mutations that "shrink"
their given values. When paired with a property or predicate function, doing so
lets you easily build test-case reducers that find the smallest input that
triggers a bug.

```rust
# fn foo() -> mutatis::Result<()> {
use mutatis::{mutators as m, MutationContext, Mutator};

// Configure mutation to only shrink the input.
let mut context = MutationContext::builder().shrink(true).build();

let mut value = u32::MAX;
for _ in 0..10 {
    m::default::<u32>().mutate(&mut context, &mut value)?;
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

When implementing `Mutator` by hand, rather than relying on the
`mutatis::mutator` module's combinators or the `derive(Mutator)` macro, be sure
to check whether `context.shrink()` returns `true` and adjust your mutation
strategy appropriately. Compared to the original input, a shrunken value should
be simpler and less complex, have fewer members inside its inner `Vec`s and
other container types, serialize to fewer bytes, etc...

```rust
# fn foo() -> mutatis::Result<()> {
use mutatis::{
    mutators as m, Error, GenerativeMutator, MutationContext, Mutator, Result,
    ResultExt,
};

/// A mutator that mutates `u32`s into powers of two.
pub struct Pow2Mutator;

impl Mutator<u32> for Pow2Mutator {
    fn mutate(
        &mut self,
        context: &mut MutationContext,
        value: &mut u32,
    ) -> Result<()> {
        // If we should only shrink the value, then only generate powers of two
        // less than the input. Otherwise, generate any power of two `u32`.
        let max_shift = if context.shrink() {
            value.ilog2().checked_sub(value.is_power_of_two() as u32).ok_or_else(|| {
                // There are more powers of two to shrink to.
                Error::mutator_exhausted()
            })?
        } else {
            31
        };
        dbg!(max_shift);

        // Choose a random `log2(value)` between 0 and `max_shift`, inclusive.
        let log2 = m::range(0..=max_shift).generate(context)?;
        dbg!(log2);

        // value = 2^log2(value) = 1 << log2(value)
        *value = 1 << log2;

        Ok(())
    }
}

// Configure mutation to only shrink the input.
let mut context = MutationContext::builder().shrink(true).build();

let mut value = u32::MAX;
for _ in 0..3 {
    Pow2Mutator.mutate(&mut context, &mut value).ignore_mutator_exhausted()?;
    println!("shrunken value is {value}");
}

// Example output:
//
//     shrunken value is 65536
//     shrunken value is 4096
//     shrunken value is 512
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

* `alloc`: Implement `Mutator`s for types in Rust's `alloc` crate and internally
  use features that the `alloc` crate provides.

* `std`: Implement `Mutator`s for types in Rust's `std` crate and internally use
  features that the `std` crate provides.

* `check`: Enable the `mutatis::check` module for writing property-based smoke
  tests with `mutatis`.

* `derive`: Enable the `#[derive(Mutator)]` macro for automatically deriving
  `Mutator` implementations for your types

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
