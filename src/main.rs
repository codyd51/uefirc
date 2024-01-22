#![no_main]
#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::mem;
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
pub struct TCPv4Protocol {

}

impl TCPv4Protocol {
    pub const GUID: Guid = guid!("65530BC7-A359-410F-B010-5AADC7EC2B62");
}
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(TCPv4Protocol::GUID)]
pub struct TCPv4(TCPv4Protocol);


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

    info!("Hello world!");
    //draw(system_table.boot_services()).unwrap();

    let bt = system_table.boot_services();
    //let handle = bt.get_handle_for_protocol::<Http>().unwrap();
    let handle = bt.get_handle_for_protocol::<TCPv4>();
    info!("Handle: {handle:?}");
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
    Status::SUCCESS
}
