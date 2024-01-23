// DNS4: AE3D28CC-E05B-4FA1-A011-7EB55A3F1401 BDB49030
// UDP4: 3AD9DF29-4501-478D-B1F8-7F7FE70E50F3 BDB49D38
// IP4: 41D94CD2-35B6-455A-8258-D4E51334AADD BDB496A0
// TCP4: 65530BC7-A359-410F-B010-5AADC7EC2B62 BDB4CE38
// HTTP: 7A59B29B-910B-4171-8242-A85A0DF25B5B BDB4C020

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use core::alloc::Layout;
use core::cell::RefCell;
use core::ffi::c_void;
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::copy_nonoverlapping;
use log::info;
use uefi::{Error, Event, Handle, Status, StatusExt};
use uefi::prelude::BootServices;
use core::ptr::NonNull;

use uefi::table::boot::{EventType, Tpl};
use uefi::proto::unsafe_protocol;
use crate::ipv4::{IPv4Address, IPv4ModeData};

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

    pub fn connect(&mut self, bs: &BootServices, _lifecycle: &Rc<RefCell<TCPv4ConnectionLifecycleManager>>) {
        unsafe {
            let event = _create_event(bs, |_event| {
                info!("Callback: connection completed!");
            });
            let completion_token = TCPv4CompletionToken::new(event.unsafe_clone());
            //lifecycle.is_waiting_for_connect_to_complete = true;
            (self.connect_fn)(
                &self,
                &completion_token,
            ).to_result().expect("Failed to call Connect()");
            bs.wait_for_event(&mut [event.unsafe_clone()]).expect("Failed to wait for connection to complete");
            //info!("Finished waiting for event!");
            bs.close_event(event).expect("Failed to close event");
        }
    }

    pub fn transmit(
        &mut self,
        bs: &BootServices,
        _lifecycle: &Rc<RefCell<TCPv4ConnectionLifecycleManager>>,
        data: &[u8],
    ) {
        unsafe {
            let event = _create_event(
                &bs,
                |_e|{
                    info!("Callback: transmit completed!")
                }
            );
            //lifecycle.is_waiting_for_transmit_to_complete = true;
            let tx_data = TCPv4TransmitData::new(data);
            let io_token = TCPv4IoToken::new(event.unsafe_clone(), tx_data);
            let result = (self.transmit_fn)(
                &self,
                &io_token,
            );
            info!("Transmit return value: {result:?}");

            bs.wait_for_event(&mut [event.unsafe_clone()]).expect("Failed to wait for transmit to complete");
            bs.close_event(event).expect("Failed to close event");
        }
    }
}

fn _create_event<F>(
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
            Some(_call_closure::<F>),
            Some(NonNull::new(data as *mut _ as *mut c_void).unwrap()),
        ).expect("Failed to create event")
    }
}

unsafe extern "efiapi" fn _call_closure<F>(
    event: Event,
    raw_context: Option<NonNull<c_void>>,
)
    where
        F: FnMut(Event) + 'static {
    let unwrapped_context = cast_ctx(raw_context);
    let callback_ptr = unwrapped_context as *mut F;
    let callback = &mut *callback_ptr;
    callback(event);
    // Drop the box carrying the closure
    let _ = Box::from_raw(unwrapped_context as *mut _);
}

unsafe fn cast_ctx<T>(raw_val: Option<core::ptr::NonNull<c_void>>) -> &'static mut T {
    let val_ptr = raw_val.unwrap().as_ptr() as *mut c_void as *mut T;
    &mut *val_ptr
}

#[derive(Debug)]
pub struct TCPv4ConnectionLifecycleManager {
    is_waiting_for_connect_to_complete: bool,
    is_waiting_for_transmit_to_complete: bool,
}

impl TCPv4ConnectionLifecycleManager {
    pub fn new() -> Self {
        Self {
            is_waiting_for_connect_to_complete: false,
            is_waiting_for_transmit_to_complete: false,
        }
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
            let f = self.fragment_buf;
            let l = self.fragment_length;
            info!("Drop TCPv4FragmentData {self:?} {f:?} {l:?}");
            let layout = Layout::array::<u8>(self.fragment_length as usize).unwrap();
            alloc::alloc::dealloc(self.fragment_buf as *mut u8, layout);
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

#[derive(Debug)]
#[repr(C)]
pub struct TCPv4TransmitData {
    push: bool,
    urgent: bool,
    data_length: u32,
    fragment_count: u32,
    fragment_table: [ManuallyDrop<TCPv4FragmentData>; 0],
}

impl TCPv4TransmitData {
    fn total_layout_size(fragment_count: usize) -> usize {
        let size_of_fragments = mem::size_of::<TCPv4FragmentData>() * fragment_count;
        mem::size_of::<Self>() + size_of_fragments
    }

    pub(crate) fn new(data: &[u8]) -> &Self {
        let fragment = ManuallyDrop::new(TCPv4FragmentData::new(data));
        let layout = Layout::from_size_align(
            Self::total_layout_size(1),
            mem::align_of::<Self>(),
        ).unwrap();
        unsafe {
            let ptr = alloc::alloc::alloc(layout) as *mut Self;
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
            &*ptr
        }
    }
}

impl Drop for TCPv4TransmitData {
    fn drop(&mut self) {
        info!("Dropping TCPv4TransmitData {self:?}");

        let ptr = self as *mut Self;
        unsafe {
            // First, drop all the fragments
            let fragment_table: *mut ManuallyDrop<TCPv4FragmentData> = (*ptr).fragment_table.as_mut_ptr();
            for i in 0..self.fragment_count {
                let fragment_ptr = fragment_table.add(i as _);
                let fragment = &mut *fragment_ptr;
                info!("Freeing fragment #{i}: {fragment:?}");
                let b = fragment.fragment_buf;
                let l = fragment.fragment_length;
                info!("Fragment has buf fragment {b:?} of size {l:?}");
                ManuallyDrop::drop(fragment);
            }

            // Finally, drop this allocation
            let layout = Layout::from_size_align(
                Self::total_layout_size(self.fragment_count as _),
                mem::align_of::<Self>(),
            ).unwrap();
            alloc::alloc::dealloc(ptr as *mut u8, layout);
        }
    }
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
