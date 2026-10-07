//! Helper module to set up everything you need in a no-std environment withotu
//! alloc support.

#[cfg(all(windows, target_env = "msvc"))]
#[link(name = "msvcrt")]
unsafe extern "C" {}

#[cfg(unix)]
#[link(name = "c")]
unsafe extern "C" {}

unsafe extern "C" {
    fn abort() -> !;
}

#[panic_handler]
fn rust_begin_panic(_: &core::panic::PanicInfo) -> ! {
    // SAFETY: `abort` is provided by the C runtime linked above and takes no
    // arguments.
    unsafe { abort() }
}

#[lang = "eh_personality"]
unsafe extern "C" fn eh_personality() {}

#[cfg(unix)]
#[unsafe(no_mangle)]
pub extern "C" fn _Unwind_Resume() {}
