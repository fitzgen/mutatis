use super::*;
use core::mem;

/// The default mutator for `BTreeMap<K, V>` values.
///
/// See the [`btree_map()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct BTreeMap<KM, VM> {
    key_mutator: KM,
    value_mutator: VM,
}

/// Create a new mutator for `BTreeMap<K, V>` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use std::collections::BTreeMap;
///
/// let mut items: BTreeMap<u8, u32> = BTreeMap::new();
///
/// let mut mutator = m::btree_map(m::u8(), m::mrange(100..=199));
///
/// let mut session = Session::new();
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut items)?;
///     println!("items = {items:?}");
/// }
///
/// // Example output:
/// //
/// //     items = {201: 146}
/// //     items = {201: 194}
/// //     items = {}
/// //     items = {44: 197}
/// //     items = {44: 197, 172: 123}
/// # Ok(()) }
/// # foo().unwrap();
/// ```
pub fn btree_map<KM, VM>(key_mutator: KM, value_mutator: VM) -> BTreeMap<KM, VM> {
    BTreeMap {
        key_mutator,
        value_mutator,
    }
}

impl<KM, VM, K, V> Mutate<alloc::collections::BTreeMap<K, V>> for BTreeMap<KM, VM>
where
    KM: Generate<K>,
    VM: Generate<V> + Mutate<V>,
    K: Ord,
{
    #[inline]
    fn mutate(
        &mut self,
        c: &mut Candidates,
        value: &mut alloc::collections::BTreeMap<K, V>,
    ) -> Result<()> {
        // Add an entry.
        if !c.shrink() {
            c.mutation(|ctx| {
                let k = self.key_mutator.generate(ctx)?;
                let v = self.value_mutator.generate(ctx)?;
                value.insert(k, v);
                Ok(())
            })?;
        }

        // Remove an entry.
        if !value.is_empty() {
            c.mutation(|ctx| {
                if ctx.rng().gen_bool() {
                    value.pop_first();
                } else {
                    value.pop_last();
                }
                Ok(())
            })?;
        }

        // Mutate an existing value.
        for (_k, v) in value.iter_mut() {
            self.value_mutator.mutate(c, v)?;
        }

        // Mutate an existing key.
        if !value.is_empty() {
            let mut early_exit = None;
            for (mut key, val) in mem::take(value) {
                if early_exit.is_none() {
                    match self.key_mutator.mutate(c, &mut key) {
                        Ok(()) => {}
                        Err(e) if e.is_early_exit() => {
                            early_exit = Some(e);
                        }
                        Err(e) => return Err(e),
                    }
                }

                value.insert(key, val);
            }

            if let Some(e) = early_exit {
                return Err(e);
            }
        }

        Ok(())
    }
}

impl<KM, VM, K, V> Generate<alloc::collections::BTreeMap<K, V>> for BTreeMap<KM, VM>
where
    KM: Generate<K>,
    VM: Generate<V> + Mutate<V>,
    K: Ord,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<alloc::collections::BTreeMap<K, V>> {
        self.generate_via_mutate(ctx, 1)
    }
}

impl<K, V> DefaultMutate for alloc::collections::BTreeMap<K, V>
where
    K: DefaultMutate + Ord,
    K::DefaultMutate: Generate<K>,
    V: DefaultMutate,
    V::DefaultMutate: Generate<V>,
{
    type DefaultMutate = BTreeMap<K::DefaultMutate, V::DefaultMutate>;
}
