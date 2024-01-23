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
use crate::ipv4::IPv4ModeData;
use crate::tcpv4::{TCPv4CompletionToken, TCPv4ConfigData, TCPv4ConnectionState, TCPv4IoToken, TCPv4Option, TCPv4Protocol, TCPv4ServiceBindingProtocol, TCPv4TransmitData};

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
    let tcp = get_tcp_protocol(bt, &tcp_service_binding);

    // 'Brutally reset' the TCP stack
    let result = (tcp.configure)(
        &tcp,
        None,
    );
    info!("Result of brutal reset {result:?}");

    let configuration = TCPv4ConfigData::new(None);
    info!("Configuration {configuration:?}");

    loop {
        let result = (tcp.configure)(
            &tcp,
            Some(&configuration),
        );
        if result == Status::SUCCESS {
            info!("Configured connection! {result:?}");
            break;
        }
        else if result == Status::NO_MAPPING {
            info!("DHCP still running, waiting...");
            bt.stall(1_000_000);
        }
        else {
            info!("Error {result:?}, will spin and try again");
            bt.stall(1_000_000);
            //result.to_result().expect("Failed to configure TCP connection");
        }
    }

    let mut connection_state = core::mem::MaybeUninit::<TCPv4ConnectionState>::uninit();
    let mut connection_state_ptr = connection_state.as_mut_ptr();

    let mut mode_data = core::mem::MaybeUninit::<IPv4ModeData>::uninit();
    let mut mode_data_ptr = mode_data.as_mut_ptr();
    unsafe {
        (tcp.get_mode_data)(
            &tcp,
            Some(&mut *connection_state_ptr),
            None,
            Some(&mut *mode_data_ptr),
            None,
            None,
        ).to_result().expect("Failed to read mode data");
    }
    let mode_data = unsafe { mode_data.assume_init() };
    info!("Got mode data: {mode_data:?}");
    let connection_state = unsafe { connection_state.assume_init() };
    info!("Got connection state: {connection_state:?}");

    // Initiate the connection
    let event = unsafe {
        bt.create_event(
            EventType::NOTIFY_SIGNAL,
            Tpl::CALLBACK,
            Some(handle_connection_operation_completed),
            None,
        ).unwrap()
    };
    let completion_token = TCPv4CompletionToken::new(event);
    let result = (tcp.connect)(
        &tcp,
        &completion_token,
    );
    info!("Result of calling connect(): {result:?}");
    bt.stall(1_000_000);

    for i in 0..10 {
        info!("Running another iteration {i}");
        let tx_data = TCPv4TransmitData::new(b"NICK phillip-testing\r\n");
        let event = unsafe {
            bt.create_event(
                EventType::NOTIFY_SIGNAL,
                Tpl::CALLBACK,
                Some(handle_notify_signal),
                None,
            ).unwrap()
        };
        info!("Got event {event:?}");
        let io = TCPv4IoToken::new(event, &tx_data);
        info!("TX Data {tx_data:?}");
        let result = (tcp.transmit)(
            &tcp,
            &io,
        );
        info!("Output: {result:?}");
        bt.stall(2_000_000);
    }

    loop {
        //info!("Spinning...");
        bt.stall(1_000_000);
    }

    Status::SUCCESS
}

unsafe extern "efiapi" fn handle_notify_signal(e: Event, _ctx: Option<NonNull<c_void>>) {
    info!("handle_notify_signal {e:?}");
}

unsafe extern "efiapi" fn handle_connection_operation_completed(e: Event, _ctx: Option<NonNull<c_void>>) {
    info!("handle_connection_operation_completed {e:?}");
}
