#![no_main]

use alloc::rc::Rc;
use alloc::vec::Vec;
use agx_definitions::{Drawable, Rect};
#[allow(dead_code)]

use agx_definitions::Size;
use libgui::{AwmWindow, KeyCode};
use libgui::ui_elements::UIElement;
use log::info;
use uefi::prelude::*;
use uefi::proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput};
use uefi::proto::console::text::Key;
use uefi::table::boot::ScopedProtocol;
use crate::app::IrcClient;
use crate::fs::read_file;
use crate::gui::MainView;
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
        irc_client.set_nickname("phillip-testing\r\nUSER phillip-testing 0 * :phillip@axleos.com\r\n");
        //let data = format!("/USER {nickname} 0 * :{real_name}\r\n").into_bytes();
        //irc_client.set_user("phillip-testing", "phillip@axleos.com");
    }
    {
        let conn = irc_client.active_connection.as_mut();
        let conn = conn.unwrap();
        Rc::clone(&conn).set_up_receive_signal_handler();
    }
    // Theory: we need to do the same careful stuff for transmit as for receive
    // To test, going to try to only set up the RX handler after doing our initial transmits

    let mut currently_held_key: Option<KeyCode> = None;
    loop {
        irc_client.step();
        let mut active_connection = irc_client.active_connection.as_mut();
        let recv_buffer = &active_connection.expect("Expected an active connection").recv_buffer;
        let recv_data = recv_buffer.lock().borrow_mut().drain(..).collect::<Vec<u8>>();
        //println!("Got recv data");
        main_view.handle_recv_data(&recv_data);
        //println!("Got recv data");
        let key_held_on_this_iteration = {
            let maybe_key = system_table.stdin().read_key().expect("Failed to poll for a key");
            match maybe_key {
                None => None,
                Some(key) => {
                    let key_as_u16 = match key {
                        Key::Special(scancode) => {
                            scancode.0
                        }
                        Key::Printable(char_u16) => {
                            char::from(char_u16) as _
                        }
                    };
                    Some(KeyCode(key_as_u16 as _))
                }
            }
        };

        // Are we changing state in any way?
        //println!("Got key {key_held_on_this_iteration:?}");
        if key_held_on_this_iteration != currently_held_key {
            // Are we switching away from a held key?
            if currently_held_key.is_some() {
                window.handle_key_released(currently_held_key.unwrap());
            }
            if key_held_on_this_iteration.is_some() {
                // Inform the window that a new key is held
                window.handle_key_pressed(key_held_on_this_iteration.unwrap());
            }
            // And update our state to track that this key is currently held
            currently_held_key = key_held_on_this_iteration;
        }

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

    let buf_as_blt_pixel = unsafe {
        let buf_as_u8 = pixel_buffer;
        let len = buf_as_u8.len() / 4;
        let capacity = len;

        let buf_as_blt_pixels = buf_as_u8.as_ptr() as *mut BltPixel;
        Vec::from_raw_parts(
            buf_as_blt_pixels,
            len,
            capacity,
        )
    };
    // Immediately forget our re-interpreted vector of pixel data, as it's really owned by the window
    core::mem::forget(buf_as_blt_pixel);

    let resolution = window.frame().size;
    graphics_protocol.blt(
        BltOp::BufferToVideo {
            buffer: &buf_as_blt_pixel,
            src: BltRegion::Full,
            dest: (0, 0),
            dims: (resolution.width as _, resolution.height as _),
        }
    ).expect("Failed to blit screen");
}
