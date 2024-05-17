//! Helper module to set up everything you need in a no-std environment withotu
//! alloc support.

#[cfg(all(windows, target_env = "msvc"))]
#[link(name = "msvcrt")]
extern "C" {}

#[cfg(unix)]
#[link(name = "c")]
extern "C" {}

#[panic_handler]
#[lang = "panic_impl"]
fn rust_begin_panic(_: &core::panic::PanicInfo) -> ! {
    core::intrinsics::abort();
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[cfg(unix)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() {}
