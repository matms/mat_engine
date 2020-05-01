/// The same as
/// `o.as_ref().unwrap()`,
/// but more convenient.
pub(crate) fn unwrap_ref<'a, T>(o: &'a Option<T>) -> &'a T {
    o.as_ref()
        .expect("Attempted to get borrow of contents of Option, but Option was None")
}

/// The same as
/// `o.as_mut().unwrap()`,
/// but more convenient.
pub(crate) fn unwrap_mut<'a, T>(o: &'a mut Option<T>) -> &'a mut T {
    o.as_mut()
        .expect("Attempted to get mut borrow of contents of Option, but Option was None")
}
