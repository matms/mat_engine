//! Abstracts over the slotmap library in case I need to change libraries in the future.

/// Key that indexes into `Arena`.
/// The arena is generational, so the key stores a generation.
///
/// Use of a key from one arena to index into another* is not supported, and may cause serious issues.
///
/// *(Except, of course, for secondary arenas, if they have been implemented).
#[derive(Copy, Clone, Debug, Default)]
pub struct ArenaKey(::slotmap::DefaultKey);

/*pub type ArenaKey = ::slotmap::DefaultKey;*/

/// Generational Arena
///
/// Uses `ArenaKey` for indexing into it.
///
/// IMPLEMENTATION: Simple wrapper around `::slotmap::DenseSlotMap`.
/// See https://docs.rs/slotmap/0.4.0/slotmap/dense/struct.DenseSlotMap.html.
///
/// EXTRA INFORMATION: We use `DenseSlotMap` (for now) because of current limitations
/// regarding valid types for SlotMap.
/// See https://docs.rs/slotmap/0.4.0/slotmap/trait.Slottable.html.
#[derive(Clone, Debug)]
pub struct Arena<T> {
    slotmap: ::slotmap::DenseSlotMap<::slotmap::DefaultKey, T>,
}

impl<T> Arena<T> {
    /// Returns a new, empty `Arena`.
    pub fn new() -> Self {
        Self {
            slotmap: ::slotmap::DenseSlotMap::<::slotmap::DefaultKey, T>::new(),
        }
    }

    /// Inserts `val`, returns the corresponding `ArenaKey`.
    pub fn insert(&mut self, val: T) -> ArenaKey {
        ArenaKey(self.slotmap.insert(val))
    }

    /// Removes the value at `key`, returning `Some(T)` if it exists and `None` if there was no
    /// value for the key.
    pub fn remove(&mut self, key: ArenaKey) -> Option<T> {
        self.slotmap.remove(key.0)
    }

    /// Gets the value at `key`, returning `Some(&T)` if it exists and `None` if it doesn't exist.
    pub fn get(&self, key: ArenaKey) -> Option<&T> {
        self.slotmap.get(key.0)
    }

    /// Gets the value at `key`, returning `Some(&mut T)` if it exists and `None` if it doesn't exist.
    pub fn get_mut(&mut self, key: ArenaKey) -> Option<&mut T> {
        self.slotmap.get_mut(key.0)
    }

    /// Same as `get()`, but unwraps and panics if the key isn't found
    /// (that is, if `get()` returns `None`).
    pub fn get_unwrap(&self, key: ArenaKey) -> &T {
        self.get(key)
            .expect("Called Arena::get_unwrap() with key but key not in arena.")
    }

    /// Same as `get_mut()`, but unwraps and panics if the key isn't found
    /// (that is, if `get_mut()` returns `None`).
    pub fn get_mut_unwrap(&mut self, key: ArenaKey) -> &mut T {
        self.get_mut(key)
            .expect("Called Arena::get_mut_unwrap() with key but key not in arena.")
    }

    /// Determines whether the arena has a value corresponding to `key`.
    pub fn has_key(&self, key: ArenaKey) -> bool {
        self.slotmap.contains_key(key.0)
    }
}
