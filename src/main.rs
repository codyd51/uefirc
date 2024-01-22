#![no_main]
#![no_std]

mod tcpv4;

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::mem;
use core::ptr::null;
use log::info;
use uefi::prelude::*;
use uefi::{Guid, guid, Result};
use uefi::proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput};
use uefi::proto::media::block::BlockIoProtocol;
use uefi::proto::rng::Rng;
use uefi::table::boot::{OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol};
use uefi::proto::unsafe_protocol;
use crate::tcpv4::{TCPv4ConfigData, TCPv4Option, TCPv4Protocol, TCPv4ServiceBindingProtocol};

fn get_tcp_service_binding_protocol(bt: &BootServices) -> ScopedProtocol<TCPv4ServiceBindingProtocol> {
    let tcp_service_binding_handle = bt.get_handle_for_protocol::<TCPv4ServiceBindingProtocol>().unwrap();
    let tcp_service_binding = unsafe {
        bt.open_protocol::<TCPv4ServiceBindingProtocol>(
            OpenProtocolParams {
                handle: tcp_service_binding_handle,
                agent: bt.image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        ).expect("Failed to open TCP service binding protocol")
    };
    tcp_service_binding
}

fn get_tcp_protocol<'a>(
    bt: &'a BootServices,
    tcp_service_binding_proto: &'a ScopedProtocol<'a, TCPv4ServiceBindingProtocol>,
) -> ScopedProtocol<'a, TCPv4Protocol> {
    let mut tcp_handle = core::mem::MaybeUninit::<Handle>::uninit();
    let mut tcp_handle_ptr = tcp_handle.as_mut_ptr();
    let result = unsafe {
        (tcp_service_binding_proto.create_child)(
            &tcp_service_binding_proto,
            &mut *tcp_handle_ptr,
        )
    }.to_result();
    result.expect("Failed to create TCP child protocol");
    let tcp_handle = unsafe { tcp_handle.assume_init() };

    let tcp_proto = unsafe {
        bt.open_protocol::<TCPv4Protocol>(
            OpenProtocolParams {
                handle: tcp_handle,
                agent: bt.image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        )
    }.expect("Failed to open TCP protocol");
    tcp_proto
}

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    let bt = system_table.boot_services();

    let tcp_service_binding = get_tcp_service_binding_protocol(bt);
    info!("Got TCP service binding protocol {tcp_service_binding:?}");
    let tcp = get_tcp_protocol(bt, &tcp_service_binding);
    info!("Got TCP protocol {tcp:?}");

    // 'Brutally reset' the TCP stack
    let result = (tcp.configure)(
        &tcp,
        None,
    );
    info!("Result of brutal reset {result:?}");

    let options = TCPv4Option::new();
    let configuration = TCPv4ConfigData::new(None);
    info!("Options {options:?}");
    info!("Configuration {configuration:?}");
    let result = (tcp.configure)(
        &tcp,
        Some(&configuration),
    );
    info!("Configured connection! {result:?}");

    bt.stall(1_000_000);
    Status::SUCCESS
}
