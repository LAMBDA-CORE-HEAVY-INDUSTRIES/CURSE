#![cfg(feature = "perf")]

use core::sync::atomic::{compiler_fence, Ordering};

use cortex_m::peripheral::DWT;

/// Simple cycle-count profiling utilities.
///
/// Example:
/// ```rust,no_run
/// # use curse::perf;
/// # fn rebuild_rt_cache() {}
/// #[cfg(feature = "perf")]
/// perf::init_cycle_counter();
/// #[cfg(feature = "perf")]
/// {
///     let cycles = perf::measure_cycles(|| {
///         rebuild_rt_cache();
///     });
///     rtt_target::rprintln!("{}", cycles);
/// }
/// ```

/// Enable DWT cycle counter. Call once early in startup.
pub fn init_cycle_counter() {
    unsafe {
        let mut cp = cortex_m::Peripherals::steal();
        cp.DCB.enable_trace();
        cp.DWT.enable_cycle_counter();
    }
}

pub fn measure_cycles<F: FnOnce()>(f: F) -> u32 {
    compiler_fence(Ordering::SeqCst);
    let start = DWT::cycle_count();
    f();
    let end = DWT::cycle_count();
    compiler_fence(Ordering::SeqCst);
    end.wrapping_sub(start)
}
