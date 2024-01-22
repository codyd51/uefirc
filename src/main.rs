#![no_main]
#![no_std]

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
use uefi::table::boot::{OpenProtocolAttributes, OpenProtocolParams};
use uefi::proto::unsafe_protocol;

#[derive(Debug)]
#[repr(C)]
pub struct HttpServiceBindingProtocol {
    /*
    pub revision: u64,
    pub media: *const BlockIoMedia,
    pub reset: unsafe extern "efiapi" fn(this: *mut Self, extended_verification: bool) -> Status,
    pub read_blocks: unsafe extern "efiapi" fn(
        this: *const Self,
        media_id: u32,
        lba: Lba,
        buffer_size: usize,
        buffer: *mut c_void,
    ) -> Status,
    pub write_blocks: unsafe extern "efiapi" fn(
        this: *mut Self,
        media_id: u32,
        lba: Lba,
        buffer_size: usize,
        buffer: *const c_void,
    ) -> Status,
    pub flush_blocks: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
    */
}

impl HttpServiceBindingProtocol {
    pub const GUID: Guid = guid!("bdc8e6af-d9bc-4379-a72a-e0c4e75dae1c");
}

#[derive(Debug)]
#[repr(C)]
pub struct HttpProtocol {
    pub get_mode_data: u64,
    pub configure: u64,
    pub request: u64,
    pub cancel: u64,
    pub response: u64,
    pub poll: u64,
}

impl HttpProtocol {
    pub const GUID: Guid = guid!("7A59B29B-910B-4171-8242-A85A0DF25B5B");
}


#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(HttpServiceBindingProtocol::GUID)]
pub struct HttpBinding(HttpServiceBindingProtocol);

#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(HttpProtocol::GUID)]
pub struct Http(HttpProtocol);

#[derive(Debug)]
#[repr(C)]
pub struct IPv4Protocol {
}

impl IPv4Protocol {
    //pub const GUID: Guid = guid!("00720665-67EB-4a99-BAF7-D3C33A1C7CC9");
    //pub const GUID: Guid = guid!("65530BC7-A359-410f-B010-5AADC7EC2B62");
    //pub const GUID: Guid = guid!("41d94cd2-35b6-455a-8258-d4e51334aadd");
    pub const GUID: Guid = guid!("41d94cd2-35b6-455a-8258-d4e51334aadd");
}
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(IPv4Protocol::GUID)]
pub struct IPv4(IPv4Protocol);

// DNS4: AE3D28CC-E05B-4FA1-A011-7EB55A3F1401 BDB49030
// UDP4: 3AD9DF29-4501-478D-B1F8-7F7FE70E50F3 BDB49D38
// IP4: 41D94CD2-35B6-455A-8258-D4E51334AADD BDB496A0
// TCP4: 65530BC7-A359-410F-B010-5AADC7EC2B62 BDB4CE38
// HTTP: 7A59B29B-910B-4171-8242-A85A0DF25B5B BDB4C020

#[derive(Debug)]
#[repr(C)]
pub struct UnmodelledPointer(pub *mut c_void);


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C, align(4))]
// PT: Cannot use the type from uefi-rs because it's always 16 bytes, which messes up alignment in TCPv4AccessPoint
pub struct IPv4Address(pub [u8; 4]);

impl IPv4Address {
    fn new(b1: u8, b2: u8, b3: u8, b4: u8) -> Self {
        Self([b1, b2, b3, b4])
    }

    fn zero() -> Self {
        Self([0, 0, 0, 0])
    }

    fn subnet24() -> Self {
        Self([255, 255, 255, 0])
    }
}

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
    fn new() -> Self {
        Self {
            use_default_address: true,
            // These two fields are meaningless because we set use_default_address above
            //station_address: IPv4Address::new(192, 168, 0, 3),
            //subnet_mask: IPv4Address::subnet24(),
            //station_address: IPv4Address::zero(),
            //subnet_mask: IPv4Address::zero(),
            /*
            station_address: IPv4Address::new(192, 169, 0, 3),
            subnet_mask: IPv4Address::new(255, 255, 0, 0),
            station_port: 0,
            remote_address: IPv4Address::new(192, 169, 0, 1),
            remote_port: 80,
            active_flag: true,
             */
            station_address: IPv4Address::zero(),
            subnet_mask: IPv4Address::zero(),
            station_port: 1234,
            remote_address: IPv4Address::zero(),
            //remote_address: IPv4Address::new(192, 169, 0, 1),
            //remote_address: IPv4Address::new(1, 0, 169, 192),
            remote_port: 0,
            active_flag: true,

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

impl TCPv4Option {
    fn new() -> Self {
        Self {
            /*
            receive_buffer_size: 32 * 1024,
            send_buffer_size: 32 * 1024,
            max_syn_back_log: 128,
            connection_timeout: 20_000,
            data_retries: 10,
            fin_timeout: 60_000,
            time_wait_timeout: 120_000,
            keep_alive_probes: 9,
            keep_alive_time: 7_200_000,
            keep_alive_interval: 75_000,
            enable_nagle: true,
            enable_time_stamp: true,
            enable_window_scaling: true,
            enable_selective_ack: true,
            enable_path_mtu_discovery: true,

             */
            receive_buffer_size: 1024,
            send_buffer_size: 1024,
            max_syn_back_log: 0,
            connection_timeout: 0,
            data_retries: 0,
            fin_timeout: 0,
            time_wait_timeout: 3,
            keep_alive_probes: 0,
            keep_alive_time: 0,
            keep_alive_interval: 0,
            enable_nagle: false,
            enable_time_stamp: false,
            enable_window_scaling: false,
            enable_selective_ack: false,
            enable_path_mtu_discovery: false,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct TCPv4ConfigData<'a> {
    type_of_service: u8,
    time_to_live: u8,
    access_point: TCPv4AccessPoint,
    option: Option<&'a TCPv4Option>,
}

impl<'a> TCPv4ConfigData<'a> {
    fn new(options: Option<&'a TCPv4Option>) -> Self {
        Self {
            // Standard values
            type_of_service: 0,
            time_to_live: 255,
            access_point: TCPv4AccessPoint::new(),
            option: options,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
#[unsafe_protocol("00720665-67EB-4a99-BAF7-D3C33A1C7CC9")]
pub struct TCPv4ServiceBindingProtocol {
    create_child: extern "efiapi" fn(
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
    get_mode_data: extern "efiapi" fn(
        this: &Self,
        out_connection_state: &mut UnmodelledPointer,
        out_config_data: &mut UnmodelledPointer,
        out_ip4_mode_data: &mut UnmodelledPointer,
        out_managed_network_config_data: &mut UnmodelledPointer,
        out_simple_network_mode: &mut UnmodelledPointer,
    ) -> Status,

    configure: extern "efiapi" fn(
        this: &Self,
        config_data: Option<&TCPv4ConfigData>,
    ) -> Status,

    routes: extern "efiapi" fn(
        this: &Self,
        delete_route: bool,
        subnet_address: &IPv4Address,
        subnet_mask: &IPv4Address,
        gateway_address: &IPv4Address,
    ) -> Status,

    connect: extern "efiapi" fn(
        this: &Self,
        connection_token: &UnmodelledPointer,
    ) -> Status,

    accept: extern "efiapi" fn(
        this: &Self,
        listen_token: &UnmodelledPointer,
    ) -> Status,

    transmit: extern "efiapi" fn(
        this: &Self,
        token: &UnmodelledPointer,
    ) -> Status,

    receive: extern "efiapi" fn(
        this: &Self,
        token: &UnmodelledPointer,
    ) -> Status,

    close: extern "efiapi" fn(
        this: &Self,
        close_token: &UnmodelledPointer,
    ) -> Status,

    cancel: extern "efiapi" fn(
        this: &Self,
        completion_token: &UnmodelledPointer,
    ) -> Status,

    poll: extern "efiapi" fn(this: &Self) -> Status,
}

struct Buffer {
    width: usize,
    height: usize,
    pixels: Vec<BltPixel>,
}

impl Buffer {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![BltPixel::new(0, 0, 0); width * height],
        }
    }

    fn pixel(&mut self, x: usize, y: usize) -> Option<&mut BltPixel> {
        self.pixels.get_mut(y * self.width + x)
    }

    fn blit(&self, gop: &mut GraphicsOutput) -> Result {
        gop.blt(
            BltOp::BufferToVideo {
                buffer: &self.pixels,
                src: BltRegion::Full,
                dest: (0, 0),
                dims: (self.width, self.height),
            }
        )
    }
}

fn rand_usize(rng: &mut Rng) -> usize {
    let mut buf = [0; mem::size_of::<usize>()];
    rng.get_rng(None, &mut buf).expect("get_rng failed");
    usize::from_le_bytes(buf)
}

fn draw(bt: &BootServices) -> Result {
    let gop_handle = bt.get_handle_for_protocol::<GraphicsOutput>().unwrap();
    let mut gop = bt.open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();

    let rng_handle = bt.get_handle_for_protocol::<Rng>()?;
    let mut rng = bt.open_protocol_exclusive::<Rng>(rng_handle)?;

    let (width, height) = gop.current_mode_info().resolution();
    let mut buf = Buffer::new(width, height);

    let mut r_mod = ((rand_usize(&mut rng)%255) as f32);
    let mut g_mod = ((rand_usize(&mut rng)%255) as f32);
    loop {
        for y in 0..height {
            let r = ((y as f32) / ((height - 1) as f32)) * r_mod;
            for x in 0..width {
                let g = ((x as f32) / ((width - 1) as f32)) * g_mod;
                let px = buf.pixel(x, y).unwrap();
                px.red = r as u8;
                px.green = g as u8;
                px.blue = 255;
            }
        }

        buf.blit(&mut gop).unwrap();
        bt.stall(1000);
        r_mod += 10.0;
        r_mod %= 255.0;
        g_mod += 10.0;
        g_mod %= 255.0;
    }
    Result::Ok(())
}

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    //draw(system_table.boot_services()).unwrap();

    let bt = system_table.boot_services();

    let tcp_service_binding_handle = bt.get_handle_for_protocol::<TCPv4ServiceBindingProtocol>().unwrap();
    let tcp_service_binding = unsafe {
        bt.open_protocol::<TCPv4ServiceBindingProtocol>(
            OpenProtocolParams {
                handle: tcp_service_binding_handle,
                agent: bt.image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        ).unwrap()
    };
    info!("Got TCP service binding handle {tcp_service_binding_handle:?}, {tcp_service_binding:?}");

    let mut tcp_service_handle = core::mem::MaybeUninit::<Handle>::uninit();
    let mut tcp_service_handle_ptr = tcp_service_handle.as_mut_ptr();
    unsafe {
        (tcp_service_binding.create_child)(
            &tcp_service_binding,
            &mut *tcp_service_handle_ptr,
        ).to_result().unwrap()
    };
    let tcp_service_handle = unsafe { tcp_service_handle.assume_init() };
    info!("Got TCP service handle {tcp_service_handle:?}");

    //let handle = bt.get_handle_for_protocol::<Http>().unwrap();
    /*
    let handle = bt.get_handle_for_protocol::<TCPv4Protocol>();
    info!("Handle: {handle:?}");
    let handle = handle.unwrap();
     */
    let handle = tcp_service_handle;

    //let mut tcp = bt.open_protocol_exclusive::<TCPv4Protocol>(handle).unwrap();
    let tcp = unsafe {
        bt.open_protocol::<TCPv4Protocol>(
            OpenProtocolParams {
                handle,
                agent: bt.image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        )
    }.unwrap();

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

    /*
    let mut connection_state = UnmodelledPointer(0 as *mut c_void);
    let mut config_data = UnmodelledPointer(0 as *mut c_void);
    let mut ip4_mode_data = UnmodelledPointer(0 as *mut c_void);
    let mut managed_network_config_data = UnmodelledPointer(0 as *mut c_void);
    let mut simple_network_mode = UnmodelledPointer(0 as *mut c_void);
    let result = (tcp.get_mode_data)(
        &tcp,
        &mut connection_state,
        &mut config_data,
        &mut ip4_mode_data,
        &mut managed_network_config_data,
        &mut simple_network_mode,
    ).to_result();
    */

    /*
    info!("Did get mode data!");
    info!("Result {result:?}");
    info!("connection_state {connection_state:?}");
    info!("config_data {config_data:?}");
    info!("ip4_mode_data {ip4_mode_data:?}");
    info!("managed_network_config_data {managed_network_config_data:?}");
    info!("simple_network_mode {simple_network_mode:?}");

     */
    /*
    get_mode_data: extern "efiapi" fn(
        this: &Self,
        out_connection_state: &mut UnmodelledPointer,
        out_config_data: &mut UnmodelledPointer,
        out_ip4_mode_data: &mut UnmodelledPointer,
        out_managed_network_config_data: &mut UnmodelledPointer,
        out_simple_network_mode: &mut UnmodelledPointer,
    ) -> Status,

     */
    /*
    info!("Got handle {handle:?}");
    //let mut gop = bt.open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();
    let h = unsafe {
        bt.open_protocol::<Http>(
            OpenProtocolParams {
                handle,
                agent: bt.image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::Exclusive,
        )
    }.unwrap();
    info!("Got h {h:?}");
    */
    bt.stall(1_000_000);
    Status::SUCCESS
}
