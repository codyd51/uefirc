#![feature(rustc_private)]
#![cfg_attr(feature = "run_in_uefi", no_std)]
#![cfg_attr(feature = "run_in_uefi", feature(start))]
#![cfg_attr(feature = "run_in_uefi", no_main)]

#[cfg(feature = "run_in_uefi")]
mod tcpv4;
#[cfg(feature = "run_in_uefi")]
mod ipv4;
#[cfg(feature = "run_in_uefi")]
mod event;
#[cfg(feature = "run_in_uefi")]
mod connection;
#[cfg(feature = "run_in_uefi")]
mod ui;
#[cfg(feature = "run_in_uefi")]
mod app;
#[cfg(feature = "run_in_uefi")]
mod fs;

mod gui;

extern crate alloc;

/* For when running in UEFI */

#[cfg(feature = "run_in_uefi")]
use uefi::Status;
#[cfg(feature = "run_in_uefi")]
use uefi::prelude::*;
#[cfg(feature = "run_in_uefi")]
mod main_uefi;

#[cfg(feature = "run_in_uefi")]
#[entry]
fn main(image_handle: Handle, system_table: SystemTable<Boot>) -> Status {
    main_uefi::main(image_handle, system_table)
}

/* For when running in a hosted environment */

#[cfg(not(feature = "run_in_uefi"))]
mod main_hosted;

#[cfg(not(feature = "run_in_uefi"))]
fn main() {
    main_hosted::main();
}

