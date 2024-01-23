use core::ffi::c_void;


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
// PT: Cannot use the type from uefi-rs because it's always 16 bytes, which messes up alignment in TCPv4AccessPoint
pub struct IPv4Address(pub [u8; 4]);

impl IPv4Address {
    pub fn new(b1: u8, b2: u8, b3: u8, b4: u8) -> Self {
        Self([b1, b2, b3, b4])
    }

    pub fn zero() -> Self {
        Self([0, 0, 0, 0])
    }

    pub fn subnet24() -> Self {
        Self([255, 255, 255, 0])
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct IPv4ModeData<'a> {
    is_started: bool,
    max_packet_size: u32,
    config_data: IPv4ConfigData,
    is_configured: bool,
    group_count: bool,
    group_table: &'a [IPv4Address; 0],
    route_count: u32,
    ip4_route_table: &'a [IPv4RouteTable; 0],
    icmp_type_count: u32,
    icmp_type_list: &'a [IPv4IcmpType; 0],
}

#[derive(Debug)]
#[repr(C)]
pub struct IPv4ConfigData {
    default_protocol: u8,
    accept_any_protocol: bool,
    accept_icmp_errors: bool,
    accept_broadcast: bool,
    accept_promiscuous: bool,
    use_default_address: bool,
    station_address: IPv4Address,
    subnet_mask: IPv4Address,
    type_of_service: u8,
    time_to_live: u8,
    do_not_fragment: bool,
    raw_data: bool,
    receive_timeout: u32,
    transmit_timeout: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct IPv4RouteTable {
    subnet_address: IPv4Address,
    subnet_mask: IPv4Address,
    gateway_address: IPv4Address,
}

#[derive(Debug)]
#[repr(C)]
pub struct IPv4IcmpType {
    _type: u8,
    code: u8,
}
