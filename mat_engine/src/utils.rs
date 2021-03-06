/// The same as
/// `o.as_ref().expect(...)`,
/// but more convenient.
pub(crate) fn unwrap_ref<'a, T>(o: &'a Option<T>) -> &'a T {
    o.as_ref().expect(
        "Attempted to get borrow of contents of Option, but Option was None.\
              It is likely that you attempted to do something that needed a certain system \
              to be initialized, but it wasn't.",
    )
}

/// The same as
/// `o.as_mut().expect(...)`,
/// but more convenient.
pub(crate) fn unwrap_mut<'a, T>(o: &'a mut Option<T>) -> &'a mut T {
    o.as_mut().expect(
        "Attempted to get mut borrow of contents of Option, but Option was None.\
              It is likely that you attempted to do something that needed a certain system \
              to be initialized, but it wasn't.",
    )
}
