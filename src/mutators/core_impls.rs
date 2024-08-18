use super::*;
use crate::Result;
use core::{cmp, ops};

mod option;
mod result;
pub use option::*;
pub use result::*;

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
/// use mutatis::{mutators as m, Mutate, Session};
///
/// let mut mutator = m::bool();
/// let mut session = Session::new();
///
/// let mut value = true;
/// session.mutate_with(&mut mutator, &mut value).unwrap();
///
/// assert_eq!(value, false);
/// ```
pub fn bool() -> Bool {
    Bool { _private: () }
}

impl Mutate<bool> for Bool {
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut bool) -> Result<()> {
        if !c.shrink() || *value {
            c.mutation(|_ctx| Ok(*value = !*value))?;
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
            #[doc = concat!("`", stringify!($fn_name), "`")]
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
            /// use mutatis::{mutators as m, Mutate, Session};
            ///
            #[doc = concat!("let mut mutator = m::", stringify!($fn_name), "();")]
            ///
            /// let mut session = Session::new().shrink(true);
            ///
            /// let mut value = 42;
            /// session.mutate_with(&mut mutator, &mut value).unwrap();
            ///
            /// assert!(value < 42);
            /// ```
            pub fn $fn_name() -> $ty_name {
                $ty_name { _private: () }
            }

            impl Mutate<$ty> for $ty_name {
                #[inline]
                fn mutate(&mut self, c: &mut Candidates, value: &mut $ty) -> Result<()> {
                    if c.shrink() && *value == 0 {
                        return Ok(());
                    }
                    c.mutation(|ctx| {
                        *value = if ctx.shrink() {
                            ctx.rng().inner().gen_range(0..*value)
                        } else {
                            ctx.rng().$method()
                        };
                        Ok(())
                    })
                }
            }

            impl DefaultMutate for $ty {
                type DefaultMutate = $ty_name;
            }

            impl Generate<$ty> for $ty_name {
                #[inline]
                fn generate(&mut self, ctx: &mut Context) -> Result<$ty> {
                    Ok(ctx.rng().$method())
                }
            }

            impl MutateInRange<$ty> for $ty_name {
                #[inline]
                fn mutate_in_range(
                    &mut self,
                    c: &mut Candidates,
                    value: &mut $ty,
                    range: &ops::RangeInclusive<$ty>,
                ) -> Result<()> {
                    let start = *range.start();
                    let end = *range.end();

                    if start > end {
                        return Err(Error::invalid_range());
                    }

                    if *value == start && c.shrink() {
                        return Ok(());
                    }

                    c.mutation(|ctx| {
                        let end = if ctx.shrink() {
                            cmp::min(*value, end)
                        } else {
                            end
                        };

                        *value = ctx.rng().inner().gen_range(start..=end);
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
    usize -> Usize : gen_usize for usize;
    i8 -> I8 : gen_i8 for i8;
    i16 -> I16 : gen_i16 for i16;
    i32 -> I32 : gen_i32 for i32;
    i64 -> I64 : gen_i64 for i64;
    i128 -> I128 : gen_i128 for i128;
    isize -> Isize : gen_isize for isize;
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
/// use mutatis::{mutators as m, Mutate, Session};
///
/// let mut mutator = m::char();
/// let mut session = Session::new();
///
/// let mut c = 'a';
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut c)?;
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
    fn mutate(&mut self, c: &mut Candidates, value: &mut char) -> Result<()> {
        if c.shrink() {
            if *value != '\0' {
                c.mutation(|ctx| {
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
            //
            // Note that the ranges below still contain some unassigned
            // characters. This is fine. Explicitly listing only assigned
            // characters is too much work, and this is just a best effort kind
            // of thing.

            let ch = |x| char::from_u32(x).unwrap_or_else(|| panic!("invalid char: {x:#x}"));
            let mut char_range = |start, end| range(ch(start)..=ch(end)).mutate(c, value);

            // Non-control ASCII characters.
            char_range(0x20, 0x7E)?;

            // Plane 0
            char_range(0x0000, 0xFFFF)?;

            // Plane 1
            char_range(0x10000, 0x14FFF)?;
            // Unassigned: 0x15000..=15FFF.
            char_range(0x16000, 0x18FFF)?;
            // Unassigned: 0x19000..=1AFFF.
            char_range(0x1A000, 0x1FFFF)?;

            // Plane 2
            char_range(0x20000, 0x2FFFF)?;

            // Plane 3
            char_range(0x30000, 0x32FFF)?;

            // Catch all: any valid character, regardless of its plane, block,
            // or if it has been assigned or not.
            c.mutation(|ctx| Ok(*value = ctx.rng().inner().gen()))?;

            Ok(())
        }
    }
}

impl DefaultMutate for char {
    type DefaultMutate = Char;
}

impl Generate<char> for Char {
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<char> {
        Ok(ctx.rng().inner().gen())
    }
}

impl MutateInRange<char> for Char {
    #[inline]
    fn mutate_in_range(
        &mut self,
        c: &mut Candidates,
        value: &mut char,
        range: &ops::RangeInclusive<char>,
    ) -> Result<()> {
        let start = *range.start();
        let end = *range.end();

        if start > end {
            return Err(Error::invalid_range());
        }

        if *value == start && c.shrink() {
            return Ok(());
        }

        c.mutation(|ctx| {
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
/// use mutatis::{mutators as m, Mutate, Session};
///
/// let mut mutator = m::f32();
/// let mut session = Session::new();
///
/// let mut value = 3.14;
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
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
    fn mutate(&mut self, c: &mut Candidates, value: &mut f32) -> Result<()> {
        let special_finite = |c: &mut Candidates, value: &mut f32| -> Result<()> {
            c.mutation(|_| Ok(*value = 0.0))?;
            c.mutation(|_| Ok(*value = 1.0))?;
            c.mutation(|_| Ok(*value = -1.0))?;
            c.mutation(|_| Ok(*value = f32::EPSILON))?;
            c.mutation(|_| Ok(*value = f32::MIN_POSITIVE))?;
            c.mutation(|_| Ok(*value = f32::MAX))?;
            c.mutation(|_| Ok(*value = f32::MIN))?;
            Ok(())
        };

        let finite = |c: &mut Candidates, value: &mut f32| -> Result<()> {
            special_finite(c, value)?;

            // Positives.
            c.mutation(|ctx| Ok(*value = ctx.rng().inner().gen::<f32>() * f32::MAX))?;

            // Negatives.
            c.mutation(|ctx| Ok(*value = ctx.rng().inner().gen::<f32>() * f32::MIN))?;

            Ok(())
        };

        if c.shrink() {
            if *value == 0.0 {
                return Ok(());
            }
            if value.is_nan() || value.is_infinite() {
                return finite(c, value);
            }
            c.mutation(|ctx| Ok(*value *= ctx.rng().inner().gen::<f32>()))?;
            Ok(())
        } else {
            finite(c, value)?;
            c.mutation(|_| Ok(*value = f32::INFINITY))?;
            c.mutation(|_| Ok(*value = f32::NEG_INFINITY))?;
            c.mutation(|_| Ok(*value = f32::NAN))?;
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
/// use mutatis::{mutators as m, Mutate, Session};
///
/// let mut mutator = m::f64();
/// let mut session = Session::new();
///
/// let mut value = 3.14;
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
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
    fn mutate(&mut self, c: &mut Candidates, value: &mut f64) -> Result<()> {
        let special_finite = |c: &mut Candidates, value: &mut f64| -> Result<()> {
            c.mutation(|_| Ok(*value = 0.0))?;
            c.mutation(|_| Ok(*value = 1.0))?;
            c.mutation(|_| Ok(*value = -1.0))?;
            c.mutation(|_| Ok(*value = f64::EPSILON))?;
            c.mutation(|_| Ok(*value = f64::MIN_POSITIVE))?;
            c.mutation(|_| Ok(*value = f64::MAX))?;
            c.mutation(|_| Ok(*value = f64::MIN))?;
            Ok(())
        };

        let finite = |c: &mut Candidates, value: &mut f64| -> Result<()> {
            special_finite(c, value)?;

            // Positives.
            c.mutation(|ctx| Ok(*value = ctx.rng().inner().gen::<f64>() * f64::MAX))?;

            // Negatives.
            c.mutation(|ctx| Ok(*value = ctx.rng().inner().gen::<f64>() * f64::MIN))?;

            Ok(())
        };

        if c.shrink() {
            if *value == 0.0 {
                return Ok(());
            }
            if value.is_nan() || value.is_infinite() {
                return finite(c, value);
            }
            c.mutation(|ctx| Ok(*value *= ctx.rng().inner().gen::<f64>()))?;
            Ok(())
        } else {
            finite(c, value)?;
            c.mutation(|_| Ok(*value = f64::INFINITY))?;
            c.mutation(|_| Ok(*value = f64::NEG_INFINITY))?;
            c.mutation(|_| Ok(*value = f64::NAN))?;
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
            /// use mutatis::{mutators as m, Mutate, Session};
            ///
            /// let mut mutator = m::tuple2(m::u8(), m::i16());
            /// let mut session = Session::new();
            ///
            /// let mut value = (42, -1234);
            /// session.mutate_with(&mut mutator, &mut value)?;
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
                    _c: &mut Candidates,
                    ( $( $t , )* ): &mut ( $( $t , )* ),
                ) -> Result<()> {
                    $(
                        self.$m.mutate(_c, $t)?;
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
/// use mutatis::{mutators as m, Error, Mutate, Session};
///
/// let mut mutator = m::unit();
/// let mut session = Session::new();
///
/// let mut value = ();
/// let err = session.mutate_with(&mut mutator, &mut value).unwrap_err();
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
    fn mutate(&mut self, _c: &mut Candidates, _value: &mut ()) -> Result<()> {
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
/// use mutatis::{mutators as m, Mutate, Session};
///
/// let mut mutator = m::array(m::u8());
/// let mut session = Session::new();
///
/// let mut value = [1, 2, 3, 4];
/// session.mutate_with(&mut mutator, &mut value).unwrap();
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
    fn mutate(&mut self, c: &mut Candidates, value: &mut [T; N]) -> Result<()> {
        for element in value.iter_mut() {
            self.mutator.mutate(c, element)?;
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

// TODO: cell, refcell

// TODO: duration
