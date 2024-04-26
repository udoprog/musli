extern "C" {
    /// Abort the program in no-std environments.
    ///
    /// This has to be implemented by the caller, and is used for unrecoverable
    /// and unusual errors. Such as when a reference count is overflowing.
    fn __musli_abort() -> !;
}

pub(crate) fn abort() -> ! {
    unsafe { __musli_abort() }
}
