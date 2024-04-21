// TODO: Make use of core::abort intrinsics when they are available.
#[allow(clippy::empty_loop)]
pub(crate) fn abort() -> ! {
    loop {}
}
