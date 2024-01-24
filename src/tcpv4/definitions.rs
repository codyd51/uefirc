use core::alloc::Layout;
use core::ffi::c_void;
use core::ptr::copy_nonoverlapping;
use uefi::{Event, Status};

use crate::ipv4::IPv4Address;
use crate::tcpv4::TCPv4TransmitData;

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
    pub fn new(data: &[u8]) -> Self {
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
