#![no_main]
#![no_std]
#![feature(rustc_private)]
#[allow(dead_code)]

mod tcpv4;
mod ipv4;
mod event;
mod connection;
mod ui;
mod app;
mod gui;
mod fs;

extern crate alloc;

use agx_definitions::Size;
use log::info;
use uefi::prelude::*;
use uefi_services::println;
use crate::app::IrcClient;
use crate::fs::read_file;
use crate::gui::Screen;
use crate::ui::set_resolution;

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    let bs = system_table.boot_services();
    let bs: &'static BootServices = unsafe {
        core::mem::transmute(bs)
    };

    info!("Parsing fonts...");
    let font_regular = ttf_renderer::parse(&read_file(bs, "EFI\\Boot\\sf_pro.ttf"));
    let font_italic = ttf_renderer::parse(&read_file(bs, "EFI\\Boot\\new_york_italic.ttf"));
    info!("All done!");

    let resolution = Size::new(1920, 1080);
    let graphics_protocol = set_resolution(
        bs,
        resolution,
    ).unwrap();

    let screen = Screen::new(
        resolution,
        graphics_protocol,
        font_regular,
        font_italic,
    );
    screen.enter_event_loop();

    /*
    let mut client = IrcClient::new(bs);
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
