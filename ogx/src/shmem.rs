/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/
use crate::lwlock::*;
use crate::{pg_sys, PgAtomic};
use std::hash::Hash;
use uuid::Uuid;

/// Custom types that want to participate in shared memory must implement this marker trait
pub unsafe trait OGXSharedMemory {}

/// In order to store a type in openGauss Shared Memory, it must be passed to
/// `pg_shmem_init!()` during `_PG_init()`.
///
/// Additionally, the type must be a `static` global and also be `#[derive(Copy, Clone)]`.
///
/// > Types that allocate on the heap, such as `String` and `Vec` are not supported.
///
/// For complex data structures like vecs and maps, `ogx` prefers the use of types from
/// [`heapless`](https://crates.io/crates/heapless).
///
/// Custom types need to also implement the `OGXSharedMemory` trait.
///
/// > Extensions that use shared memory **must** be loaded via `postgresql.conf`'s
/// `shared_preload_libraries` configuration setting.  
///
/// # Example
///
/// ```rust,no_run
/// use ogx::*;
///
/// // primitive types must be protected behind a `PgLwLock`
/// static PRIMITIVE: PgLwLock<i32> = PgLwLock::new();
///
/// // Rust atomics can be used without locks, wrapped in a `PgAtomic`
/// static ATOMIC: PgAtomic<std::sync::atomic::AtomicBool> = PgAtomic::new();
///
/// #[og_guard]
/// pub extern "C" fn _PG_init() {
///     pg_shmem_init!(PRIMITIVE);
///     pg_shmem_init!(ATOMIC);
/// }
/// ```
#[macro_export]
macro_rules! pg_shmem_init {
    ($thing:expr) => {
        $thing.pg_init();

        unsafe {
            static mut PREV_SHMEM_STARTUP_HOOK: Option<unsafe extern "C" fn()> = None;
            PREV_SHMEM_STARTUP_HOOK = pg_sys::shmem_startup_hook;
            pg_sys::shmem_startup_hook = Some(__ogx_private_shmem_hook);

            #[og_guard]
            extern "C" fn __ogx_private_shmem_hook() {
                unsafe {
                    if let Some(i) = PREV_SHMEM_STARTUP_HOOK {
                        i();
                    }
                }
                $thing.shmem_init();
            }
        }
    };
}
/// A trait that types can implement to provide their own Postgres Shared Memory initialization process
pub trait PgSharedMemoryInitialization {
    /// Automatically called when the an extension is loaded.  If using the `pg_shmem_init!()` macro
    /// in `_PG_init()`, this is called automatically
    fn pg_init(&'static self);

    /// Automatically called by the `pg_shmem_init!()` macro, when Postgres is initializing its
    /// shared memory system
    fn shmem_init(&'static self);
}

impl<T> PgSharedMemoryInitialization for PgLwLock<T>
where
    T: Default + OGXSharedMemory + 'static,
{
    fn pg_init(&'static self) {
        PgSharedMem::pg_init_locked(self);
    }

    fn shmem_init(&'static self) {
        PgSharedMem::shmem_init_locked(self);
    }
}

impl<T> PgSharedMemoryInitialization for PgAtomic<T>
where
    T: atomic_traits::Atomic + Default,
{
    fn pg_init(&'static self) {
        PgSharedMem::pg_init_atomic(self);
    }

    fn shmem_init(&'static self) {
        PgSharedMem::shmem_init_atomic(self);
    }
}

/// This struct contains methods to drive creation of types in shared memory
pub struct PgSharedMem {}

impl PgSharedMem {
    /// Must be run from PG_init, use for types which are guarded by a LWLock
    pub fn pg_init_locked<T: Default + OGXSharedMemory>(lock: &PgLwLock<T>) {
        unsafe {
            let lock = std::ffi::CString::new(lock.get_name()).expect("CString::new failed");
            pg_sys::RequestAddinShmemSpace(std::mem::size_of::<T>());
            pg_sys::RequestNamedLWLockTranche(lock.as_ptr(), 1);
        }
    }

    /// Must be run from _PG_init for atomics
    pub fn pg_init_atomic<T: atomic_traits::Atomic + Default>(_atomic: &PgAtomic<T>) {
        unsafe {
            pg_sys::RequestAddinShmemSpace(std::mem::size_of::<T>());
        }
    }

    /// Must be run from the shared memory init hook, use for types which are guarded by a `LWLock`
    pub fn shmem_init_locked<T: Default + OGXSharedMemory>(lock: &PgLwLock<T>) {
        let mut found = false;
        unsafe {
            let shm_name = std::ffi::CString::new(lock.get_name()).expect("CString::new failed");
            let addin_shmem_init_lock: *mut pg_sys::LWLock =
                &mut (*pg_sys::MainLWLockArray.add(21)).lock;
            pg_sys::LWLockAcquire(addin_shmem_init_lock, pg_sys::LWLockMode_LW_EXCLUSIVE);

            let fv_shmem =
                pg_sys::ShmemInitStruct(shm_name.into_raw(), std::mem::size_of::<T>(), &mut found)
                    as *mut T;

            std::ptr::write(fv_shmem, <T>::default());

            lock.attach(fv_shmem);
            pg_sys::LWLockRelease(addin_shmem_init_lock);
        }
    }

    /// Must be run from the shared memory init hook, use for rust atomics behind `PgAtomic`
    pub fn shmem_init_atomic<T: atomic_traits::Atomic + Default>(atomic: &PgAtomic<T>) {
        unsafe {
            let shm_name =
                std::ffi::CString::new(Uuid::new_v4().to_string()).expect("CString::new() failed");

            let addin_shmem_init_lock: *mut pg_sys::LWLock =
                &mut (*pg_sys::MainLWLockArray.add(21)).lock;

            let mut found = false;
            pg_sys::LWLockAcquire(addin_shmem_init_lock, pg_sys::LWLockMode_LW_EXCLUSIVE);
            let fv_shmem =
                pg_sys::ShmemInitStruct(shm_name.into_raw(), std::mem::size_of::<T>(), &mut found)
                    as *mut T;

            atomic.attach(fv_shmem);
            let atomic = T::default();
            std::ptr::copy(&atomic, fv_shmem, 1);
            pg_sys::LWLockRelease(addin_shmem_init_lock);
        }
    }
}

unsafe impl OGXSharedMemory for bool {}
unsafe impl OGXSharedMemory for char {}
unsafe impl OGXSharedMemory for str {}
unsafe impl OGXSharedMemory for () {}
unsafe impl OGXSharedMemory for i8 {}
unsafe impl OGXSharedMemory for i16 {}
unsafe impl OGXSharedMemory for i32 {}
unsafe impl OGXSharedMemory for i64 {}
unsafe impl OGXSharedMemory for i128 {}
unsafe impl OGXSharedMemory for u8 {}
unsafe impl OGXSharedMemory for u16 {}
unsafe impl OGXSharedMemory for u32 {}
unsafe impl OGXSharedMemory for u64 {}
unsafe impl OGXSharedMemory for u128 {}
unsafe impl OGXSharedMemory for usize {}
unsafe impl OGXSharedMemory for isize {}
unsafe impl OGXSharedMemory for f32 {}
unsafe impl OGXSharedMemory for f64 {}
unsafe impl<T> OGXSharedMemory for [T] where T: OGXSharedMemory + Default {}
unsafe impl<A, B> OGXSharedMemory for (A, B)
where
    A: OGXSharedMemory + Default,
    B: OGXSharedMemory + Default,
{
}
unsafe impl<A, B, C> OGXSharedMemory for (A, B, C)
where
    A: OGXSharedMemory + Default,
    B: OGXSharedMemory + Default,
    C: OGXSharedMemory + Default,
{
}
unsafe impl<A, B, C, D> OGXSharedMemory for (A, B, C, D)
where
    A: OGXSharedMemory + Default,
    B: OGXSharedMemory + Default,
    C: OGXSharedMemory + Default,
    D: OGXSharedMemory + Default,
{
}
unsafe impl<A, B, C, D, E> OGXSharedMemory for (A, B, C, D, E)
where
    A: OGXSharedMemory + Default,
    B: OGXSharedMemory + Default,
    C: OGXSharedMemory + Default,
    D: OGXSharedMemory + Default,
    E: OGXSharedMemory + Default,
{
}
unsafe impl<T, const N: usize> OGXSharedMemory for heapless::Vec<T, N> {}
unsafe impl<K: Eq + Hash, V: Default, S, const N: usize> OGXSharedMemory
    for heapless::IndexMap<K, V, S, N>
{
}
