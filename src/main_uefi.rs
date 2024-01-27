#![no_main]

use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use agx_definitions::{Point, Rect};
#[allow(dead_code)]

use agx_definitions::Size;
use libgui::AwmWindow;
use libgui::text_input_view::TextInputView;
use libgui::ui_elements::UIElement;
use log::info;
use uefi::prelude::*;
use uefi::proto::console::gop::{BltOp, BltPixel, BltRegion};
use uefi_services::println;
use crate::app::IrcClient;
use crate::fs::read_file;
use crate::gui::MainView;
//use crate::gui::Screen;
use crate::ui::set_resolution;

pub fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    let bs = system_table.boot_services();
    let bs: &'static BootServices = unsafe {
        core::mem::transmute(bs)
    };

    info!("Parsing fonts...");
    //let font_regular = ttf_renderer::parse(&read_file(bs, "EFI\\Boot\\sf_pro.ttf"));
    /*
    Nice:
    Bodoni
    DIN
    BigCaslon
    Chancery
     */
    let font_regular = ttf_renderer::parse(&read_file(bs, "EFI\\Boot\\BigCaslon.ttf"));
    let font_italic = ttf_renderer::parse(&read_file(bs, "EFI\\Boot\\chancery.ttf"));
    info!("All done!");

    //let resolution = Size::new(1920, 1080);
    let resolution = Size::new(1360, 768);
    let mut graphics_protocol = set_resolution(
        bs,
        resolution,
    ).unwrap();

    let window = AwmWindow::new(resolution);
    let main_view = MainView::new(
        move |_v, superview_size| Rect::with_size(superview_size)
    );
    Rc::clone(&window).add_component(Rc::clone(&main_view) as Rc<dyn UIElement>);

    loop {
        window.draw();
        let layer = window.layer.borrow_mut();
        let pixel_buffer = layer.framebuffer.borrow_mut();

        let buf_as_u32 = {
            let buf_as_u8 = pixel_buffer;
            let len = buf_as_u8.len() / 4;
            let capacity = len;

            let raw_parts = buf_as_u8.as_ptr() as *mut u32;
            let buf_as_u32 = unsafe { Vec::from_raw_parts(raw_parts, len, capacity) };
            buf_as_u32
        };

        let mut pixels: Vec<BltPixel> = vec![];
        for px in buf_as_u32.iter() {
            let bytes = px.to_le_bytes();
            pixels.push(
                BltPixel::new(
                    bytes[2],
                    bytes[1],
                    bytes[0],
                )
            );
        }

        graphics_protocol.blt(
            BltOp::BufferToVideo {
                buffer: &pixels,
                src: BltRegion::Full,
                dest: (0, 0),
                dims: (resolution.width as _, resolution.height as _),
            }
        ).expect("Failed to blit screen");

        // Don't free the memory once done as it's owned by the pixel buffer
        core::mem::forget(buf_as_u32);
    }
    //let irc_client = IrcClient::new(bs);
    /*
    let screen = Screen::new(
        resolution,
        graphics_protocol,
        font_regular,
        font_italic,
        irc_client,
    );
    loop {
        screen.step();
    }

     */

    /*
    loop {
        client.step();
    }
    */

    /*
    connection.transmit(b"NICK phillip-testing\r\n");
    connection.transmit(b"USER phillip-testing O * :phillip@axleos.com\r\n");
    loop {
        connection.step();
    }

     */
    loop{}
}
