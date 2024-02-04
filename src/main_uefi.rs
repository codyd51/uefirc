#![no_main]

use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::cmp::{max, min};
use agx_definitions::{Color, Drawable, NestedLayerSlice, Point, Rect, StrokeThickness};
#[allow(dead_code)]

use agx_definitions::Size;
use libgui::{AwmWindow, KeyCode};
use libgui::button::Button;
use libgui::ui_elements::UIElement;
use log::info;
use ttf_renderer::{Font, rendered_string_size};
use uefi::prelude::*;
use uefi::proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput};
use uefi::proto::console::pointer::Pointer;
use uefi::proto::console::text::Key;
use uefi::table::boot::ScopedProtocol;
use crate::app::IrcClient;
use crate::fs::read_file;
use crate::gui::{ContentView, InputBoxView, TitleView};
use crate::irc::{IrcCommand, IrcCommandName, IrcMessage, ResponseParser};
use crate::ui::set_resolution;

#[derive(Debug, Copy, Clone)]
struct RenderStructuredMessageAttributes<'a> {
    leading_text: &'a str,
    leading_text_color: Color,
    leading_text_background_color: Color,
    leading_text_background_border_color: Color,

    main_text: &'a str,
    main_text_color: Color,
    main_text_background_color: Color,
    main_text_background_border_color: Color,
}

impl<'a> RenderStructuredMessageAttributes<'a> {
    fn new(
        leading_text: &'a str,
        leading_text_color: Color,
        leading_text_background_color: Color,
        leading_text_background_border_color: Color,
        main_text: &'a str,
        main_text_color: Color,
        main_text_background_color: Color,
        main_text_background_border_color: Color,
    ) -> Self {
        Self {
            leading_text,
            leading_text_color,
            leading_text_background_color,
            leading_text_background_border_color,
            main_text,
            main_text_color,
            main_text_background_color,
            main_text_background_border_color,
        }
    }
}

struct App<'a> {
    irc_client: RefCell<IrcClient<'a>>,
    font_regular: Font,
    font_italic: Font,
    window: Rc<AwmWindow>,
    content_view: Rc<ContentView>,
    input_box_view: Rc<InputBoxView>,
    currently_held_key: RefCell<Option<KeyCode>>,
    current_pointer_pos: RefCell<Point>,
    cursor_size: Size,
    pointer_resolution: Point,
    is_left_click_down: RefCell<bool>,
    response_parser: RefCell<ResponseParser>,
    is_user_manually_scrolling: RefCell<bool>,
}

impl<'a> App<'a> {
    fn new(
        resolution: Size,
        font_regular: Font,
        font_italic: Font,
        pointer_resolution: Point,
        irc_client: IrcClient<'a>,
    ) -> Rc<Self> {
        let window = AwmWindow::new(resolution);
        let title_sizer = |superview_size: Size| {
            Rect::with_size(
                Size::new(
                    superview_size.width,
                    (superview_size.height as f64 * 0.084) as _,
                )
            )
        };

        let title_sizer_clone = title_sizer.clone();
        let content_sizer = move |superview_size: Size| {
            let title_frame = title_sizer_clone(superview_size);
            Rect::from_parts(
                Point::new(0, title_frame.max_y()),
                Size::new(
                    superview_size.width,
                    (superview_size.height as f64 * 0.82) as _,
                )
            )
        };

        let content_sizer_clone = content_sizer.clone();
        let input_box_sizer = move |superview_size: Size| {
            let content_frame = content_sizer_clone(superview_size);
            Rect::from_parts(
                Point::new(
                    0,
                    content_frame.max_y(),
                ),
                Size::new(
                    (superview_size.width as f64 * 0.9) as _,
                    (superview_size.height as f64 * 0.1) as _,
                )
            )
        };

        let input_box_sizer_clone = input_box_sizer.clone();
        let send_button_sizer = move |superview_size: Size| {
            let input_box_frame = input_box_sizer_clone(superview_size);
            Rect::from_parts(
                Point::new(
                    input_box_frame.max_x(),
                    input_box_frame.min_y(),
                ),
                Size::new(
                    superview_size.width - input_box_frame.width(),
                    input_box_frame.height(),
                )
            )
        };

        let title = TitleView::new(
            font_regular.clone(),
            Size::new(32, 32),
            move |v, s| title_sizer(s),
        );

        let content = ContentView::new(
            font_regular.clone(),
            Size::new(20, 20),
            move |v, s| content_sizer(s),
        );

        let input_box = InputBoxView::new(
            font_regular.clone(),
            Size::new(24, 24),
            move |v, s| input_box_sizer(s),
        );

        let send_button = Button::new(
            "Send",
            font_regular.clone(),
            move |v, s| send_button_sizer(s),
        );

        Rc::clone(&window).add_component(Rc::clone(&title) as Rc<dyn UIElement>);
        Rc::clone(&window).add_component(Rc::clone(&content) as Rc<dyn UIElement>);
        Rc::clone(&window).add_component(Rc::clone(&input_box) as Rc<dyn UIElement>);
        Rc::clone(&window).add_component(Rc::clone(&send_button) as Rc<dyn UIElement>);

        let _self = Rc::new(
            Self {
                irc_client: RefCell::new(irc_client),
                font_regular,
                font_italic,
                window,
                content_view: content,
                input_box_view: Rc::clone(&input_box),
                currently_held_key: RefCell::new(None),
                // Start off the mouse in the middle of the screen
                current_pointer_pos: RefCell::new(Point::new(resolution.mid_x(), resolution.mid_y())),
                cursor_size: Size::new(15, 15),
                pointer_resolution,
                is_left_click_down: RefCell::new(false),
                response_parser: RefCell::new(ResponseParser::new()),
                is_user_manually_scrolling: RefCell::new(false),
            }
        );

        let self_clone_for_button_cb: Rc<App<'static>> = unsafe { core::mem::transmute(Rc::clone(&_self)) };
        send_button.on_left_click(move |b|{
            self_clone_for_button_cb.send_input_and_clear_input_text_box();
        });

        let self_clone_for_input_box_cb: Rc<App<'static>> = unsafe { core::mem::transmute(Rc::clone(&_self)) };
        input_box.view.set_on_key_pressed(move |v, key_code|{
            // PT: UEFI represents the enter key as a carriage return rather than newline
            if key_code.0 as u8 == '\r' as u8 {
                Rc::clone(&self_clone_for_input_box_cb).handle_enter_key_pressed();
            }
        });

        _self
    }

    fn scroll_to_last_visible_line(&self) {
        // Auto-scroll to the last visible message
        let cursor_pos = self.content_view.view.cursor_pos.borrow().1;
        let viewport_height = self.content_view.frame().height();
        *self.content_view.view.view.layer.scroll_offset.borrow_mut() = Point::new(
            0,
            cursor_pos.y - viewport_height + 40,
        );
    }

    fn scroll_up(&self) {
        let scroll_pos = *self.content_view.view.view.layer.scroll_offset.borrow();
        *self.content_view.view.view.layer.scroll_offset.borrow_mut() = Point::new(
            scroll_pos.x,
            scroll_pos.y - 30,
        );
        *self.is_user_manually_scrolling.borrow_mut() = true;
    }

    fn scroll_down(&self) {
        let scroll_pos = *self.content_view.view.view.layer.scroll_offset.borrow();
        *self.content_view.view.view.layer.scroll_offset.borrow_mut() = Point::new(
            scroll_pos.x,
            scroll_pos.y + 30,
        );
        // TODO(PT): Unset this if we've scrolled to the bottom
        *self.is_user_manually_scrolling.borrow_mut() = true;
    }

    fn write_string(&self, s: &str) {
        self.content_view.view.draw_string(s, Color::black());
        self.scroll_to_last_visible_line();
    }

    pub fn handle_recv_data(&self, recv_data: &[u8]) {
        let recv_as_str = core::str::from_utf8(recv_data).unwrap();
        self.write_string(recv_as_str);
    }

    fn render_structured_message_with_attributes(
        &self,
        attributes: RenderStructuredMessageAttributes,
    ) {
        let text_view = &self.content_view.view;
        let scroll_view = &self.content_view.view.view;

        // TODO(PT): Share this with the content view?
        let font_size = Size::new(20, 20);
        let leading_right_side_padding_px = 10;
        let rendered_leading_text_size = rendered_string_size(
            attributes.leading_text,
            &self.font_italic,
            font_size,
        );

        // TODO(PT): This does not take into account extra height induced by line breaks
        let rendered_message_text_size = rendered_string_size(
            attributes.main_text,
            &self.font_regular,
            font_size,
        );

        // The background rectangles should take the larger size of the rendered LHS or RHS
        let background_rect_height = max(rendered_leading_text_size.height, rendered_message_text_size.height);

        let initial_cursor = text_view.cursor_pos();
        let start_of_message_content_x = rendered_leading_text_size.width + leading_right_side_padding_px;
        let leading_text_background_frame = Rect::from_parts(
            initial_cursor.1,
            Size::new(
                start_of_message_content_x,
                background_rect_height,
            ),
        );
        // Background of leading text
        scroll_view.get_slice().fill_rect(
            leading_text_background_frame,
            attributes.leading_text_background_color,
            StrokeThickness::Filled,
        );
        // Background border of leading text
        scroll_view.get_slice().fill_rect(
            leading_text_background_frame,
            attributes.leading_text_background_border_color,
            StrokeThickness::Width(1),
        );

        text_view.draw_string_with_font(
            attributes.leading_text,
            &self.font_italic,
            font_size,
            attributes.leading_text_color,
        );
        let mut cursor = text_view.cursor_pos();
        let message_left_side_padding_x = 6;
        cursor.1.x = start_of_message_content_x + message_left_side_padding_x;
        text_view.set_cursor_pos(cursor);

        // Draw the background for the message itself
        // TODO(PT): What about when we need to break to a new line..?
        let message_line_size = Size::new(
            text_view.frame().size.width - start_of_message_content_x,
            background_rect_height,
        );
        let message_background_frame = Rect::from_parts(
            Point::new(start_of_message_content_x, cursor.1.y),
            message_line_size,
        );
        // Background of leading text
        scroll_view.get_slice().fill_rect(
            message_background_frame,
            attributes.main_text_background_color,
            StrokeThickness::Filled,
        );
        /*
        // Background border of leading text
        // TODO(PT): It looks like outline rects that are spread across multiple scroll view tiles render edges
        // at tile boundaries, which is incorrect
        scroll_view.get_slice().fill_rect(
            message_background_frame,
            attributes.main_text_background_border_color,
            StrokeThickness::Width(1),
        );
        */
        text_view.draw_string(attributes.main_text, Color::black());
        // Advance to the next line
        let mut updated_cursor = text_view.cursor_pos();
        updated_cursor.1 = Point::new(
            initial_cursor.1.x,
            initial_cursor.1.y + background_rect_height,
        );
        text_view.set_cursor_pos(updated_cursor);
    }

    fn render_structured_server_notice(
        &self,
        leading_text: &str,
        message_text: &str,
    ) {
        self.render_structured_message_with_attributes(
            RenderStructuredMessageAttributes::new(
                leading_text,
                Color::new(92, 92, 92),
                Color::new(71, 179, 255),
                Color::new(53, 133, 189),
                message_text,
                Color::black(),
                Color::new(181, 224, 255),
                Color::new(150, 186, 212),
            )
        )
    }

    fn render_structured_user_notice(
        &self,
        leading_text: &str,
        message_text: &str,
    ) {
        self.render_structured_message_with_attributes(
            RenderStructuredMessageAttributes::new(
                leading_text,
                Color::new(92, 92, 92),
                Color::new(255, 143, 38),
                Color::new(207, 116, 31),
                message_text,
                Color::black(),
                Color::new(252, 187, 126),
                Color::new(196, 145, 96),
            )
        )
    }

    fn render_structured_user_notice_level2(
        &self,
        leading_text: &str,
        message_text: &str,
    ) {
        self.render_structured_message_with_attributes(
            RenderStructuredMessageAttributes::new(
                leading_text,
                Color::new(92, 92, 92),
                Color::new(255, 143, 38),
                Color::new(207, 116, 31),
                message_text,
                Color::black(),
                Color::new(250, 198, 150),
                Color::new(199, 158, 119),
            )
        )
    }

    fn render_message(&self, msg: IrcMessage) {
        match msg.command {
            IrcCommand::Notice(p) => {
                self.render_structured_server_notice("Notice", &p.message);
            }
            IrcCommand::ReplyLocalUsers(p) => {
                self.render_structured_server_notice("User Info", &p.message);
            }
            IrcCommand::ReplyMessageOfTheDayStart(p) => {
                self.render_structured_user_notice("Welcome", &p.message);
            }
            IrcCommand::ReplyMessageOfTheDayLine(p) => {
                self.render_structured_user_notice_level2("Welcome", &p.message);
            }
            IrcCommand::ReplyMessageOfTheDayEnd(p) => {
                self.render_structured_user_notice("Welcome", &p.message);
            }
            IrcCommand::Mode(p) => {
                self.render_structured_server_notice("Mode", &p.mode);
            }
            IrcCommand::ReplyListClientUsers(p) => {
                self.render_structured_server_notice("User Info", &p.message);
            }
            IrcCommand::ReplyListOperatorUsers(p) => {
                self.render_structured_server_notice(
                    "User Info",
                    &format!("{} {}", p.operator_count, p.message),
                );
            }
            IrcCommand::ReplyListChannels(p) => {
                self.render_structured_server_notice(
                    "Channel Info",
                    &format!("{} {}", p.channel_count, p.message),
                );
            }
            IrcCommand::ReplyListUnknownUsers(p) => {
                self.render_structured_server_notice(
                    "Stats",
                    &format!("{} {}", p.unknown_user_count, p.message),
                );
            }
            IrcCommand::ReplyListUserMe(p) => {
                self.render_structured_server_notice(
                    "User Info",
                    &p.message,
                );
            }
            IrcCommand::ReplyGlobalUsers(p) => {
                self.render_structured_server_notice(
                    "Stats",
                    &p.message,
                );
            }
            IrcCommand::ReplyConnectionStats(p) => {
                self.render_structured_server_notice(
                    "Stats",
                    &p.message,
                );
            }
            IrcCommand::ReplyWelcome(p) => {
                self.render_structured_user_notice_level2("Welcome", &p.message);
            }
            IrcCommand::ReplyYourHost(p) => {
                self.render_structured_user_notice_level2("Host", &p.message);
            }
            IrcCommand::ReplyCreated(p) => {
                self.render_structured_server_notice("Created", &p.message);
            }
            IrcCommand::ReplyMyInfo(p) => {
                //self.render_structured_server_notice("Created", &p.message);
                //self.write_string(&format!("MyInfo {}: {} {} {} {} {:?}", p.nick, p.version, p.server_name, p.available_user_modes, p.available_channel_modes, p.channel_modes_with_params));
            }
            IrcCommand::ReplyISupport(p) => {
                //self.write_string(&format!("ISupport {}: {:?}", p.nick, p.entries));
            }
            unknown => {
                self.render_structured_server_notice("Unknown", &format!("{unknown:?}"));
                //self.write_string(&format!("Don't know how to format: {unknown:?}"));
            }
        }
    }

    fn handle_enter_key_pressed(&self) {
        self.send_input_and_clear_input_text_box();
    }

    fn send_input_and_clear_input_text_box(&self) {
        let mut irc_client = self.irc_client.borrow_mut();
        let input_view = &self.input_box_view;
        let input_str = {
            let input_drawn_characters = input_view.view.view.text.borrow();
            input_drawn_characters.iter().map(|c| c.value).collect::<String>()
        };
        irc_client.send_message_to_user("codyd51", &input_str);
        self.input_box_view.view.clear();
    }

    fn render_window_to_display(
        &self,
        graphics_protocol: &mut ScopedProtocol<GraphicsOutput>,
    ) {
        let layer = self.window.layer.borrow_mut();
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

        let resolution = self.window.frame().size;
        graphics_protocol.blt(
            BltOp::BufferToVideo {
                buffer: &buf_as_blt_pixel,
                src: BltRegion::Full,
                dest: (0, 0),
                dims: (resolution.width as _, resolution.height as _),
            }
        ).expect("Failed to blit screen");

        // Forget our re-interpreted vector of pixel data, as it's really owned by the window
        core::mem::forget(buf_as_blt_pixel);
    }

    fn handle_keyboard_updates(&self, system_table: &mut SystemTable<Boot>) {
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
        let currently_held_key = *self.currently_held_key.borrow();
        if key_held_on_this_iteration != currently_held_key {
            // Are we switching away from a held key?
            if currently_held_key.is_some() {
                self.window.handle_key_released(currently_held_key.unwrap());
            }
            if key_held_on_this_iteration.is_some() {
                // Inform the window that a new key is held
                self.window.handle_key_pressed(key_held_on_this_iteration.unwrap());

                // Hack to support scrolling the main content view up and down
                // UEFI key map for arrow keys:
                // Up: 1
                // Down: 2
                // Right: 3
                // Left: 4
                if key_held_on_this_iteration.unwrap() == KeyCode(1) {
                    self.scroll_up();
                }
                else if key_held_on_this_iteration.unwrap() == KeyCode(2) {
                    self.scroll_down();
                }
            }
            // And update our state to track that this key is currently held
            *self.currently_held_key.borrow_mut() = key_held_on_this_iteration;
        }
    }

    fn handle_mouse_updates(&self, pointer: &mut Pointer, pointer_resolution: Point) {
        let orig_mouse_position = *self.current_pointer_pos.borrow();
        let mut updated_current_pointer_pos = orig_mouse_position;
        // Process any updates from the pointer protocol
        let pointer_updates = pointer.read_state().expect("Failed to read pointer state");
        if let Some(pointer_updates) = pointer_updates {
            // Firstly, handle changes to the mouse position
            let rel_x = pointer_updates.relative_movement[0] as isize / pointer_resolution.x;
            let rel_y = pointer_updates.relative_movement[1] as isize /  pointer_resolution.y;
            // Ensure we're using non-zero values so log2 plays nice
            if rel_x != 0 || rel_y != 0 {
                // 'Scale' the movement so that larger motions from the user translate to faster motions across the screen
                let scale_factor = (rel_x.abs() + rel_y.abs()).ilog2() as isize;
                let scaled_rel_x = rel_x * scale_factor;
                let scaled_rel_y = rel_y * scale_factor;
                updated_current_pointer_pos.x += scaled_rel_x;
                updated_current_pointer_pos.y += scaled_rel_y;
            }

            // Bind the mouse to the screen resolution
            updated_current_pointer_pos.x = max(0, updated_current_pointer_pos.x);
            updated_current_pointer_pos.x = min(
                self.window.frame().size.width - self.cursor_size.width,
                updated_current_pointer_pos.x,
            );
            updated_current_pointer_pos.y = min(
                self.window.frame().size.height - self.cursor_size.height,
                updated_current_pointer_pos.y,
            );
            updated_current_pointer_pos.y = max(0, updated_current_pointer_pos.y);

            // Next, handle changes to the button state
            let orig_is_left_click_down = *self.is_left_click_down.borrow();
            let is_left_click_down_now = pointer_updates.button[0];
            if !orig_is_left_click_down && is_left_click_down_now {
                // We just entered a left click
                self.window.handle_mouse_left_click_down(updated_current_pointer_pos);
            }
            else {
                // We just exited a left click
                self.window.handle_mouse_left_click_up(updated_current_pointer_pos);
            }
            *self.is_left_click_down.borrow_mut() = is_left_click_down_now;
        }

        // And dispatch events to our view tree, if anything changed
        if updated_current_pointer_pos != orig_mouse_position {
            *self.current_pointer_pos.borrow_mut() = updated_current_pointer_pos;
            self.window.handle_mouse_moved(updated_current_pointer_pos);
        }
    }

    fn draw_cursor(&self) {
        let window_slice = self.window.get_slice();
        let cursor_frame = Rect::from_parts(
            *self.current_pointer_pos.borrow(),
            self.cursor_size,
        );
        // Inner cursor
        window_slice.fill_rect(
            cursor_frame,
            Color::new(66, 206, 245),
            StrokeThickness::Filled,
        );
        // Black outline
        window_slice.fill_rect(
            cursor_frame,
            Color::new(20, 20, 20),
            StrokeThickness::Width(3),
        );
    }

    fn draw_and_push_to_display(&self, graphics_protocol: &mut ScopedProtocol<GraphicsOutput>) {
        // Render the view tree
        self.window.draw();

        // Draw the cursor on top of everything else
        self.draw_cursor();

        // Push it all to the display
        self.render_window_to_display(graphics_protocol);
    }

    // TODO(PT): To make the UI more responsive, only draw 1 line of text per refresh in the startup message?

    fn step(&self) {
        let mut irc_client = self.irc_client.borrow_mut();
        irc_client.step();
        let mut active_connection = irc_client.active_connection.as_mut();
        let recv_buffer = &active_connection.expect("Expected an active connection").recv_buffer;
        let recv_data = recv_buffer.lock().borrow_mut().drain(..).collect::<Vec<u8>>();
        let mut response_parser = self.response_parser.borrow_mut();
        response_parser.ingest(&recv_data);

        while let Some(msg) = response_parser.parse_next_line() {
            self.render_message(msg);
        }

        // Ensure we always scroll to see the last line that was output
        // Unless the user is manually controlling our scroll position
        if !self.is_user_manually_scrolling() {
            self.scroll_to_last_visible_line();
        }
    }

    fn is_user_manually_scrolling(&self) -> bool {
        *self.is_user_manually_scrolling.borrow()
    }
}

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
    let font_regular = ttf_renderer::parse(&read_file(bs, "EFI\\Boot\\BigCaslon.ttf"));
    //let font_arial = ttf_renderer::parse(&read_file(bs, "EFI\\Boot\\Chancery.ttf"));
    //let font_italic = ttf_renderer::parse(&read_file(bs, "EFI\\Boot\\chancery.ttf"));
    let font_italic = ttf_renderer::parse(&read_file(bs, "EFI\\Boot\\new_york_italic.ttf"));
    info!("All done!");

    let resolution = Size::new(1360, 768);
    let mut graphics_protocol = set_resolution(
        bs,
        resolution,
    ).unwrap();

    let mut irc_client = IrcClient::new(bs);
    {
        irc_client.connect_to_server();
        let nickname = "phillip-testing2";
        let real_name = "phillip@axleos.com";
        irc_client.set_nickname(nickname);
        irc_client.set_user(nickname, real_name);
    }
    {
        let conn = irc_client.active_connection.as_mut();
        let conn = conn.unwrap();
        Rc::clone(&conn).set_up_receive_signal_handler();
    }
    // Theory: we need to do the same careful stuff for transmit as for receive
    // To test, going to try to only set up the RX handler after doing our initial transmits

    let pointer_handle = bs.get_handle_for_protocol::<Pointer>().expect("Failed to find handle for Pointer protocol");
    let mut pointer = bs.open_protocol_exclusive::<Pointer>(pointer_handle).expect("failed to open proto");
    pointer.reset(false).expect("Failed to reset cursor");

    let pointer_resolution = pointer.mode().resolution;
    let pointer_resolution = Point::new(
        pointer_resolution[0] as _,
        pointer_resolution[1] as _,
    );

    let mut app = App::new(
        resolution,
        font_regular,
        font_italic,
        pointer_resolution,
        irc_client,
    );

    //app.handle_recv_data(&("This is a test of a bnch of text that gets sent over the network pipe and sent to the client where it is then rendered and rendered again perhaps on a new line considering its length").as_bytes());

    loop {
        app.step();
        app.handle_keyboard_updates(&mut system_table);
        app.handle_mouse_updates(&mut pointer, pointer_resolution);
        app.draw_and_push_to_display(&mut graphics_protocol);
    }
}

