/*!

# The `#[derive(Mutate)]` Macro

The `#[derive(Mutate)]` macro is only available when the `derive` cargo feature
is enabled:

```toml
[dependencies]
mutatis = { version = "...", features = ["derive"] }
```

Once the `derive` feature is enabled, you can put `#[derive(Mutate)]` on top of
your `struct` and `enum` definitions. It supports unit-, tuple-, and
named-field-styles of `struct`s and `enum` variants. You cannot derive a mutator
for `union`s.

```rust
# fn foo() -> mutatis::Result<()> {
# #![cfg(feature = "derive")]
use mutatis::Mutate;

#[derive(Mutate)]
pub struct Hero {
    needed: bool,
    deserved: bool,
    armor: Option<u32>,
}
# Ok(())
# }
# #[cfg(feature = "derive")] foo().unwrap();
```

The derive macro generates the following:

### A `HeroMutator` type

```rust
# use mutatis::DefaultMutate;
// A mutator for `Hero` values.
pub struct HeroMutator<
    MutatorT0 = <bool as DefaultMutate>::DefaultMutate,
    MutatorT1 = <bool as DefaultMutate>::DefaultMutate,
    MutatorT2 = <Option<u32> as DefaultMutate>::DefaultMutate,
> {
#   _priv: (MutatorT0, MutatorT1, MutatorT2),
    // ...
}
```

The name of the generated type can be customized with the `mutator_name`
attribute. See the "Container Attributes" section below.

The generated mutator type has the same visibility (`pub`, `pub(crate)`, etc...)
as the original type.

You can customize the documentation comment for the generated mutator type with
the `#[mutatis(mutator_doc = "...")]` attribute. See the "Container Attributes"
section below.

The generated mutator has a generic type parameter for each field in the
`struct`, or each field in each variant of the `enum`. These generic type
parameters control how their associated field is mutated. The exception being
any fields marked with `#[mutatis(ignore)]`, whose associated fields are never
mutated, or any fields marked with `#[mutatis(default_mutator)]`, which will
always use the default mutator. See the "Field Attributes" section below for
more details.

Each generic type parameter defaults to the default mutator of its associated
field's type, unless the `enum`/`struct` is annotated with
`#[mutatis(default_mutate = false)]`.

### A `Mutate<Hero> for HeroMutator` implementation

```rust
# struct Hero { }
# struct HeroMutator<MutatorT0, MutatorT1, MutatorT2> { _priv: (MutatorT0, MutatorT1, MutatorT2) }
# trait Mutate<X> {}
impl<MutatorT0, MutatorT1, MutatorT2> Mutate<Hero>
    for HeroMutator<MutatorT0, MutatorT1, MutatorT2>
where
    MutatorT0: Mutate<bool>,
    MutatorT1: Mutate<bool>,
    MutatorT2: Mutate<Option<u32>>,
{
    // ...
}
```

### A `HeroMutator::new` constructor

```rust,no_run
# struct HeroMutator<MutatorT0, MutatorT1, MutatorT2> { _priv: (MutatorT0, MutatorT1, MutatorT2) }
impl<MutatorT0, MutatorT1, MutatorT2> HeroMutator<MutatorT0, MutatorT1, MutatorT2> {
    pub fn new(needed: MutatorT0, deserved: MutatorT1, armor: MutatorT2) -> Self {
#       todo!()
        // ...
    }
}
```

This constructor takes a parameter for each of the mutator type's generic type
parameters.

Any mutator you pass into the constructor will be used whenever the mutator is
mutating the associated field. For example, if we always wanted
`armor.is_some()`, then we could do the following:

```rust,no_run
use mutatis::mutators as m;

# struct HeroMutator<MutatorT0, MutatorT1, MutatorT2> { _priv: (MutatorT0, MutatorT1, MutatorT2) }
# impl<MutatorT0, MutatorT1, MutatorT2> HeroMutator<MutatorT0, MutatorT1, MutatorT2> {
#     pub fn new(needed: MutatorT0, deserved: MutatorT1, armor: MutatorT2) -> Self { todo!() }
# }
// Create a `HeroMutator` that mutates `Hero`s such that they always have
// some armor.
let mut mutator = HeroMutator::new(m::bool(), m::bool(), m::some(m::u32()));
```

### A `Default for HeroMutator` implementation

```rust
# struct HeroMutator<MutatorT0, MutatorT1, MutatorT2> { _priv: (MutatorT0, MutatorT1, MutatorT2) }
impl<MutatorT0, MutatorT1, MutatorT2> Default for HeroMutator<MutatorT0, MutatorT1, MutatorT2> {
    fn default() -> Self {
#       todo!()
        // ...
    }
}
```

This is omitted if the `#[mutatis(default_mutate = false)]` attribute is
present on the container. See the "Container Attributes" section below for
more details.

## Container Attributes

The `#[derive(Mutate)]` macro supports the following attributes on `struct`s
and `enum`s:

### `#[mutatis(mutator_name = MyCoolName)]`

Generate a mutator type named `MyCoolName` instead of appending `Mutator` to the
input type's name.

```rust
# fn foo() -> mutatis::Result<()> {
# #![cfg(feature = "derive")]
use mutatis::{mutators as m, Mutate};

#[derive(Mutate)]
#[mutatis(mutator_name = TheMutatorForFoo)]
pub struct Foo(u32);

let mut mutator = TheMutatorForFoo::new(m::just(5));
# Ok(())
# }
# #[cfg(feature = "derive")] foo().unwrap();
```

### `#[mutatis(mutator_doc = "my documentation")]`

Generate a custom doc comment for the generated mutator type. This may be
repeated multiple times. The resulting doc comment is a concatenation of all
occurrences.

```rust
# fn foo() -> mutatis::Result<()> {
# #![cfg(feature = "derive")]
use mutatis::Mutate;

#[derive(Mutate)]
#[mutatis(mutator_doc = r###"
This is a mutator for `Foo` values.

You can use with with the `mutatis` crate and its combinators to perform
pseudo-random mutations on `Foo` values.

Etc...
"###)]
pub struct Foo(u32);
# Ok(())
# }
# #[cfg(feature = "derive")] foo().unwrap();
```

### `#[mutatis(default_mutate = false)]`

Do not implement the `DefaultMutate` trait for the generated mutator type.

This is useful if the container type contains a field whose type does not
implement `DefaultMutate`, or you want to customize the `DefaultMutate`
implementation yourself.

```rust
# fn foo() -> mutatis::Result<()> {
# #![cfg(feature = "derive")]
use mutatis::{mutators as m, DefaultMutate, Mutate};

#[derive(Default, Mutate)]
#[mutatis(default_mutate = false)]
pub struct Foo(u32);

// Implement `DefaultMutate` ourselves with a particular sub-mutator, rather
// than the default mutator for `u32`.
impl DefaultMutate for Foo {
    type DefaultMutate = FooMutator<m::Just<u32>>;
}
# Ok(())
# }
# #[cfg(feature = "derive")] foo().unwrap();
```

## Field Attributes

The `#[derive(Mutate)]` macro suports the following attributes on fields within
`struct`s and `enum` variants:

### `#[mutatis(ignore)]`

Do not mutate this field. Do not generate a generic type parameter on the
mutator for it, nor an argument in the mutator's constructor for it.

```rust
# fn foo() -> mutatis::Result<()> {
# #![cfg(feature = "derive")]
use mutatis::{Mutate, Session};

#[derive(Clone, Default, Mutate)]
struct MyStruct {
    x: u64,

    #[mutatis(ignore)]
    y: u64,
}

let mut session = Session::new();

let orig = MyStruct::default();
let mut value = orig.clone();

for _ in 0..100 {
    session.mutate(&mut value)?;
    assert_eq!(orig.y, value.y);
}
# Ok(())
# }
# #[cfg(feature = "derive")] foo().unwrap();
```

### `#[mutatis(default_mutate)]`

Always use this field's type's `DefaultMutate` implementation to mutate this
field. Do not generate a generic type parameter or argument to the generated
mutator's constructor for mutating this field.

```rust
# fn foo() -> mutatis::Result<()> {
# #![cfg(feature = "derive")]
use mutatis::{mutators as m, Mutate, Session};

#[derive(Debug, Default, Mutate)]
struct MyStruct {
    x: u64,

    #[mutatis(default_mutate)]
    y: u64,
}

let mut session = Session::new();

// Only an `x` mutator argument because `y` always uses the default mutator.
let mut mutator = MyStructMutator::new(m::range(10..=19));

let mut value = MyStruct::default();
session.mutate_with(&mut mutator, &mut value)?;
# Ok(())
# }
# #[cfg(feature = "derive")] foo().unwrap();
```

 */
