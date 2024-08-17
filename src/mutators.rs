//! The provided set of [`Mutate`] implementations and combinators.
//!
//! It is idiomatic to import this module with the alias `m`:

use super::*;
use core::ops;
use rand::Rng;

mod combinators;
pub use combinators::*;

// TODO: mod alloc;
// TODO: pub use alloc::*;

// TODO: mod std;
// TODO: pub use std::*;

/// A convenience function to get the default mutator for a type.
///
/// This is equivalent to `<T as DefaultMutate>::DefaultMutate::default()` but a
/// little less wordy.
pub fn default<T>() -> <T as DefaultMutate>::DefaultMutate
where
    T: DefaultMutate,
{
    T::DefaultMutate::default()
}

/// The default mutator for `bool` values.
///
/// See the [`bool()`] function to create new instances and for example usage.
#[derive(Clone, Debug, Default)]
pub struct Bool {
    _private: (),
}

/// Create a new `bool` mutator.
///
/// # Example
///
/// ```
/// use mutatis::{mutators as m, Mutate, MutationBuilder};
///
/// let mut mutator = m::bool();
/// let mut mtn = MutationBuilder::new();
///
/// let mut value = true;
/// mtn.mutate_with(&mut mutator, &mut value).unwrap();
///
/// assert_eq!(value, false);
/// ```
pub fn bool() -> Bool {
    Bool { _private: () }
}

impl Mutate<bool> for Bool {
    #[inline]
    fn mutate(&mut self, muts: &mut MutationSet, value: &mut bool) -> crate::Result<()> {
        if !muts.shrink() || *value {
            muts.mutation(|_ctx| Ok(*value = !*value))?;
        }
        Ok(())
    }
}

impl DefaultMutate for bool {
    type DefaultMutate = Bool;
}

macro_rules! ints {
    (
        $(
            $fn_name:ident -> $ty_name:ident : $method:ident for $ty:ty ;
        )*
    ) => {
        $(
            /// A mutator for
            #[doc = concat!("`", stringify!($ty), "`")]
            /// values.
            ///
            /// See the
            #[doc = concat!("[`", stringify!($fn_name), "()`]")]
            /// function to create new instances and for
            /// example usage.
            #[derive(Clone, Debug, Default)]
            pub struct $ty_name {
                _private: (),
            }

            /// Create a new
            #[doc = concat!("`", stringify!($ty), "`")]
            /// mutator.
            ///
            /// # Example
            ///
            /// ```
            /// use mutatis::{mutators as m, Mutate, MutationBuilder};
            ///
            #[doc = concat!("let mut mutator = m::", stringify!($fn_name), "();")]
            ///
            /// let mut mtn = MutationBuilder::new().shrink(true);
            ///
            /// let mut value = 42;
            /// mtn.mutate_with(&mut mutator, &mut value).unwrap();
            ///
            /// assert!(value < 42);
            /// ```
            pub fn $fn_name() -> $ty_name {
                $ty_name { _private: () }
            }

            impl Mutate<$ty> for $ty_name {
                #[inline]
                fn mutate(&mut self, muts: &mut MutationSet, value: &mut $ty) -> crate::Result<()> {
                    if muts.shrink() {
                        if *value != 0 {
                            muts.mutation(|ctx| {
                                *value = ctx.rng().inner().gen_range(0..=*value);
                                Ok(())
                            })?;
                        }
                        Ok(())
                    } else {
                        muts.mutation(|ctx| Ok(*value = ctx.rng().$method()))?;
                        Ok(())
                    }
                }
            }

            impl DefaultMutate for $ty {
                type DefaultMutate = $ty_name;
            }

            impl Generate<$ty> for $ty_name {
                #[inline]
                fn generate(&mut self, ctx: &mut MutationContext) -> crate::Result<$ty> {
                    Ok(ctx.rng().$method())
                }
            }

            impl MutateInRange<$ty> for $ty_name {
                #[inline]
                fn mutate_in_range(
                    &mut self,
                    muts: &mut MutationSet,
                    value: &mut $ty,
                    range: &ops::RangeInclusive<$ty>,
                ) -> crate::Result<()> {
                    let start = *range.start();
                    let end = *range.end();

                    if start > end {
                        return Err(Error::invalid_range());
                    }

                    if *value == start && muts.shrink() {
                        return Ok(());
                    }

                    muts.mutation(|ctx| {
                        let end = if ctx.shrink() {
                            core::cmp::min(*value, end)
                        } else {
                            end
                        };

                        let new_value = ctx.rng().inner().gen_range(start..=end);
                        debug_assert!(
                            start <= new_value && new_value <= end,
                            "{start} <= {new_value} <= {end}",
                        );
                        *value = new_value;
                        Ok(())
                    })
                }
            }
        )*
    };
}

ints! {
    u8 -> U8 : gen_u8 for u8;
    u16 -> U16 : gen_u16 for u16;
    u32 -> U32 : gen_u32 for u32;
    u64 -> U64 : gen_u64 for u64;
    u128 -> U128 : gen_u128 for u128;
    usize -> USIZE : gen_usize for usize;
    i8 -> I8 : gen_i8 for i8;
    i16 -> I16 : gen_i16 for i16;
    i32 -> I32 : gen_i32 for i32;
    i64 -> I64 : gen_i64 for i64;
    i128 -> I128 : gen_i128 for i128;
    isize -> ISIZE : gen_isize for isize;
}

/// A mutator for `char` values.
///
/// See the [`char()`] function to create new instances and for example usage.
#[derive(Clone, Debug, Default)]
pub struct Char {
    _private: (),
}

/// Create a mutator for `char` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, MutationBuilder};
///
/// let mut mutator = m::char();
/// let mut mtn = MutationBuilder::new();
///
/// let mut c = 'a';
/// for _ in 0..5 {
///     mtn.mutate_with(&mut mutator, &mut c)?;
///     println!("mutated c is {c}");
/// }
///
/// // Example output:
/// //
/// //     mutated c is !
/// //     mutated c is ᐠ
/// //     mutated c is 𬸚
/// //     mutated c is 1
/// //     mutated c is 꼜
/// # Ok(())
/// # }
/// # foo().unwrap();
/// ```
pub fn char() -> Char {
    Char { _private: () }
}

impl Mutate<char> for Char {
    #[inline]
    fn mutate(&mut self, muts: &mut MutationSet, value: &mut char) -> crate::Result<()> {
        if muts.shrink() {
            if *value != '\0' {
                muts.mutation(|ctx| {
                    *value = ctx.rng().inner().gen_range('\0'..*value);
                    Ok(())
                })?;
            }
            Ok(())
        } else {
            // Choose between one of a few different mutation strategies to bias
            // the distribution towards interesting characters.
            //
            // See https://en.wikipedia.org/wiki/Plane_(Unicode)#Overview and
            // https://en.wikipedia.org/wiki/Unicode_block#List_of_blocks

            let c = |x| char::from_u32(x).unwrap_or_else(|| panic!("invalid char: {x:#x}"));

            // Non-control ASCII characters.
            range(c(0x20)..=c(0x7E)).mutate(muts, value)?;

            // Plane 0
            range(c(0x0000)..=c(0x0FFF)).mutate(muts, value)?;
            range(c(0x1000)..=c(0x1FFF)).mutate(muts, value)?;
            range(c(0x2000)..=c(0x2FFF)).mutate(muts, value)?;
            range(c(0x3000)..=c(0x3FFF)).mutate(muts, value)?;
            range(c(0x4000)..=c(0x4FFF)).mutate(muts, value)?;
            range(c(0x5000)..=c(0x5FFF)).mutate(muts, value)?;
            range(c(0x6000)..=c(0x6FFF)).mutate(muts, value)?;
            range(c(0x7000)..=c(0x7FFF)).mutate(muts, value)?;
            range(c(0x8000)..=c(0x8FFF)).mutate(muts, value)?;
            range(c(0x9000)..=c(0x9FFF)).mutate(muts, value)?;
            range(c(0xA000)..=c(0xAFFF)).mutate(muts, value)?;
            range(c(0xB000)..=c(0xBFFF)).mutate(muts, value)?;
            range(c(0xC000)..=c(0xCFFF)).mutate(muts, value)?;
            range(c(0xD000)..=c(0xD7FF)).mutate(muts, value)?;
            range(c(0xE000)..=c(0xEFFF)).mutate(muts, value)?;
            range(c(0xF000)..=c(0xFFFF)).mutate(muts, value)?;

            // Plane 1
            range(c(0x10000)..=c(0x10FFF)).mutate(muts, value)?;
            range(c(0x11000)..=c(0x11FFF)).mutate(muts, value)?;
            range(c(0x12000)..=c(0x12FFF)).mutate(muts, value)?;
            range(c(0x13000)..=c(0x13FFF)).mutate(muts, value)?;
            range(c(0x14000)..=c(0x14FFF)).mutate(muts, value)?;
            range(c(0x16000)..=c(0x16FFF)).mutate(muts, value)?;
            range(c(0x17000)..=c(0x17FFF)).mutate(muts, value)?;
            range(c(0x18000)..=c(0x18FFF)).mutate(muts, value)?;
            range(c(0x1A000)..=c(0x1AFFF)).mutate(muts, value)?;
            range(c(0x1B000)..=c(0x1BFFF)).mutate(muts, value)?;
            range(c(0x1C000)..=c(0x1CFFF)).mutate(muts, value)?;
            range(c(0x1D000)..=c(0x1DFFF)).mutate(muts, value)?;
            range(c(0x1E000)..=c(0x1EFFF)).mutate(muts, value)?;
            range(c(0x1F000)..=c(0x1FFFF)).mutate(muts, value)?;

            // Plane 2
            range(c(0x20000)..=c(0x20FFF)).mutate(muts, value)?;
            range(c(0x21000)..=c(0x21FFF)).mutate(muts, value)?;
            range(c(0x22000)..=c(0x22FFF)).mutate(muts, value)?;
            range(c(0x23000)..=c(0x23FFF)).mutate(muts, value)?;
            range(c(0x24000)..=c(0x24FFF)).mutate(muts, value)?;
            range(c(0x25000)..=c(0x25FFF)).mutate(muts, value)?;
            range(c(0x26000)..=c(0x26FFF)).mutate(muts, value)?;
            range(c(0x27000)..=c(0x27FFF)).mutate(muts, value)?;
            range(c(0x28000)..=c(0x28FFF)).mutate(muts, value)?;
            range(c(0x29000)..=c(0x29FFF)).mutate(muts, value)?;
            range(c(0x2A000)..=c(0x2AFFF)).mutate(muts, value)?;
            range(c(0x2B000)..=c(0x2BFFF)).mutate(muts, value)?;
            range(c(0x2C000)..=c(0x2CFFF)).mutate(muts, value)?;
            range(c(0x2D000)..=c(0x2DFFF)).mutate(muts, value)?;
            range(c(0x2E000)..=c(0x2EFFF)).mutate(muts, value)?;
            range(c(0x2F000)..=c(0x2FFFF)).mutate(muts, value)?;

            // Plane 3
            range(c(0x30000)..=c(0x30FFF)).mutate(muts, value)?;
            range(c(0x31000)..=c(0x31FFF)).mutate(muts, value)?;
            range(c(0x32000)..=c(0x32FFF)).mutate(muts, value)?;

            // Catch all: any valid character, regardless of its plane,
            // block, or if it has been assigned or not.
            muts.mutation(|ctx| Ok(*value = ctx.rng().inner().gen()))?;

            Ok(())
        }
    }
}

impl DefaultMutate for char {
    type DefaultMutate = Char;
}

impl Generate<char> for Char {
    #[inline]
    fn generate(&mut self, ctx: &mut MutationContext) -> crate::Result<char> {
        Ok(ctx.rng().inner().gen())
    }
}

impl MutateInRange<char> for Char {
    #[inline]
    fn mutate_in_range(
        &mut self,
        muts: &mut MutationSet,
        value: &mut char,
        range: &ops::RangeInclusive<char>,
    ) -> crate::Result<()> {
        let start = *range.start();
        let end = *range.end();

        if start > end {
            return Err(Error::invalid_range());
        }

        if *value == start && muts.shrink() {
            return Ok(());
        }

        muts.mutation(|ctx| {
            let end = if ctx.shrink() {
                core::cmp::min(*value, end)
            } else {
                end
            };
            *value = ctx.rng().inner().gen_range(start..=end);
            Ok(())
        })
    }
}

/// A mutator for `f32` values.
///
/// See the [`f32()`] function to create new instances and for example usage.
pub struct F32 {
    _private: (),
}

/// Create a mutator for `f32` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, MutationBuilder};
///
/// let mut mutator = m::f32();
/// let mut mtn = MutationBuilder::new();
///
/// let mut value = 3.14;
/// for _ in 0..5 {
///     mtn.mutate_with(&mut mutator, &mut value)?;
///     println!("mutated value is {value}");
/// }
///
/// // Example output:
/// //
/// //     mutated value is NaN
/// //     mutated value is -inf
/// //     mutated value is 0.00000011920929
/// //     mutated value is -260030670000000000000000000000000000000
/// //     mutated value is 57951606000000000000000000000000000000
/// # Ok(())
/// # }
/// # foo().unwrap();
/// ```
pub fn f32() -> F32 {
    F32 { _private: () }
}

impl Mutate<f32> for F32 {
    #[inline]
    fn mutate(&mut self, muts: &mut MutationSet, value: &mut f32) -> crate::Result<()> {
        let special_finite = |muts: &mut MutationSet, value: &mut f32| -> crate::Result<()> {
            muts.mutation(|_| Ok(*value = 0.0))?;
            muts.mutation(|_| Ok(*value = 1.0))?;
            muts.mutation(|_| Ok(*value = -1.0))?;
            muts.mutation(|_| Ok(*value = f32::EPSILON))?;
            muts.mutation(|_| Ok(*value = f32::MIN_POSITIVE))?;
            muts.mutation(|_| Ok(*value = f32::MAX))?;
            muts.mutation(|_| Ok(*value = f32::MIN))?;
            Ok(())
        };

        let finite = |muts: &mut MutationSet, value: &mut f32| -> crate::Result<()> {
            special_finite(muts, value)?;

            // Positives.
            muts.mutation(|ctx| Ok(*value = ctx.rng().inner().gen::<f32>() * f32::MAX))?;

            // Negatives.
            muts.mutation(|ctx| Ok(*value = ctx.rng().inner().gen::<f32>() * f32::MIN))?;

            Ok(())
        };

        if muts.shrink() {
            if *value == 0.0 {
                return Ok(());
            }
            if value.is_nan() || value.is_infinite() {
                return finite(muts, value);
            }
            muts.mutation(|ctx| Ok(*value *= ctx.rng().inner().gen::<f32>()))?;
            Ok(())
        } else {
            finite(muts, value)?;
            muts.mutation(|_| Ok(*value = f32::INFINITY))?;
            muts.mutation(|_| Ok(*value = f32::NEG_INFINITY))?;
            muts.mutation(|_| Ok(*value = f32::NAN))?;
            Ok(())
        }
    }
}

/// A mutator for `f64` values.
///
/// See the [`f64()`] function to create new instances and for example usage.
pub struct F64 {
    _private: (),
}

/// Create a mutator for `f64` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, MutationBuilder};
///
/// let mut mutator = m::f64();
/// let mut mtn = MutationBuilder::new();
///
/// let mut value = 3.14;
/// for _ in 0..5 {
///     mtn.mutate_with(&mut mutator, &mut value)?;
///     println!("mutated value is {value}");
/// }
///
/// // Example output:
/// //
/// //     mutated value is 0.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000022250738585072014
/// //     mutated value is 30615525916172793000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
/// //     mutated value is -inf
/// //     mutated value is -179769313486231570000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
/// //     mutated value is NaN
/// # Ok(())
/// # }
/// # foo().unwrap();
/// ```
pub fn f64() -> F64 {
    F64 { _private: () }
}

impl Mutate<f64> for F64 {
    #[inline]
    fn mutate(&mut self, muts: &mut MutationSet, value: &mut f64) -> crate::Result<()> {
        let special_finite = |muts: &mut MutationSet, value: &mut f64| -> crate::Result<()> {
            muts.mutation(|_| Ok(*value = 0.0))?;
            muts.mutation(|_| Ok(*value = 1.0))?;
            muts.mutation(|_| Ok(*value = -1.0))?;
            muts.mutation(|_| Ok(*value = f64::EPSILON))?;
            muts.mutation(|_| Ok(*value = f64::MIN_POSITIVE))?;
            muts.mutation(|_| Ok(*value = f64::MAX))?;
            muts.mutation(|_| Ok(*value = f64::MIN))?;
            Ok(())
        };

        let finite = |muts: &mut MutationSet, value: &mut f64| -> crate::Result<()> {
            special_finite(muts, value)?;

            // Positives.
            muts.mutation(|ctx| Ok(*value = ctx.rng().inner().gen::<f64>() * f64::MAX))?;

            // Negatives.
            muts.mutation(|ctx| Ok(*value = ctx.rng().inner().gen::<f64>() * f64::MIN))?;

            Ok(())
        };

        if muts.shrink() {
            if *value == 0.0 {
                return Ok(());
            }
            if value.is_nan() || value.is_infinite() {
                return finite(muts, value);
            }
            muts.mutation(|ctx| Ok(*value *= ctx.rng().inner().gen::<f64>()))?;
            Ok(())
        } else {
            finite(muts, value)?;
            muts.mutation(|_| Ok(*value = f64::INFINITY))?;
            muts.mutation(|_| Ok(*value = f64::NEG_INFINITY))?;
            muts.mutation(|_| Ok(*value = f64::NAN))?;
            Ok(())
        }
    }
}

// TODO: str

// TODO: slice

macro_rules! tuples {
    ( $( $fn_name:ident -> $ty_name:ident ( $( $m:ident : $t:ident , )* ) ; )* ) => {
        $(
            /// A mutator for tuples.
            #[derive(Clone, Debug, Default)]
            #[allow(non_snake_case)]
            pub struct $ty_name<$( $m , )*> {
                $(
                    $m: $m,
                )*
            }

            /// Create a new mutator for a tuple of
            #[doc = stringify!(tuples!(@count $( $m )*))]
            /// elements.
            ///
            /// # Example
            ///
            /// ```
            /// # fn _foo() -> mutatis::Result<()> {
            /// use mutatis::{mutators as m, Mutate, MutationBuilder};
            ///
            /// let mut mutator = m::tuple2(m::u8(), m::i16());
            /// let mut mtn = MutationBuilder::new();
            ///
            /// let mut value = (42, -1234);
            /// mtn.mutate_with(&mut mutator, &mut value)?;
            ///
            /// println!("mutated value is {value:?}");
            /// # Ok(())
            /// # }
            /// ```
            #[allow(non_snake_case)]
            pub fn $fn_name< $( $m ),* >( $( $m: $m ),* ) -> $ty_name<$( $m , )*> {
                $ty_name {
                    $(
                        $m,
                    )*
                }
            }

            #[allow(non_snake_case)]
            impl< $( $m , $t, )* > Mutate<( $( $t , )* )> for $ty_name<$( $m , )*>
            where
                $(
                    $m: Mutate<$t>,
                )*
            {
                #[inline]
                fn mutate(
                    &mut self,
                    _muts: &mut MutationSet,
                    ( $( $t , )* ): &mut ( $( $t , )* ),
                ) -> crate::Result<()> {
                    $(
                        self.$m.mutate(_muts, $t)?;
                    )*
                    Ok(())
                }
            }

            #[allow(non_snake_case)]
            impl< $( $t , )* > DefaultMutate for ( $( $t , )* )
            where
                $(
                    $t: DefaultMutate,
                )*
            {
                type DefaultMutate = $ty_name<$( $t::DefaultMutate , )*>;
            }
        )*
    };

    (@count) => { 0 };
    (@count $head:ident $( $rest:ident )*) => { 1 + tuples!(@count $( $rest )*) };
}

tuples! {
    tuple1 -> Tuple1(M0: T0,);
    tuple2 -> Tuple2(M0: T0, M1: T1,);
    tuple3 -> Tuple3(M0: T0, M1: T1, M2: T2,);
    tuple4 -> Tuple4(M0: T0, M1: T1, M2: T2, M3: T3,);
    tuple5 -> Tuple5(M0: T0, M1: T1, M2: T2, M3: T3, M4: T4,);
    tuple6 -> Tuple6(M0: T0, M1: T1, M2: T2, M3: T3, M4: T4, M5: T5,);
    tuple7 -> Tuple7(M0: T0, M1: T1, M2: T2, M3: T3, M4: T4, M5: T5, M6: T6,);
    tuple8 -> Tuple8(M0: T0, M1: T1, M2: T2, M3: T3, M4: T4, M5: T5, M6: T6, M7: T7,);
    tuple9 -> Tuple9(M0: T0, M1: T1, M2: T2, M3: T3, M4: T4, M5: T5, M6: T6, M7: T7, M8: T8,);
    tuple10 -> Tuple10(M0: T0, M1: T1, M2: T2, M3: T3, M4: T4, M5: T5, M6: T6, M7: T7, M8: T8, M9: T9,);
    tuple11 -> Tuple11(M0: T0, M1: T1, M2: T2, M3: T3, M4: T4, M5: T5, M6: T6, M7: T7, M8: T8, M9: T9, M10: T10,);
    tuple12 -> Tuple12(M0: T0, M1: T1, M2: T2, M3: T3, M4: T4, M5: T5, M6: T6, M7: T7, M8: T8, M9: T9, M10: T10, M11: T11,);
    tuple13 -> Tuple13(M0: T0, M1: T1, M2: T2, M3: T3, M4: T4, M5: T5, M6: T6, M7: T7, M8: T8, M9: T9, M10: T10, M11: T11, M12: T12,);
    tuple14 -> Tuple14(M0: T0, M1: T1, M2: T2, M3: T3, M4: T4, M5: T5, M6: T6, M7: T7, M8: T8, M9: T9, M10: T10, M11: T11, M12: T12, M13: T13,);
    tuple15 -> Tuple15(M0: T0, M1: T1, M2: T2, M3: T3, M4: T4, M5: T5, M6: T6, M7: T7, M8: T8, M9: T9, M10: T10, M11: T11, M12: T12, M13: T13, M14: T14,);
    tuple16 -> Tuple16(M0: T0, M1: T1, M2: T2, M3: T3, M4: T4, M5: T5, M6: T6, M7: T7, M8: T8, M9: T9, M10: T10, M11: T11, M12: T12, M13: T13, M14: T14, M15: T15,);
}

/// A unit mutator.
#[derive(Clone, Debug, Default)]
pub struct Unit {
    _private: (),
}

/// Create a new unit (a.k.a zero-element tuple) mutator.
///
/// # Example
///
/// ```
/// use mutatis::{mutators as m, Error, Mutate, MutationBuilder};
///
/// let mut mutator = m::unit();
/// let mut mtn = MutationBuilder::new();
///
/// let mut value = ();
/// let err = mtn.mutate_with(&mut mutator, &mut value).unwrap_err();
///
/// // Because there is only one possible value for the unit type, the mutator
/// // is always exhausted.
/// assert!(err.is_exhausted());
/// ```
pub fn unit() -> Unit {
    Unit { _private: () }
}

impl Mutate<()> for Unit {
    #[inline]
    fn mutate(&mut self, _muts: &mut MutationSet, _value: &mut ()) -> crate::Result<()> {
        Ok(())
    }
}

/// A mutator for fixed-size arrays.
///
/// See the [`array()`] function to create a new `Array` mutator and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct Array<const N: usize, M> {
    mutator: M,
}

/// Create a new `Array` mutator.
///
/// # Example
///
/// ```
/// use mutatis::{mutators as m, Mutate, MutationBuilder};
///
/// let mut mutator = m::array(m::u8());
/// let mut mtn = MutationBuilder::new();
///
/// let mut value = [1, 2, 3, 4];
/// mtn.mutate_with(&mut mutator, &mut value).unwrap();
///
/// println!("mutated array is {value:?}");
/// ```
pub fn array<const N: usize, M>(mutator: M) -> Array<N, M> {
    Array { mutator }
}

impl<const N: usize, M, T> Mutate<[T; N]> for Array<N, M>
where
    M: Mutate<T>,
{
    #[inline]
    fn mutate(&mut self, muts: &mut MutationSet, value: &mut [T; N]) -> crate::Result<()> {
        for element in value.iter_mut() {
            self.mutator.mutate(muts, element)?;
        }
        Ok(())
    }
}

impl<const N: usize, T> DefaultMutate for [T; N]
where
    T: DefaultMutate,
{
    type DefaultMutate = Array<N, T::DefaultMutate>;
}

/// A mutator for `Option<T>`.
///
/// See the [`option`] function to create a new `Option` mutator and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct Option<M> {
    mutator: M,
}

/// Create a new `Option` mutator.
///
/// # Example
///
/// ```
/// use mutatis::{mutators as m, Mutate, MutationBuilder};
///
/// let mut mutator = m::option(m::u32());
/// let mut mtn = MutationBuilder::new();
///
/// let mut value = Some(36);
/// mtn.mutate_with(&mut mutator, &mut value).unwrap();
///
/// println!("mutated option is {value:?}");
/// ```
pub fn option<M>(mutator: M) -> Option<M> {
    Option { mutator }
}

impl<M, T> Mutate<core::option::Option<T>> for Option<M>
where
    M: Generate<T>,
{
    #[inline]
    fn mutate(
        &mut self,
        muts: &mut MutationSet,
        value: &mut core::option::Option<T>,
    ) -> crate::Result<()> {
        if muts.shrink() && value.is_none() {
            return Ok(());
        }

        match value.as_mut() {
            None => muts.mutation(|ctx| Ok(*value = Some(self.mutator.generate(ctx)?))),
            Some(v) => {
                self.mutator.mutate(muts, v)?;
                muts.mutation(|_| Ok(*value = None))
            }
        }
    }
}

impl<T> DefaultMutate for core::option::Option<T>
where
    T: DefaultMutate,
    T::DefaultMutate: Generate<T>,
{
    type DefaultMutate = Option<T::DefaultMutate>;
}

/// A mutator for `Result<T, E>`.
///
/// See the [`result`] function for creating new `Result` mutators and for
/// example usage.
#[derive(Clone, Debug, Default)]
pub struct Result<M, N> {
    ok_mutator: M,
    err_mutator: N,
}

/// Create a new `Result` mutator.
///
/// # Example
///
/// ```
/// use mutatis::{mutators as m, Mutate, MutationBuilder};
///
/// let mut mutator = m::result(m::u32(), m::i8());
/// let mut mtn = MutationBuilder::new();
///
/// let mut value = Ok(1312);
/// mtn.mutate_with(&mut mutator, &mut value).unwrap();
///
/// println!("mutated result is {value:?}");
/// ```
pub fn result<M, N>(ok_mutator: M, err_mutator: N) -> Result<M, N> {
    Result {
        ok_mutator,
        err_mutator,
    }
}

impl<M, N, T, E> Mutate<core::result::Result<T, E>> for Result<M, N>
where
    M: Generate<T>,
    N: Generate<E>,
{
    #[inline]
    fn mutate(
        &mut self,
        muts: &mut MutationSet,
        value: &mut core::result::Result<T, E>,
    ) -> crate::Result<()> {
        match value {
            Ok(x) => {
                self.ok_mutator.mutate(muts, x)?;
                if !muts.shrink() {
                    muts.mutation(|ctx| Ok(*value = Err(self.err_mutator.generate(ctx)?)))?;
                }
            }
            Err(e) => {
                self.err_mutator.mutate(muts, e)?;
                muts.mutation(|ctx| Ok(*value = Ok(self.ok_mutator.generate(ctx)?)))?;
            }
        }
        Ok(())
    }
}

impl<T, E> DefaultMutate for core::result::Result<T, E>
where
    T: DefaultMutate,
    T::DefaultMutate: Generate<T>,
    E: DefaultMutate,
    E::DefaultMutate: Generate<E>,
{
    type DefaultMutate = Result<T::DefaultMutate, E::DefaultMutate>;
}

/// A mutator for `T` values within a given range.
///
/// See the [`range`] function to create new `Range` mutator instances and for
/// example usage.
#[derive(Clone, Debug)]
pub struct Range<M, T> {
    mutator: M,
    range: ops::RangeInclusive<T>,
}

/// Create a new mutator for `T` values, keeping them within the given range.
///
/// # Example
///
/// ```
/// use mutatis::{mutators as m, Mutate, MutationBuilder};
///
/// let mut mutator = m::range(111..=666);
/// let mut mtn = MutationBuilder::new();
///
/// let mut value = 123;
/// mtn.mutate_with(&mut mutator, &mut value).unwrap();
///
/// assert!(value >= 111);
/// assert!(value <= 666);
/// ```
pub fn range<T>(range: ops::RangeInclusive<T>) -> Range<T::DefaultMutate, T>
where
    T: DefaultMutate,
{
    let mutator = default::<T>();
    Range { mutator, range }
}

/// Like [`range`] but uses the given `mutator` rather than the `T`'s default
/// mutator.
pub fn range_with<M, T>(range: ops::RangeInclusive<T>, mutator: M) -> Range<M, T> {
    Range { mutator, range }
}

impl<M, T> Mutate<T> for Range<M, T>
where
    M: MutateInRange<T>,
{
    #[inline]
    fn mutate(&mut self, muts: &mut MutationSet, value: &mut T) -> crate::Result<()> {
        self.mutator.mutate_in_range(muts, value, &self.range)
    }
}

impl<M, T> Generate<T> for Range<M, T>
where
    M: Generate<T> + MutateInRange<T>,
{
    #[inline]
    fn generate(&mut self, context: &mut MutationContext) -> crate::Result<T> {
        let mut value = self.mutator.generate(context)?;
        context.mutate_in_range_with(&mut self.mutator, &mut value, &self.range)?;
        Ok(value)
    }
}

// TODO: cell, refcell

// TODO: duration
