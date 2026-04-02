use super::*;
use std::mem;

/// The default mutator for `HashMap<K, V>` values.
///
/// See the [`hash_map()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct HashMap<KM, VM> {
    key_mutator: KM,
    value_mutator: VM,
}

/// Create a new mutator for `HashMap<K, V>` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use std::collections::HashMap;
///
/// let mut items: HashMap<u8, u32> = HashMap::new();
///
/// let mut mutator = m::hash_map(m::u8(), m::mrange(100..=199));
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
/// //     items = {10: 124}
/// //     items = {10: 124, 172: 123}
/// # Ok(()) }
/// # foo().unwrap();
/// ```
pub fn hash_map<KM, VM>(key_mutator: KM, value_mutator: VM) -> HashMap<KM, VM> {
    HashMap {
        key_mutator,
        value_mutator,
    }
}

impl<KM, VM, K, V> Mutate<std::collections::HashMap<K, V>> for HashMap<KM, VM>
where
    KM: Generate<K>,
    VM: Generate<V> + Mutate<V>,
    K: Eq + Hash,
{
    #[inline]
    fn mutate(
        &mut self,
        c: &mut Candidates,
        value: &mut std::collections::HashMap<K, V>,
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
                let target = ctx.rng().gen_index(value.len()).unwrap();
                let mut i = 0;
                value.retain(|_, _| {
                    let keep = i != target;
                    i += 1;
                    keep
                });
                Ok(())
            })?;
        }

        // Mutate an existing value.
        for (_k, v) in value.iter_mut() {
            self.value_mutator.mutate(c, v)?;
        }

        // Mutate an existing key.
        if !value.is_empty() {
            let elems = mem::take(value).into_iter();
            value.reserve(elems.len());

            let mut early_exit = None;
            for (mut key, val) in elems {
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

impl<KM, VM, K, V> Generate<std::collections::HashMap<K, V>> for HashMap<KM, VM>
where
    KM: Generate<K>,
    VM: Generate<V> + Mutate<V>,
    K: Eq + Hash,
{
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<std::collections::HashMap<K, V>> {
        self.generate_via_mutate(ctx, 1)
    }
}

impl<K, V> DefaultMutate for std::collections::HashMap<K, V>
where
    K: DefaultMutate + Eq + Hash,
    K::DefaultMutate: Generate<K>,
    V: DefaultMutate,
    V::DefaultMutate: Generate<V>,
{
    type DefaultMutate = HashMap<K::DefaultMutate, V::DefaultMutate>;
}
