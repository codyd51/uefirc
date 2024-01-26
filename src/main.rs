#![no_main]
#![no_std]
#[allow(dead_code)]

mod tcpv4;
mod ipv4;
mod event;

extern crate alloc;

use core::str;
use alloc::vec;
use alloc::vec::Vec;
use log::info;
use uefi::prelude::*;
use uefi::table::boot::{EventType, OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol, TimerTrigger};
use crate::event::ManagedEvent;
use crate::ipv4::IPv4Address;
use crate::tcpv4::{TCPv4ClientConnectionModeParams, TCPv4ConnectionMode, TCPv4IoToken, TCPv4Protocol, TCPv4ReceiveDataHandle, TCPv4ServiceBindingProtocol};

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
    let tcp_handle_ptr = tcp_handle.as_mut_ptr();
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
    let bs: &'static BootServices = unsafe {
        core::mem::transmute(bs)
    };
    let tcp_service_binding_protocol = get_tcp_service_binding_protocol(bs);
    let tcp = get_tcp_protocol(bs, &tcp_service_binding_protocol);

    let mut connection = TcpConnection::new(
        bs,
        tcp,
        IPv4Address::new(93, 158, 237, 2),
        6665,
    );

    connection.transmit(b"NICK phillip-testing\r\n");
    loop {
        connection.step();
    }
}

struct TcpConnection<'a> {
    boot_services: &'static BootServices,
    tcp: ScopedProtocol<'a, TCPv4Protocol>,
    active_rx: Option<(ManagedEvent, TCPv4ReceiveDataHandle)>,
    recv_buffer: Vec<u8>,
}

impl<'a> TcpConnection<'a> {
    fn new(
        boot_services: &'static BootServices,
        mut tcp: ScopedProtocol<'a, TCPv4Protocol>,
        remote_ip: IPv4Address,
        remote_port: u16,
    ) -> Self {
        tcp.configure(
            boot_services,
            TCPv4ConnectionMode::Client(
                TCPv4ClientConnectionModeParams::new(remote_ip, remote_port),
            )
        ).expect("Failed to configure the TCP connection");
        tcp.connect(boot_services);

        Self {
            boot_services,
            tcp,
            active_rx: None,
            recv_buffer: vec![],
        }
    }

    fn transmit(&mut self, data: &[u8]) {
        self.tcp.transmit(&self.boot_services, data)
    }

    fn receive_with_timeout(&mut self) {
        let bs = &self.boot_services;
        let timer_event = ManagedEvent::new(
            bs,
            EventType::TIMER,
            // UEFI doesn't invoke callbacks for timers
            move |_|{ info!("Should not happen: Timer callback called"); }
        );
        let one_ms = 1_000;
        bs.set_timer(
            &timer_event.event,
            TimerTrigger::Relative(one_ms * 100)
        ).expect("Failed to set timer");

        if self.active_rx.is_none() {
            let rx_event = ManagedEvent::new(
                bs,
                EventType::NOTIFY_WAIT,
                |_| {},
            );
            let rx_data_handle = TCPv4ReceiveDataHandle::new();
            let rx_data = rx_data_handle.get_data_ref();
            let io_token = TCPv4IoToken::new(&rx_event, None, Some(&rx_data));
            let result = (self.tcp.receive_fn)(
                &self.tcp,
                &io_token,
            );
            result.to_result().expect("Failed to initiate recv");
            self.active_rx = Some((rx_event, rx_data_handle));
        }
        else {
            // The previous iteration must have been a timeout, so the previous RX event and handle
            // are still in progress / being held by UEFI.
        }
        let (rx_event, rx_data_handle) = self.active_rx.as_ref().unwrap();

        let triggered_event_idx = ManagedEvent::wait_for_events(
            bs,
            &[rx_event, &timer_event],
        );
        match triggered_event_idx {
            0 => {
                // The 'receive' event was triggered, we have data to read!
                let received_data = rx_data_handle.get_data_ref().read_buffers();
                self.recv_buffer.extend_from_slice(&received_data);
                match str::from_utf8(&received_data) {
                    Ok(v) => {
                        info!("RX {v}");
                    },
                    Err(_) => {
                        info!("RX (no decode) {0:?}", received_data);
                    }
                };
                self.active_rx = None;
            }
            1 => {
                // The timeout was triggered, no data is available now
            }
            _ => panic!("Unexpected index"),
        }
    }

    fn step(&mut self) {
        // Give ourselves a chance to receive data
        self.receive_with_timeout();
    }
}
