#![cfg(all(feature = "derive", feature = "std"))]

use mutatis::{mutators as m, DefaultMutator, MutationContext, Mutator};

#[test]
fn derive_on_struct_with_named_fields() -> anyhow::Result<()> {
    #[derive(Debug, Default, Mutator)]
    struct MyStruct {
        x: u8,
        y: bool,
    }

    let mut context = MutationContext::default();
    let mut mutator = m::default::<MyStruct>();
    let mut value = MyStruct::default();
    mutator.mutate(&mut context, &mut value)?;
    Ok(())
}

#[test]
fn derive_on_struct_with_unnamed_fields() -> anyhow::Result<()> {
    #[derive(Debug, Default, Mutator)]
    struct MyStruct(u8, bool);

    let mut context = MutationContext::default();
    let mut mutator = m::default::<MyStruct>();
    let mut value = MyStruct::default();
    mutator.mutate(&mut context, &mut value)?;
    Ok(())
}

#[test]
fn derive_on_unit_struct() -> anyhow::Result<()> {
    #[derive(Debug, Default, Mutator)]
    struct MyUnitStruct;

    let mut context = MutationContext::default();
    let mut mutator = m::default::<MyUnitStruct>();
    let mut value = MyUnitStruct::default();
    mutator.mutate(&mut context, &mut value)?;
    Ok(())
}

#[test]
fn derive_on_enum() -> anyhow::Result<()> {
    #[derive(Debug, Mutator)]
    enum MyEnum {
        Unit,
        Unnamed(u8, bool),
        Named { x: u8, y: bool },
    }

    let mut context = MutationContext::default();
    let mut mutator = m::default::<MyEnum>();

    let mut value = MyEnum::Unit;
    mutator.mutate(&mut context, &mut value)?;

    let mut value = MyEnum::Unnamed(0, false);
    mutator.mutate(&mut context, &mut value)?;

    let mut value = MyEnum::Named { x: 0, y: false };
    mutator.mutate(&mut context, &mut value)?;

    Ok(())
}

#[test]
fn mutator_name() -> anyhow::Result<()> {
    #[derive(Debug, Default, Mutator)]
    #[mutatis(mutator_name = MyMutator)]
    struct MyStruct;

    let mut context = MutationContext::default();
    let mut mutator = MyMutator::default();
    let mut value = MyStruct::default();
    mutator.mutate(&mut context, &mut value)?;
    Ok(())
}

#[test]
fn mutator_doc() -> anyhow::Result<()> {
    #[derive(Debug, Default, Mutator)]
    #[mutatis(mutator_doc = "This is a cool mutator")]
    struct MyStruct;

    let mut context = MutationContext::default();
    let mut mutator = m::default::<MyStruct>();
    let mut value = MyStruct::default();
    mutator.mutate(&mut context, &mut value)?;
    Ok(())
}

#[test]
fn ignore_field() -> anyhow::Result<()> {
    #[derive(Clone, Debug, Default, PartialEq, Eq, Mutator)]
    struct MyStruct {
        x: u64,

        #[mutatis(ignore)]
        y: u64,
    }

    let mut context = MutationContext::default();
    let mut mutator = m::default::<MyStruct>();

    let orig = MyStruct::default();
    let mut value = orig.clone();

    while value == orig {
        mutator.mutate(&mut context, &mut value)?;
        assert_eq!(orig.y, value.y);
    }

    Ok(())
}

#[test]
fn default_mutator() -> anyhow::Result<()> {
    #[derive(Debug, Default, Mutator)]
    struct MyStruct {
        x: u64,

        #[mutatis(default_mutator)]
        y: u64,
    }

    let mut context = MutationContext::default();

    // Only an `x` mutator parameter because `y` is always the default mutator.
    let mut mutator = MyStructMutator::new(m::u64());

    let mut value = MyStruct::default();
    mutator.mutate(&mut context, &mut value)?;

    Ok(())
}

#[test]
fn derive_with_generic_parameters() -> anyhow::Result<()> {
    #[derive(Debug, Mutator)]
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

    let mut context = MutationContext::default();
    let mut mutator = m::default::<MyGenericStruct<'_, '_, 3, u32, i8>>();

    let x = 5;
    let y = 10;
    let z = [1, 2, 3];
    let w = 100;
    let mut value = MyGenericStruct { x: &x, y: &y, z, w };

    mutator.mutate(&mut context, &mut value)?;

    Ok(())
}

#[test]
fn no_default_mutator() -> anyhow::Result<()> {
    #[derive(Debug, Mutator)]
    #[mutatis(default_mutator = false)]
    struct MyStruct {
        x: u64,
    }

    // If the derive macro had emitted a `DefaultMutator` implementation, then
    // this one would be a compile error.
    impl DefaultMutator for MyStruct {
        type DefaultMutator = MyStructMutator<m::U64>;
    }

    let mut context = MutationContext::default();
    let mut mutator = MyStructMutator::new(m::u64());
    let mut value = MyStruct { x: 0 };
    mutator.mutate(&mut context, &mut value)?;
    Ok(())
}

fn main() {}
