use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::RefCell;
use log::info;
use uefi::{Event, Handle, Status, StatusExt};
use uefi::prelude::BootServices;
use crate::event::ManagedEvent;
use crate::ipv4::{IPv4Address, IPv4ModeData};
use crate::tcpv4::{TCPv4ConnectionLifecycleManager, TCPv4ConnectionMode};
use crate::tcpv4::transmit_data::TCPv4TransmitDataHandle;
use uefi::proto::unsafe_protocol;
use crate::tcpv4::definitions::{TCPv4CompletionToken, TCPv4ConfigData, TCPv4ConnectionState, TCPv4IoToken, UnmodelledPointer};
use uefi::Error;
use crate::tcpv4::receive_data::TCPv4ReceiveDataHandle;
use core::str;
use uefi::table::boot::{EventType, TimerTrigger};

#[derive(Debug)]
#[repr(C)]
#[unsafe_protocol("00720665-67EB-4a99-BAF7-D3C33A1C7CC9")]
pub struct TCPv4ServiceBindingProtocol {
    pub(crate) create_child: extern "efiapi" fn(
        this: &Self,
        out_child_handle: &mut Handle,
    ) -> Status,

    destroy_child: extern "efiapi" fn(
        this: &Self,
        child_handle: Handle,
    ) -> Status,
}


#[derive(Debug)]
#[repr(C)]
#[unsafe_protocol("65530BC7-A359-410F-B010-5AADC7EC2B62")]
pub struct TCPv4Protocol {
    get_mode_data_fn: extern "efiapi" fn(
        this: &Self,
        out_connection_state: Option<&mut TCPv4ConnectionState>,
        out_config_data: Option<&mut UnmodelledPointer>,
        out_ip4_mode_data: Option<&mut IPv4ModeData>,
        out_managed_network_config_data: Option<&mut UnmodelledPointer>,
        out_simple_network_mode: Option<&mut UnmodelledPointer>,
    ) -> Status,

    configure_fn: extern "efiapi" fn(
        this: &Self,
        config_data: Option<&TCPv4ConfigData>,
    ) -> Status,

    routes_fn: extern "efiapi" fn(
        this: &Self,
        delete_route: bool,
        subnet_address: &IPv4Address,
        subnet_mask: &IPv4Address,
        gateway_address: &IPv4Address,
    ) -> Status,

    connect_fn: extern "efiapi" fn(
        this: &Self,
        connection_token: &TCPv4CompletionToken,
    ) -> Status,

    accept_fn: extern "efiapi" fn(
        this: &Self,
        listen_token: &UnmodelledPointer,
    ) -> Status,

    pub(crate) transmit_fn: extern "efiapi" fn(
        this: &Self,
        token: &TCPv4IoToken,
    ) -> Status,

    pub receive_fn: extern "efiapi" fn(
        this: &Self,
        token: &TCPv4IoToken,
    ) -> Status,

    close_fn: extern "efiapi" fn(
        this: &Self,
        close_token: &UnmodelledPointer,
    ) -> Status,

    cancel_fn: extern "efiapi" fn(
        this: &Self,
        completion_token: &UnmodelledPointer,
    ) -> Status,

    poll_fn: extern "efiapi" fn(this: &Self) -> Status,
}

impl TCPv4Protocol {
    pub fn reset_stack(&self) {
        // The UEFI specification states that configuring with NULL options "brutally resets" the TCP stack
        (self.configure_fn)(
            self,
            None,
        ).to_result().expect("Failed to reset TCP stack")
    }

    pub fn configure(&self, bt: &BootServices, connection_mode: TCPv4ConnectionMode) -> uefi::Result<(), String> {
        let configuration = TCPv4ConfigData::new(connection_mode, None);
        // Maximum timeout of 10 seconds
        for _ in 0..10 {
            let result = (self.configure_fn)(
                self,
                Some(&configuration),
            );
            if result == Status::SUCCESS {
                info!("Configured connection! {result:?}");
                return Ok(())
            }
            else if result == Status::NO_MAPPING {
                info!("DHCP still running, waiting...");
                bt.stall(1_000_000);
            }
            else {
                info!("Error {result:?}, will spin and try again");
                bt.stall(1_000_000);
            }
        }
        Err(Error::new(Status::PROTOCOL_ERROR, "Timeout before configuring the connection succeeded.".to_string()))
    }

    pub fn get_tcp_connection_state(&self) -> TCPv4ConnectionState {
        let mut connection_state = core::mem::MaybeUninit::<TCPv4ConnectionState>::uninit();
        let connection_state_ptr = connection_state.as_mut_ptr();
        unsafe {
            (self.get_mode_data_fn)(
                self,
                Some(&mut *connection_state_ptr),
                None,
                None,
                None,
                None,
            ).to_result().expect("Failed to read connection state");
            connection_state.assume_init()
        }
    }

    pub fn get_ipv4_mode_data(&self) -> IPv4ModeData {
        let mut mode_data = core::mem::MaybeUninit::<IPv4ModeData>::uninit();
        let mode_data_ptr = mode_data.as_mut_ptr();
        unsafe {
            (self.get_mode_data_fn)(
                self,
                None,
                None,
                Some(&mut *mode_data_ptr),
                None,
                None,
            ).to_result().expect("Failed to read mode data");
            mode_data.assume_init()
        }
    }

    pub fn connect(
        &mut self,
        bs: &'static BootServices,
    ) {
        let event = ManagedEvent::new(
            bs,
            EventType::NOTIFY_WAIT,
            |_| {},
        );
        let completion_token = TCPv4CompletionToken::new(&event);
        (self.connect_fn)(
            &self,
            &completion_token,
        ).to_result().expect("Failed to call Connect()");
        event.wait();
    }

    pub fn transmit(
        &mut self,
        bs: &'static BootServices,
        data: &[u8],
    ) {
        let event = ManagedEvent::new(
            bs,
            EventType::NOTIFY_WAIT,
            move |_e| {
            info!("This should not be called. Callback: Transmit complete!");
            //lifecycle_clone.borrow_mut().register_transmitting_complete();
        });

        let tx_data_handle = TCPv4TransmitDataHandle::new(data);
        let tx_data = tx_data_handle.get_data_ref();
        let io_token = TCPv4IoToken::new(&event, Some(&tx_data), None);
        (self.transmit_fn)(
            &self,
            &io_token,
        ).to_result().expect("Failed to transmit");
        match str::from_utf8(&data) {
            Ok(v) => {
                info!("TX {v}");
            },
            Err(_e) => {
                info!("Transmit data (no decode) {data:?}");
            }
        };
        event.wait();
    }
}
