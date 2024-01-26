use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ffi::c_void;
use uefi::Event;
use uefi::prelude::BootServices;
use uefi::table::boot::{EventType, Tpl};
use core::ptr::NonNull;
use log::info;

pub struct ManagedEvent {
    pub event: Event,
    boxed_closure: *mut (dyn FnMut(Event) + 'static),
    boot_services: &'static BootServices,
}

/// Higher level modelling on top of the thin wrapper that uefi-rs provides.
/// The wrapper as-is can't be used because the wrapper can be cheaply cloned and passed around,
/// whereas we need there to be a single instance per event (so the destructor only runs once).
impl ManagedEvent {
    pub fn new<F>(
        bs: &'static BootServices,
        callback: F,
    ) -> Self
    where
        F: FnMut(Event) + 'static {
        let boxed_closure = Box::into_raw(Box::new(callback));
        unsafe {
            let event = bs.create_event(
                EventType::NOTIFY_WAIT,
                Tpl::CALLBACK,
                Some(call_closure::<F>),
                Some(NonNull::new(boxed_closure as *mut _ as *mut c_void).unwrap()),
            ).expect("Failed to create event");
            Self {
                event,
                boxed_closure,
                boot_services: bs,
            }
        }
    }

    pub fn wait(&self) {
        // Safety: The event clone is discarded after being passed to the UEFI function.
        unsafe {
            self.boot_services.wait_for_event(
                &mut [self.event.unsafe_clone()]
            ).expect("Failed to wait for transmit to complete");
        }
    }
}

impl Drop for ManagedEvent {
    fn drop(&mut self) {
        //info!("Dropping ManagedEvent");
        unsafe {
            // Close the UEFI handle
            // Safety: We're dropping the event here and don't use the handle again after
            // passing it to the UEFI function.
            self.boot_services.close_event(self.event.unsafe_clone()).expect("Failed to close event");
            // *Drop the box* that carries the closure.
            let _ = Box::from_raw(self.boxed_closure);
        }
    }
}

unsafe extern "efiapi" fn call_closure<F>(
    event: Event,
    raw_context: Option<NonNull<c_void>>,
) where F: FnMut(Event) + 'static {
    let unwrapped_context = cast_ctx(raw_context);
    let callback_ptr = unwrapped_context as *mut F;
    let callback = &mut *callback_ptr;
    callback(event);
    // Safety: *Don't drop the box* that carries the closure yet, because
    // the closure might be invoked again.
}

unsafe fn cast_ctx<T>(raw_val: Option<NonNull<c_void>>) -> &'static mut T {
    let val_ptr = raw_val.unwrap().as_ptr() as *mut c_void as *mut T;
    &mut *val_ptr
}
