#![no_main]
#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::mem;
use log::info;
use uefi::prelude::*;
use uefi::Result;
use uefi::proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput};
use uefi::proto::rng::Rng;
use uefi::table::boot::{OpenProtocolAttributes, OpenProtocolParams};

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
    draw(system_table.boot_services()).unwrap();
    Status::SUCCESS
}
