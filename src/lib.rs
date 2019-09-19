mod com_ptr;
mod inproc;
pub mod interfaces;
pub mod offset;
mod runtime;

pub use com_ptr::ComPtr;
pub use inproc::*;
pub use runtime::ApartmentThreadedRuntime;

use interfaces::iunknown::IUnknown;
use winapi::shared::guiddef::IID;

/// A COM compliant interface
///
/// # Safety
///
/// The trait or struct implementing this trait must provide a valid vtable as the
/// associated VTable type. A vtable is valid if:
/// * it is `#[repr(C)]`
/// * the type only contains `extern "stdcall" fn" definitions
pub unsafe trait ComInterface: IUnknown {
    type VTable;
    const IID: IID;

    fn is_iid_in_inheritance_chain(riid: &IID) -> bool;
}

/// A COM compliant class
///
/// # Safety
///
/// The implementing struct must have the following properties:
/// * it is `#[repr(C)]`
/// * The first fields of the struct are pointers to the backing VTables for
/// each of the COM Interfaces the class implements
pub unsafe trait CoClass: IUnknown {}

pub trait ProductionComInterface<T: IUnknown>: ComInterface {
    fn vtable<O: offset::Offset>() -> Self::VTable;
}

#[macro_export]
macro_rules! vtable {
    ($class:ident: $interface:ident, $offset:ident) => {
        <dyn $interface as $crate::ProductionComInterface<$class>>::vtable::<
            $crate::offset::$offset,
        >();
    };
    ($class:ident: $interface:ident, 2usize) => {
        $crate::vtable!($class: $interface, Two)
    };
    ($class:ident: $interface:ident, 1usize) => {
        $crate::vtable!($class: $interface, One)
    };
    ($class:ident: $interface:ident, 0usize) => {
        $crate::vtable!($class: $interface, Zero)
    };
    ($class:ident: $interface:ident) => {
        $crate::vtable!($class: $interface, Zero)
    };
}

// Export winapi for use by macros
#[doc(hidden)]
pub extern crate winapi as _winapi;

#[doc(hidden)]
pub use macros::*;

// this allows for the crate to refer to itself as `com` to keep macros consistent
// whether they are used by some other crate or internally
#[doc(hidden)]
extern crate self as com;
