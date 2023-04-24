#![deny(unsafe_op_in_unsafe_fn)]
/**
Given a closure that is assumed to be a wrapped openGauss `extern "C"` function, [og_guard_ffi_boundary]
works with the openGauss and C runtimes to create a "barrier" that allows Rust to catch openGauss errors
(`elog(ERROR)`) while running the supplied closure. This is done for the sake of allowing Rust to run
destructors before openGauss destroys the memory contexts that Rust-in-openGauss code may be enmeshed in.

Wrapping the FFI into openGauss enables
- memory safety
- improving error logging
- minimizing resource leaks

But only the first of these is considered paramount.

At all times OGX reserves the right to choose an implementation that achieves memory safety.
Currently, this function is only used by OGX's generated openGauss bindings.
It is not (yet) intended (or even necessary) for normal user code.

# Safety

This function should not be called from any thread but the main thread if such ever may throw an exception,
on account of the postmaster ultimately being a single-threaded runtime.

More generally, Rust cannot guarantee destructors are always run, OGX is written in Rust code, and
the implementation of `og_guard_ffi_boundary` relies on help from openGauss, the OS, and the C runtime;
thus, relying on the FFI boundary catching an error and propagating it back into Rust to guarantee
Rust's language-level memory safety when calling openGauss is unsound (i.e. there are no promises).
openGauss can and does opt to erase exception and error context stacks in some situations.
The C runtime is beholden to the operating system, which may do as it likes with a thread.
OGX has many magical powers, some of considerable size, but they are not infinite cosmic power.

Thus, if openGauss gives you a pointer into the database's memory, and you corrupt that memory
in a way technically permitted by Rust, intending to fix it before openGauss or Rust notices,
then you may not call openGauss and expect openGauss to not notice the code crimes in progress.
openGauss and Rust will see you. Whether they choose to ignore such misbehavior is up to them, not OGX.
If you are manipulating transient "pure Rust" data, however, it is unlikely this is of consequence.

# Implementation Note

The main implementation uses`sigsetjmp`, [`pg_sys::error_context_stack`], and [`pg_sys::PG_exception_stack`].
which, when openGauss enters its exception handling in `elog.c`, will prompt a `siglongjmp` back to it.

This caught error is then converted into a Rust `panic!()` and propagated up the stack, ultimately
being converted into a transaction-aborting openGauss `ERROR` by OGX.

**/
#[inline(always)]
pub(crate) unsafe fn og_guard_ffi_boundary<T, F: FnOnce() -> T>(f: F) -> T {
    // SAFETY: Caller promises not to call us from anything but the main thread.
    unsafe { og_guard_ffi_boundary_impl(f) }
}

#[inline(always)]
unsafe fn og_guard_ffi_boundary_impl<T, F: FnOnce() -> T>(f: F) -> T {
    //! This is the version that uses sigsetjmp and all that, for "normal" Rust/OGX interfaces.
    use crate as pg_sys;

    // SAFETY: This should really, really not be done in a multithreaded context as it
    // accesses multiple `static mut`. The ultimate caller asserts this is the main thread.
    unsafe {
        let prev_exception_stack = pg_sys::PG_exception_stack;
        let prev_error_context_stack = pg_sys::error_context_stack;
        let mut jump_buffer = std::mem::MaybeUninit::uninit();
        let jump_value = crate::sigsetjmp(jump_buffer.as_mut_ptr(), 0);

        let result = if jump_value == 0 {
            // first time through, not as the result of a longjmp
            pg_sys::PG_exception_stack = jump_buffer.as_mut_ptr();

            // execute the closure, which will be a wrapped internal openGauss function
            f()
        } else {
            // we're back here b/c of a longjmp originating in openGauss
            // as such, we need to put openGauss' understanding of its exception/error state back together
            pg_sys::PG_exception_stack = prev_exception_stack;
            pg_sys::error_context_stack = prev_error_context_stack;

            // and ultimately we panic
            std::panic::panic_any(pg_sys::JumpContext {});
        };

        pg_sys::PG_exception_stack = prev_exception_stack;
        pg_sys::error_context_stack = prev_error_context_stack;

        result
    }
}
