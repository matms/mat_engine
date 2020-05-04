//! Abstracts over the slotmap library in case I need to change libraries in the future

pub type ArenaKey = ::slotmap::DefaultKey;

/// Simple wrapper around `::slotmap::DenseSlotMap`
/// See https://docs.rs/slotmap/0.4.0/slotmap/dense/struct.DenseSlotMap.html
pub struct Arena<T> {
    // We use DenseSlotMap (for now) because of limitations regarding valid types for SlotMap
    // See https://docs.rs/slotmap/0.4.0/slotmap/trait.Slottable.html
    slotmap: ::slotmap::DenseSlotMap<ArenaKey, T>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self {
            slotmap: ::slotmap::DenseSlotMap::<ArenaKey, T>::new(),
        }
    }

    pub fn insert(&mut self, val: T) -> ArenaKey {
        self.slotmap.insert(val)
    }

    pub fn remove(&mut self, key: ArenaKey) -> Option<T> {
        self.slotmap.remove(key)
    }

    pub fn get(&self, key: ArenaKey) -> Option<&T> {
        self.slotmap.get(key)
    }

    pub fn get_mut(&mut self, key: ArenaKey) -> Option<&mut T> {
        self.slotmap.get_mut(key)
    }

    pub fn has_key(&self, key: ArenaKey) -> bool {
        self.slotmap.contains_key(key)
    }
}
