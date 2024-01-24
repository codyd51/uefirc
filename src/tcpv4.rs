use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::alloc::Layout;
use core::cell::RefCell;
use core::ffi::c_void;
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::copy_nonoverlapping;
use log::info;
use uefi::{Error, Event, Handle, Status, StatusExt};
use uefi::prelude::BootServices;

use uefi::proto::unsafe_protocol;
use crate::ipv4::{IPv4Address, IPv4ModeData};
use crate::event::ManagedEvent;

#[derive(Debug)]
#[repr(C)]
pub struct UnmodelledPointer(pub *mut c_void);

#[derive(Debug)]
#[repr(C)]
pub struct TCPv4AccessPoint {
    use_default_address: bool,
    station_address: IPv4Address,
    subnet_mask: IPv4Address,
    station_port: u16,
    remote_address: IPv4Address,
    remote_port: u16,
    active_flag: bool,
}

impl TCPv4AccessPoint {
    fn new(connection_mode: TCPv4ConnectionMode) -> Self {
        let (remote_ip, remote_port, is_client) = match connection_mode {
            TCPv4ConnectionMode::Client(params) => {
                (params.remote_ip, params.remote_port, true)
            }
            TCPv4ConnectionMode::Server => {
                (IPv4Address::zero(), 0, false)
            }
        };
        Self {
            use_default_address: true,
            // These two fields are meaningless because we set use_default_address above
            station_address: IPv4Address::zero(),
            subnet_mask: IPv4Address::zero(),
            // Chosen on-demand
            station_port: 0,
            remote_address: remote_ip,
            remote_port,
            active_flag: is_client,

        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct TCPv4Option {
    receive_buffer_size: u32,
    send_buffer_size: u32,
    max_syn_back_log: u32,
    connection_timeout: u32,
    data_retries: u32,
    fin_timeout: u32,
    time_wait_timeout: u32,
    keep_alive_probes: u32,
    keep_alive_time: u32,
    keep_alive_interval: u32,
    enable_nagle: bool,
    enable_time_stamp: bool,
    enable_window_scaling: bool,
    enable_selective_ack: bool,
    enable_path_mtu_discovery: bool,
}

#[derive(Debug)]
#[repr(C)]
pub struct TCPv4ConfigData<'a> {
    type_of_service: u8,
    time_to_live: u8,
    access_point: TCPv4AccessPoint,
    option: Option<&'a TCPv4Option>,
}

#[derive(Debug)]
pub struct TCPv4ClientConnectionModeParams {
    remote_ip: IPv4Address,
    remote_port: u16,
}

impl TCPv4ClientConnectionModeParams {
    pub fn new(
        remote_ip: IPv4Address,
        remote_port: u16,
    ) -> Self {
        Self {
            remote_ip,
            remote_port,
        }
    }
}

#[derive(Debug)]
pub enum TCPv4ConnectionMode {
    Client(TCPv4ClientConnectionModeParams),
    // TODO(PT): There may be parameters we need to model when operating as a server
    Server,
}

impl<'a> TCPv4ConfigData<'a> {
    pub(crate) fn new(
        connection_mode: TCPv4ConnectionMode,
        options: Option<&'a TCPv4Option>,
    ) -> Self {
        Self {
            type_of_service: 0,
            time_to_live: 255,
            access_point: TCPv4AccessPoint::new(connection_mode),
            option: options,
        }
    }
}

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

    receive_fn: extern "efiapi" fn(
        this: &Self,
        token: &UnmodelledPointer,
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
        bs: &BootServices,
        lifecycle: &Rc<RefCell<TCPv4ConnectionLifecycleManager>>,
    ) {
        unsafe {
            let lifecycle_clone = Rc::clone(&lifecycle);
            let event = ManagedEvent::new(bs, move |_e| {
                lifecycle_clone.borrow_mut().register_connecting_complete();
            });
            lifecycle.borrow_mut().register_started_connecting();
            let completion_token = TCPv4CompletionToken::new(event.event.unsafe_clone());
            (self.connect_fn)(
                &self,
                &completion_token,
            ).to_result().expect("Failed to call Connect()");
            bs.wait_for_event(&mut [event.event.unsafe_clone()]).expect("Failed to wait for connection to complete");
        }
    }

    pub fn transmit(
        &mut self,
        bs: &BootServices,
        lifecycle: &Rc<RefCell<TCPv4ConnectionLifecycleManager>>,
        data: &[u8],
    ) {
        let lifecycle_clone = Rc::clone(&lifecycle);
        unsafe {
            let event = ManagedEvent::new(bs, move |_e| {
                lifecycle_clone.borrow_mut().register_transmitting_complete();
            });
            lifecycle.borrow_mut().register_started_transmitting();

            let tx_data_handle = TCPv4TransmitDataHandle::new(data);
            let tx_data = tx_data_handle.get_data_ref();
            let io_token = TCPv4IoToken::new(event.event.unsafe_clone(), &tx_data);
            let result = (self.transmit_fn)(
                &self,
                &io_token,
            );
            info!("Transmit return value: {result:?}");

            bs.wait_for_event(&mut [event.event.unsafe_clone()]).expect("Failed to wait for transmit to complete");
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum TCPv4ConnectionOperation {
    Connecting,
    Transmitting,
}

#[derive(Debug)]
pub struct TCPv4ConnectionLifecycleManager {
    pending_operations: Vec<TCPv4ConnectionOperation>,
}

impl TCPv4ConnectionLifecycleManager {
    pub fn new() -> Self {
        Self {
            pending_operations: vec![],
        }
    }

    fn add_if_not_present(&mut self, op: TCPv4ConnectionOperation) {
        if !self.pending_operations.contains(&op) {
            self.pending_operations.push(op)
        }
    }

    fn remove_if_present(&mut self, op: TCPv4ConnectionOperation) {
        if self.pending_operations.contains(&op) {
            self.pending_operations.retain(|&x| x != op);
        }
    }

    pub fn register_started_connecting(&mut self) {
        self.add_if_not_present(TCPv4ConnectionOperation::Connecting);
    }

    pub fn register_connecting_complete(&mut self) {
        self.remove_if_present(TCPv4ConnectionOperation::Connecting);
        info!("Callback: connection completed!");
    }

    pub fn register_started_transmitting(&mut self) {
        self.add_if_not_present(TCPv4ConnectionOperation::Transmitting);
    }

    pub fn register_transmitting_complete(&mut self) {
        self.remove_if_present(TCPv4ConnectionOperation::Transmitting);
        info!("Callback: transmit completed!");
    }
}

#[repr(C)]
pub struct TCPv4IoToken<'a> {
    pub completion_token: TCPv4CompletionToken,
    packet: TCPv4Packet<'a>,
}

impl<'a> TCPv4IoToken<'a> {
    pub fn new(event: Event, tx: &'a TCPv4TransmitData) -> Self {
        Self {
            completion_token: TCPv4CompletionToken::new(event),
            packet: TCPv4Packet { tx_data: tx },
        }
    }
}

#[repr(C)]
union TCPv4Packet<'a> {
    rx_data: &'a TCPv4ReceiveData<'a>,
    tx_data: &'a TCPv4TransmitData,
}

#[derive(Debug)]
#[repr(C)]
pub struct TCPv4CompletionToken {
    pub event: Event,
    status: Status,
}

impl TCPv4CompletionToken {
    pub fn new(event: Event) -> Self {
        // PT: Replace in IO with MaybeUninit?
        Self {
            event,
            status: Status::SUCCESS,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct TCPv4FragmentData {
    fragment_length: u32,
    fragment_buf: *const c_void,
}

impl TCPv4FragmentData {
    fn new(data: &[u8]) -> Self {
        unsafe {
            let data_len = data.len();
            let layout = Layout::array::<u8>(data_len).unwrap();
            let buffer = alloc::alloc::alloc(layout);
            //info!("Allocated fragment {buffer:?} of size {data_len:?}");
            copy_nonoverlapping(
                data.as_ptr(),
                buffer,
                data_len,
            );
            Self {
                fragment_length: data_len as u32,
                fragment_buf: buffer as *const c_void,
            }
        }
    }
}

impl Drop for TCPv4FragmentData {
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::array::<u8>(self.fragment_length as usize).unwrap();
            alloc::alloc::dealloc(self.fragment_buf as *mut u8, layout);
            //println!("Deallocated fragment {:?}", self.fragment_buf);
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct TCPv4ReceiveData<'a> {
    urgent_flag: bool,
    data_length: u32,
    fragment_count: u32,
    fragment_table: &'a [TCPv4FragmentData],
}

/// This type is necessary because the underlying structure has a flexible array member.
/// Due to this, the memory for the instance needs to be carefully managed.
/// A Box cannot be used because the Box doesn't have the full knowledge of the layout.
/// A wide pointer also cannot be used because the layout needs to be precisely controlled for FFI.
/// Therefore, we use a wrapper 'handle' to manage the lifecycle of the allocation manually.
#[derive(Debug)]
#[repr(C)]
pub struct TCPv4TransmitDataHandle {
    ptr: *const TCPv4TransmitData,
    layout: Layout,
}

impl TCPv4TransmitDataHandle {
    fn total_layout_size(fragment_count: usize) -> usize {
        let size_of_fragments = mem::size_of::<ManuallyDrop<TCPv4FragmentData>>() * fragment_count;
        let ret = mem::size_of::<Self>() + size_of_fragments;
        info!("Total layout size: {ret}");
        ret
    }

    pub(crate) fn new(data: &[u8]) -> Self {
        let fragment = ManuallyDrop::new(TCPv4FragmentData::new(data));
        let layout = Layout::from_size_align(
            Self::total_layout_size(1),
            mem::align_of::<Self>(),
        ).unwrap();
        unsafe {
            let ptr = alloc::alloc::alloc(layout) as *mut TCPv4TransmitData;
            (*ptr).push = true;
            (*ptr).urgent = false;
            (*ptr).data_length = data.len() as _;

            let fragment_count = 1;
            (*ptr).fragment_count = fragment_count as _;
            copy_nonoverlapping(
                &fragment as *const _,
                (*ptr).fragment_table.as_mut_ptr(),
                fragment_count,
            );

            Self {
                ptr: ptr as _,
                layout,
            }
        }
    }

    fn get_data_ref(&self) -> &TCPv4TransmitData {
        // Safety: The reference is strictly tied to the lifetime of this handle
        unsafe { &*self.ptr }
    }
}

impl Drop for TCPv4TransmitDataHandle {
    fn drop(&mut self) {
        unsafe {
            info!("Dropping TX handle");

            let ptr = self.ptr as *mut TCPv4TransmitData;

            // First, drop all the fragments
            let fragment_table: *mut ManuallyDrop<TCPv4FragmentData> = (*ptr).fragment_table.as_mut_ptr();
            for i in 0..((*ptr).fragment_count as usize) {
                let fragment_ptr = fragment_table.add(i as _);
                ManuallyDrop::drop(&mut *fragment_ptr);
            }

            // Lastly, drop the allocation itself
            alloc::alloc::dealloc(ptr as *mut u8, self.layout);
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct TCPv4TransmitData {
    push: bool,
    urgent: bool,
    data_length: u32,
    fragment_count: u32,
    fragment_table: [ManuallyDrop<TCPv4FragmentData>; 0],
}

#[derive(Debug)]
#[repr(C)]
pub enum TCPv4ConnectionState {
    Closed = 0,
    Listen = 1,
    SynSent = 2,
    SynReceived = 3,
    Established = 4,
    FinWait1 = 5,
    FinWait2 = 6,
    Closing = 7,
    TimeWait = 8,
    CloseWait = 9,
    LastAck = 10,
}
