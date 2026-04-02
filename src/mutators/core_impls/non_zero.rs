use super::*;

macro_rules! non_zero {
    (
        $(
            $fn_name:ident -> $ty_name:ident : $inner_ty:ty, $inner_mutator:ident ;
        )*
    ) => {
        $(
            /// A mutator for
            #[doc = concat!("`NonZero<", stringify!($inner_ty), ">`")]
            /// values.
            ///
            /// See the
            #[doc = concat!("[`", stringify!($fn_name), "()`]")]
            /// function to create new instances and for example usage.
            #[derive(Clone, Debug, Default)]
            pub struct $ty_name {
                inner: $inner_mutator,
            }

            /// Create a new
            #[doc = concat!("`NonZero<", stringify!($inner_ty), ">`")]
            /// mutator.
            ///
            /// # Example
            ///
            /// ```
            /// # fn foo() -> mutatis::Result<()> {
            /// use mutatis::{mutators as m, Mutate, Session};
            /// use core::num::NonZero;
            ///
            #[doc = concat!("let mut mutator = m::", stringify!($fn_name), "();")]
            /// let mut session = Session::new();
            ///
            #[doc = concat!("let mut value = NonZero::<", stringify!($inner_ty), ">::new(42).unwrap();")]
            /// for _ in 0..5 {
            ///     session.mutate_with(&mut mutator, &mut value)?;
            ///     println!("value = {value}");
            /// }
            ///
            /// // Example output:
            /// //
            /// //     value = 190
            /// //     value = 101
            /// //     value = 49
            /// //     value = 160
            /// //     value = 99
            /// # Ok(())
            /// # }
            /// # foo().unwrap();
            /// ```
            pub fn $fn_name() -> $ty_name {
                $ty_name {
                    inner: $inner_mutator { _private: () },
                }
            }

            impl Mutate<core::num::NonZero<$inner_ty>> for $ty_name {
                #[inline]
                fn mutate(
                    &mut self,
                    c: &mut Candidates,
                    value: &mut core::num::NonZero<$inner_ty>,
                ) -> Result<()> {
                    c.mutation(|ctx| {
                        let v = self.inner.generate(ctx)?.max(1);
                        *value = core::num::NonZero::new(v).unwrap();
                        Ok(())
                    })?;
                    Ok(())
                }
            }

            impl Generate<core::num::NonZero<$inner_ty>> for $ty_name {
                #[inline]
                fn generate(
                    &mut self,
                    ctx: &mut Context,
                ) -> Result<core::num::NonZero<$inner_ty>> {
                    let val = self.inner.generate(ctx)?.max(1);
                    Ok(core::num::NonZero::new(val).unwrap())
                }
            }

            impl DefaultMutate for core::num::NonZero<$inner_ty> {
                type DefaultMutate = $ty_name;
            }
        )*
    };
}

non_zero! {
    non_zero_u8 -> NonZeroU8 : u8, U8;
    non_zero_u16 -> NonZeroU16 : u16, U16;
    non_zero_u32 -> NonZeroU32 : u32, U32;
    non_zero_u64 -> NonZeroU64 : u64, U64;
    non_zero_u128 -> NonZeroU128 : u128, U128;
    non_zero_usize -> NonZeroUsize : usize, Usize;
    non_zero_i8 -> NonZeroI8 : i8, I8;
    non_zero_i16 -> NonZeroI16 : i16, I16;
    non_zero_i32 -> NonZeroI32 : i32, I32;
    non_zero_i64 -> NonZeroI64 : i64, I64;
    non_zero_i128 -> NonZeroI128 : i128, I128;
    non_zero_isize -> NonZeroIsize : isize, Isize;
}
