//! Abstracts over the slotmap library in case I need to change libraries in the future

#[derive(Copy, Clone, Debug, Default)]
pub struct ArenaKey(::slotmap::DefaultKey);

/*pub type ArenaKey = ::slotmap::DefaultKey;*/

/// Simple wrapper around `::slotmap::DenseSlotMap`
/// See https://docs.rs/slotmap/0.4.0/slotmap/dense/struct.DenseSlotMap.html
pub struct Arena<T> {
    // We use DenseSlotMap (for now) because of limitations regarding valid types for SlotMap
    // See https://docs.rs/slotmap/0.4.0/slotmap/trait.Slottable.html
    slotmap: ::slotmap::DenseSlotMap<::slotmap::DefaultKey, T>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self {
            slotmap: ::slotmap::DenseSlotMap::<::slotmap::DefaultKey, T>::new(),
        }
    }

    pub fn insert(&mut self, val: T) -> ArenaKey {
        ArenaKey(self.slotmap.insert(val))
    }

    pub fn remove(&mut self, key: ArenaKey) -> Option<T> {
        self.slotmap.remove(key.0)
    }

    pub fn get(&self, key: ArenaKey) -> Option<&T> {
        self.slotmap.get(key.0)
    }

    pub fn get_mut(&mut self, key: ArenaKey) -> Option<&mut T> {
        self.slotmap.get_mut(key.0)
    }

    /// Panics if the key isn't found (that is, if get() returns None).
    pub fn get_unwrap(&self, key: ArenaKey) -> &T {
        self.get(key)
            .expect("Called Arena::get_unwrap() with key but key not in arena.")
    }

    /// Panics if the key isn't found (that is, if get_mut() returns None).
    pub fn get_mut_unwrap(&mut self, key: ArenaKey) -> &mut T {
        self.get_mut(key)
            .expect("Called Arena::get_mut_unwrap() with key but key not in arena.")
    }

    pub fn has_key(&self, key: ArenaKey) -> bool {
        self.slotmap.contains_key(key.0)
    }
}
