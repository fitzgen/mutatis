<div align="center">
  <h1><code>mutatis</code></h1>
  <p>
    <strong>Easily create custom, structure-aware mutators for fuzzing.</strong>
  </p>
  <p>
    <a href="https://crates.io/crates/mutatis"><img src="https://img.shields.io/crates/v/mutatis.svg" alt="crates.io"></a>
    <a href="https://docs.rs/mutatis"><img src="https://docs.rs/mutatis/badge.svg" alt="docs.rs"></a>
    <img src="https://img.shields.io/badge/rustc-stable+-green.svg" alt="supported rustc stable" />
  </p>
  <h3>
    <a href="https://github.com/fitzgen/mutatis">Repository</a>
    <span> | </span>
    <a href="https://docs.rs/mutatis">Docs</a>
    <span> | </span>
    <a href="https://docs.rs/mutatis/latest/mutatis/_guide/index.html">Guide</a>
    <span> | </span>
    <a href="https://github.com/fitzgen/mutatis/blob/main/CONTRIBUTING.md">Contributing</a>
  </h3>
</div>

## About

The most popular fuzzers — including [`libfuzzer`][libfuzzer] and [AFL] — are
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
behavior as well. Consider the scenario where we are fuzzing a compiler: if some
input made it all the way through the parser, type checker, and into code
generation &mdash; rather than bouncing off early due to an invalid token
&mdash; then a new input derived from this one is also likely to go deep into
the compiler's pipeline. At least it is more likely to do so than a completely
new, random string.

But what happens when we aren't fuzzing a text or binary interface? What happens
when we have a custom input type that the fuzzer's built-in mutation strategies
aren't very good at targeting? Many fuzzers will expose a hook for customizing
the routine for mutating an existing input from its corpus to create a new
candidate input, for example `libfuzzer` has the [`fuzz_mutator!`][fuzz-mutator]
hook.

**`mutatis` exists to make writing these custom mutators easy and efficient.**

[fuzz-mutator]: https://docs.rs/libfuzzer-sys/latest/libfuzzer_sys/macro.fuzz_mutator.html

## Using Default Mutators

To randomly mutate a value with its default, off-the-shelf mutator:

* Create a [`mutatis::Session`](https://docs.rs/mutatis/latest/mutatis/struct.Session.html).
* Call
  [`session.mutate`](https://docs.rs/mutatis/latest/mutatis/struct.Session.html#method.mutate),
  passing in the value you wish to mutate.

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

## Combining and Customizing Mutators

You can use the mutator combinators in the
[`mutatis::mutators`](https://docs.rs/mutatis/latest/mutatis/mutators/index.html)
module to build more complex mutators from simpler ones or to customize mutation
strategies to, for example, maintain a type's internal invariants or bound the
resulting values into a particular range. The `mutatis::mutators` module is
typically imported under the alias `m`.

To randomly mutate a value with a custom mutator:

* Create the custom mutator from
  [`mutatis::mutators`](https://docs.rs/mutatis/latest/mutatis/mutators/)
  combinators and `Mutate` trait adapter methods.
* Create a [`mutatis::Session`](https://docs.rs/mutatis/latest/mutatis/struct.Session.html).
* Call
  [`session.mutate_with`](https://docs.rs/mutatis/latest/mutatis/struct.Session.html#method.mutate_with),
  passing in the value you wish to mutate and the mutator you wish to use to
  perform the mutation.

Here's an example of using `mutatis` to define a custom mutator for a custom
`struct` type that has multiple fields, and maintains a relationship between the
fields' values:

```rust
# fn foo() -> mutatis::Result<()> {
use mutatis::{mutators as m, Mutate, Session};

/// A scary monster type.
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

## Automatically Deriving Mutators with `#[derive(Mutate)]`

First, enable this crate's `derive` feature, then slap `#[derive(Mutate)]` onto
your type definitions:

```rust
# fn foo() -> mutatis::Result<()> {
#![cfg(feature = "derive")]
use mutatis::{Mutate, Session};

// An RGB color.
#[derive(Debug)]
#[derive(Mutate)] // Automatically derive a mutator for `Rgb`!
pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

// Create an RGB color: chartreuse.
let mut color = Rgb {
    r: 0x7f,
    g: 0xff,
    b: 0x00,
};

// ...and mutate it a bunch of times!
let mut session = Session::new();
for _ in 0..5 {
    session.mutate(&mut color)?;
    println!("mutated color is {color:?}");
}

// Example output:
//
//     mutated color is Rgb { r: 127, g: 45, b: 0 }
//     mutated color is Rgb { r: 127, g: 134, b: 0 }
//     mutated color is Rgb { r: 127, g: 10, b: 0 }
//     mutated color is Rgb { r: 127, g: 10, b: 29 }
//     mutated color is Rgb { r: 172, g: 10, b: 29 }
# Ok(())
# }
# #[cfg(feature = "derive")] foo().unwrap()
```

## Writing Smoke Tests with `mutatis::check`

When you enable the `check` feature in `Cargo.toml`, [the `mutatis::check`
module](https://docs.rs/mutatis/latest/mutatis/check/index.html) provides a tiny
property-based testing framework that is suitable for writing smoke tests that
you use for local development and CI. It is not intended to replace a
full-fledged, coverage-guided fuzzing engine that you'd use for in-depth,
continuous fuzzing.

```rust
# #[cfg(feature = "check")]
#[cfg(test)]
mod tests {
    use mutatis::check::Check;

    #[test]
    fn test_that_addition_commutes() {
        Check::new()
            .iters(1000)
            .shrink_iters(1000)
            .run(|(a, b): &(i32, i32)| {
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

See [the `check` module's
documentation](https://docs.rs/mutatis/latest/mutatis/check/index.html) for more
details.

## Documentation

#### API Reference Documentation

The API reference documentation is available on
[docs.rs](https://docs.rs/mutatis).

#### Guide

[Check out the guide](https://docs.rs/mutatis/latest/mutatis/guide/index.html)
for tutorials, discussions, and recipes; everything else that doesn't fall into
the API-reference category.

## License

Licensed under dual MIT or Apache-2.0 at your choice.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
