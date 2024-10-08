#![cfg(all(feature = "derive", feature = "std"))]

use mutatis::{error::ResultExt, mutators as m, DefaultMutate, Mutate, Session};

#[test]
fn derive_on_struct_with_named_fields() -> anyhow::Result<()> {
    #[derive(Debug, Default, Mutate)]
    struct MyStruct {
        x: u8,
        y: bool,
    }

    let mut session = Session::new();
    let mut value = MyStruct::default();
    session.mutate(&mut value)?;
    Ok(())
}

#[test]
fn derive_on_struct_with_unnamed_fields() -> anyhow::Result<()> {
    #[derive(Debug, Default, Mutate)]
    struct MyStruct(u8, bool);

    let mut session = Session::new();
    let mut value = MyStruct::default();
    session.mutate(&mut value)?;
    Ok(())
}

#[test]
fn derive_on_unit_struct() -> anyhow::Result<()> {
    #[derive(Debug, Default, Mutate)]
    struct MyUnitStruct;

    let mut session = Session::new();
    let mut value = MyUnitStruct::default();
    session.mutate(&mut value).ignore_exhausted()?;
    Ok(())
}

#[test]
fn derive_on_enum() -> anyhow::Result<()> {
    #[derive(Debug, Mutate)]
    enum MyEnum {
        Unit,
        Unnamed(u8, bool),
        Named { x: u8, y: bool },
    }

    let mut session = Session::new();

    let mut value = MyEnum::Unit;
    session
        .mutate(&mut value)
        // TODO: support mutating from one enum variant to another
        .ignore_exhausted()?;

    let mut value = MyEnum::Unnamed(0, false);
    session.mutate(&mut value)?;

    let mut value = MyEnum::Named { x: 0, y: false };
    session.mutate(&mut value)?;

    Ok(())
}

#[test]
fn mutator_name() -> anyhow::Result<()> {
    #[derive(Debug, Default, Mutate)]
    #[mutatis(mutator_name = MyCoolMutator)]
    struct MyStruct(u8);

    let mut session = Session::new();
    let mut mutator = MyCoolMutator::new(m::u8());
    let mut value = MyStruct::default();
    session.mutate_with(&mut mutator, &mut value)?;
    Ok(())
}

#[test]
fn mutator_doc() -> anyhow::Result<()> {
    #[derive(Debug, Default, Mutate)]
    #[mutatis(mutator_doc = "This is a cool mutator")]
    struct MyStruct(u8);

    let mut session = Session::new();
    let mut value = MyStruct::default();
    session.mutate(&mut value)?;
    Ok(())
}

#[test]
fn ignore_field() -> anyhow::Result<()> {
    #[derive(Clone, Debug, Default, PartialEq, Eq, Mutate)]
    struct MyStruct {
        x: u64,

        #[mutatis(ignore)]
        y: u64,
    }

    let mut session = Session::new();

    let orig = MyStruct::default();
    let mut value = orig.clone();

    while value == orig {
        session.mutate(&mut value)?;
        assert_eq!(orig.y, value.y);
    }

    Ok(())
}

#[test]
fn default_mutate_field() -> anyhow::Result<()> {
    #[derive(Debug, Default, Mutate)]
    struct MyStruct {
        x: u64,

        #[mutatis(default_mutate)]
        y: u64,
    }

    let mut session = Session::new();

    // Only an `x` mutator parameter because `y` is always the default mutator.
    let mut mutator = MyStructMutator::new(m::u64());

    let mut value = MyStruct::default();
    session.mutate_with(&mut mutator, &mut value)?;

    Ok(())
}

#[test]
fn derive_with_generic_parameters() -> anyhow::Result<()> {
    #[derive(Debug, Mutate)]
    struct MyGenericStruct<'a, 'b: 'a, const N: usize, T: Copy, U>
    where
        U: Default,
    {
        #[mutatis(ignore)]
        #[allow(dead_code)]
        x: &'a T,

        #[mutatis(ignore)]
        #[allow(dead_code)]
        y: &'b T,

        z: [T; N],
        w: U,
    }

    let mut session = Session::new();

    let x = 5;
    let y = 10;
    let z = [1, 2, 3];
    let w = 100;
    let mut value = MyGenericStruct { x: &x, y: &y, z, w };

    session.mutate(&mut value)?;

    Ok(())
}

#[test]
fn no_default_mutator() -> anyhow::Result<()> {
    #[derive(Debug, Mutate)]
    #[mutatis(default_mutate = false)]
    struct MyStruct {
        x: u64,
    }

    // If the derive macro had emitted a `DefaultMutate` implementation, then
    // this one would be a compile error.
    impl DefaultMutate for MyStruct {
        type DefaultMutate = MyStructMutator<m::U64>;
    }

    let mut session = Session::new();
    let mut mutator = MyStructMutator::new(m::u64());
    let mut value = MyStruct { x: 0 };
    session.mutate_with(&mut mutator, &mut value)?;
    Ok(())
}
