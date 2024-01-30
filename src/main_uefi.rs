#![no_main]

use alloc::rc::Rc;
use alloc::{format, vec};
use alloc::vec::Vec;
use agx_definitions::{Drawable, Point, Rect};
#[allow(dead_code)]

use agx_definitions::Size;
use libgui::AwmWindow;
use libgui::text_input_view::TextInputView;
use libgui::ui_elements::UIElement;
use log::info;
use uefi::prelude::*;
use uefi::proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput};
use uefi::table::boot::ScopedProtocol;
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

    // Disable the UEFI watchdog timer as we want to run indefinitely
    bs.set_watchdog_timer(
        0,
        0x1ffff,
        None,
    ).expect("Failed to disable watchdog timer");

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
    let font_arial = ttf_renderer::parse(&read_file(bs, "EFI\\Boot\\Chancery.ttf"));
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
        font_regular,
        font_arial,
        move |_v, superview_size| Rect::with_size(superview_size)
    );
    Rc::clone(&window).add_component(Rc::clone(&main_view) as Rc<dyn UIElement>);

    let mut irc_client = IrcClient::new(bs);
    {
        irc_client.connect_to_server();
        irc_client.set_nickname("phillip1-testing\r\nUSER phillip1-testing 0 * :phillip@axleos.com\r\n");
        //let data = format!("/USER {nickname} 0 * :{real_name}\r\n").into_bytes();
        //irc_client.set_user("phillip-testing", "phillip@axleos.com");
    }
    //Rc::clone(&conn).set_up_receive_signal_handler();
    {
        let conn = irc_client.active_connection.as_mut();
        let conn = conn.unwrap();
        Rc::clone(&conn).set_up_receive_signal_handler();
    }
    // Theory: we need to do the same careful stuff for transmit as for receive
    // To test, going to try to only set up the RX handler after doing our initial transmits
    loop {
        //irc_client.step();
        let mut active_connection = irc_client.active_connection.as_mut();
        let recv_buffer = &active_connection.expect("Expected an active connection").recv_buffer;
        let recv_data = recv_buffer.lock().borrow_mut().drain(..).collect::<Vec<u8>>();
        //println!("Got recv data");
        main_view.handle_recv_data(&recv_data);
        //println!("Got recv data");
        window.draw();
        render_window_to_display(&window, &mut graphics_protocol);
    }
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

fn render_window_to_display(
    window: &AwmWindow,
    graphics_protocol: &mut ScopedProtocol<GraphicsOutput>,
) {
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

    let resolution = window.frame().size;
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
