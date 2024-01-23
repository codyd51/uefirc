use alloc::boxed::Box;
use core::ffi::c_void;
use uefi::Event;
use uefi::prelude::BootServices;
use uefi::table::boot::{EventType, Tpl};
use core::ptr::NonNull;

// PT: Trait because I can't directly modify Event in UEFI.
// This can go away if I upstream my code to uefi-rs.
pub trait EventExt {
    fn new<F>(
        bs: &BootServices,
        callback: F,
    ) -> Event
        where
            F: FnMut(Event) + 'static;
}

impl EventExt for Event {
    fn new<F>(
        bs: &BootServices,
        callback: F,
    ) -> Event
        where
            F: FnMut(Event) + 'static {
        let data = Box::into_raw(Box::new(callback));
        unsafe {
            bs.create_event(
                EventType::NOTIFY_WAIT,
                Tpl::CALLBACK,
                Some(call_closure::<F>),
                Some(NonNull::new(data as *mut _ as *mut c_void).unwrap()),
            ).expect("Failed to create event")
        }
    }
}

unsafe extern "efiapi" fn call_closure<F>(
    event: Event,
    raw_context: Option<NonNull<c_void>>,
)
    where
        F: FnMut(Event) + 'static {
    let unwrapped_context = cast_ctx(raw_context);
    let callback_ptr = unwrapped_context as *mut F;
    let callback = &mut *callback_ptr;
    callback(event);
    // Safety: *Don't drop the box* that carries the closure, because
    // the closure might be invoked again.
    // let _ = Box::from_raw(unwrapped_context as *mut _);
}

unsafe fn cast_ctx<T>(raw_val: Option<core::ptr::NonNull<c_void>>) -> &'static mut T {
    let val_ptr = raw_val.unwrap().as_ptr() as *mut c_void as *mut T;
    &mut *val_ptr
}
