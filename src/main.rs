#![no_main]
#![no_std]
#![feature(core_intrinsics)]

mod tcpv4;
mod ipv4;

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::mem;
use core::ptr::{null, NonNull};
use log::info;
use uefi::prelude::*;
use uefi::{Event, Guid, guid, Result};
use uefi::proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput};
use uefi::proto::media::block::BlockIoProtocol;
use uefi::proto::rng::Rng;
use uefi::table::boot::{EventType, OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol, Tpl};
use uefi::proto::unsafe_protocol;
use crate::ipv4::{IPv4Address, IPv4ModeData};
use crate::tcpv4::{TCPv4ClientConnectionModeParams, TCPv4CompletionToken, TCPv4ConfigData, TCPv4ConnectionLifecycleManager, TCPv4ConnectionMode, TCPv4ConnectionState, TCPv4IoToken, TCPv4Option, TCPv4Protocol, TCPv4ServiceBindingProtocol, TCPv4TransmitData};

fn get_tcp_service_binding_protocol(bs: &BootServices) -> ScopedProtocol<TCPv4ServiceBindingProtocol> {
    let tcp_service_binding_handle = bs.get_handle_for_protocol::<TCPv4ServiceBindingProtocol>().unwrap();
    let tcp_service_binding = unsafe {
        bs.open_protocol::<TCPv4ServiceBindingProtocol>(
            OpenProtocolParams {
                handle: tcp_service_binding_handle,
                agent: bs.image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        ).expect("Failed to open TCP service binding protocol")
    };
    tcp_service_binding
}

fn get_tcp_protocol<'a>(
    bs: &'a BootServices,
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
        bs.open_protocol::<TCPv4Protocol>(
            OpenProtocolParams {
                handle: tcp_handle,
                agent: bs.image_handle(),
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
    let bs = system_table.boot_services();

    let tcp_service_binding = get_tcp_service_binding_protocol(bs);
    let mut tcp = get_tcp_protocol(bs, &tcp_service_binding);

    //tcp.reset_stack();
    tcp.configure(
        bs,
        TCPv4ConnectionMode::Client(
            TCPv4ClientConnectionModeParams::new(
                IPv4Address::new(93, 158, 237, 2),
                6665,
            ),
        )
    ).expect("Failed to configure the TCP connection");

    /*
    let mode_data = tcp.get_ipv4_mode_data();
    info!("Got mode data: {mode_data:?}");
    let connection_state = tcp.get_tcp_connection_state();
    info!("Got connection state: {connection_state:?}");
    */

    let mut lifecycle = TCPv4ConnectionLifecycleManager::new();
    tcp.connect(&bs, &mut lifecycle);

    //let tx_data = TCPv4TransmitData::new(b"NICK phillip-testing\r\n");
    //info!("Tx data {tx_data:?}");
    for _ in 0..1 {
        //info!("Start iteration {i}");
        tcp.transmit(&bs, &mut lifecycle, b"NICK phillip-testing\r\n");
        //tcp.transmit(&bs, &mut lifecycle, b"");
        //info!("Finished iteration {i}");
    }

    loop {
        bs.stall(1_000_000);
    }

    Status::SUCCESS
}

unsafe extern "efiapi" fn handle_notify_signal(e: Event, _ctx: Option<NonNull<c_void>>) {
    info!("handle_notify_signal {e:?}");
}

unsafe extern "efiapi" fn handle_connection_operation_completed(e: Event, _ctx: Option<NonNull<c_void>>) {
    info!("handle_connection_operation_completed {e:?}");
}
