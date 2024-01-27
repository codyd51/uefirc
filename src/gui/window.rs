use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::cmp::max;
use core::fmt::{Display, Formatter};
use agx_definitions::{CHAR_HEIGHT, CHAR_WIDTH, Color, FONT8X8, LikeLayerSlice, Point, Rect, Size, StrokeThickness};
use log::info;
use ttf_renderer::{Font, render_glyph_onto, Codepoint};
use uefi::proto::console::gop::{BltOp, BltPixel, BltRegion, GraphicsOutput};
use uefi::table::boot::ScopedProtocol;
use uefi_services::println;
use crate::app::IrcClient;

struct PixelBuffer {
    size: Size,
    bpp: usize,
    pixels: Vec<u8>,
}

impl PixelBuffer {
    fn new(size: Size) -> Self {
        // PT: Assume 4 bytes per pixel everywhere...
        let bpp = 4;
        Self {
            size,
            bpp,
            pixels: vec![0; (size.width * size.height * (bpp as isize)) as usize],
        }
    }

    fn get_frame_mut(&mut self) -> &mut Vec<u8> {
        &mut self.pixels
    }
}

struct PixelLayerSlice {
    parent: Rc<RefCell<PixelBuffer>>,
    parent_size: Size,
    frame: Rect,
    global_origin: Point,
}

impl PixelLayerSlice {
    fn new(
        parent: &Rc<RefCell<PixelBuffer>>,
        parent_size: Size,
        frame: Rect,
        global_origin: Point,
    ) -> Self {
        Self {
            parent: Rc::clone(parent),
            parent_size,
            frame,
            global_origin,
        }
    }
}

impl Display for PixelLayerSlice {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "<PixelLayerSlice {}>", self.frame)
    }
}

impl LikeLayerSlice for PixelLayerSlice {
    fn frame(&self) -> Rect {
        self.frame
    }

    fn fill_rect(&self, raw_rect: Rect, color: Color, thickness: StrokeThickness) {
        let mut rect = self.frame.constrain(raw_rect);
        rect.size.width = max(rect.size.width, 0);
        rect.size.height = max(rect.size.height, 0);

        let bpp = 4;
        let parent_size = self.parent_size;
        let parent_bytes_per_row = parent_size.width * bpp;
        let bpp_multiple = Point::new(bpp, parent_bytes_per_row);
        let slice_origin_offset = self.global_origin * bpp_multiple;
        let rect_origin_offset = slice_origin_offset + (rect.origin * bpp_multiple);

        if let StrokeThickness::Width(px_count) = thickness {
            let top = Rect::from_parts(rect.origin, Size::new(rect.width(), px_count));
            self.fill_rect(top, color, StrokeThickness::Filled);

            let left = Rect::from_parts(rect.origin, Size::new(px_count, rect.height()));
            self.fill_rect(left, color, StrokeThickness::Filled);

            // The leftmost `px_count` pixels of the bottom rect are drawn by the left rect
            let bottom = Rect::from_parts(
                Point::new(rect.origin.x + px_count, rect.max_y() - px_count),
                Size::new(rect.width() - px_count, px_count),
            );
            self.fill_rect(bottom, color, StrokeThickness::Filled);

            // The topmost `px_count` pixels of the right rect are drawn by the top rect
            // The bottommost `px_count` pixels of the right rect are drawn by the bottom rect
            let right = Rect::from_parts(
                Point::new(rect.max_x() - px_count, rect.origin.y + px_count),
                Size::new(px_count, rect.height() - (px_count * 2)),
            );
            self.fill_rect(right, color, StrokeThickness::Filled);
        } else {
            let mut pixels = self.parent.borrow_mut();
            let fb = pixels.get_frame_mut();
            // Construct the filled row of pixels that we can copy row-by-row
            let bytes_in_row = (rect.width() * bpp) as usize;
            let mut src_row_slice = vec![0; bytes_in_row];
            for pixel_bytes_chunk in src_row_slice.chunks_exact_mut(bpp as _) {
                pixel_bytes_chunk[0] = color.b;
                pixel_bytes_chunk[1] = color.g;
                pixel_bytes_chunk[2] = color.r;
                pixel_bytes_chunk[3] = 0xff;
            }

            for y in 0..rect.height() {
                let row_start = (rect_origin_offset.y
                    + (y * parent_bytes_per_row)
                    + rect_origin_offset.x) as usize;
                let dst_row_slice = &mut fb[row_start..row_start + ((rect.width() * bpp) as usize)];
                dst_row_slice.copy_from_slice(&src_row_slice);
            }
        }
    }

    fn fill(&self, color: Color) {
        self.fill_rect(
            Rect::from_parts(Point::zero(), self.frame.size),
            color,
            StrokeThickness::Filled,
        )
    }

    fn putpixel(&self, loc: Point, color: Color) {
        /*
        if !self.frame.contains(loc + self.frame.origin) {
            return;
        }
        */

        let bpp = 4;
        let parent_size = self.parent_size;
        let parent_bytes_per_row = parent_size.width * bpp;
        let bpp_multiple = Point::new(bpp, parent_bytes_per_row);
        let mut pixels = self.parent.borrow_mut();
        let fb = pixels.get_frame_mut();
        let slice_origin_offset = self.global_origin * bpp_multiple;
        //let off = slice_origin_offset + (loc.y * parent_bytes_per_row) + (loc.x * bpp);
        let point_offset = slice_origin_offset + (loc * bpp_multiple);
        let off = (point_offset.y + point_offset.x) as usize;
        fb[off + 0] = color.b;
        fb[off + 1] = color.g;
        fb[off + 2] = color.r;
        fb[off + 3] = 0xff;
    }

    fn getpixel(&self, _loc: Point) -> Color {
        todo!()
    }

    fn get_slice(&self, rect: Rect) -> Box<dyn LikeLayerSlice> {
        //println!("LikeLayerSlice for PixelLayerSlice.get_slice({rect})");
        let constrained = Rect::from_parts(Point::zero(), self.frame.size).constrain(rect);
        let to_current_coordinate_system =
            Rect::from_parts(self.frame.origin + rect.origin, constrained.size);
        Box::new(Self::new(
            &self.parent,
            self.parent_size,
            to_current_coordinate_system,
            self.global_origin + rect.origin,
        ))
    }

    fn blit(
        &self,
        _source_layer: &Box<dyn LikeLayerSlice>,
        _source_frame: Rect,
        _dest_origin: Point,
    ) {
        todo!()
    }

    fn blit2(&self, source_layer: &Box<dyn LikeLayerSlice>) {
        // TODO(PT): Share this implementation with LayerSlice?
        assert_eq!(
            self.frame().size,
            source_layer.frame().size,
            "{} != {}",
            self.frame().size,
            source_layer.frame().size
        );

        let bpp = 4;
        let parent_size = self.parent_size;
        let parent_bytes_per_row = parent_size.width * bpp;
        let bpp_multiple = Point::new(bpp, parent_bytes_per_row);
        let mut pixels = self.parent.borrow_mut();
        let fb = pixels.get_frame_mut();
        let slice_origin_offset = self.frame.origin * bpp_multiple;

        let (src_base, src_slice_row_size, src_parent_framebuf_row_size) =
            source_layer.get_buf_ptr_and_row_size();

        for y in 0..self.frame().height() {
            // Blit an entire row at once
            let point_offset = slice_origin_offset + (Point::new(0, y) * bpp_multiple);
            let off = (point_offset.y + point_offset.x) as usize;
            let dst_row_slice = &mut fb[off..off + ((self.frame.width() * bpp) as usize)];
            let src_row_slice = unsafe {
                let src_row_start = src_base.offset(y * (src_parent_framebuf_row_size as isize));
                core::slice::from_raw_parts(src_row_start, src_slice_row_size)
            };
            dst_row_slice.copy_from_slice(src_row_slice);
        }
    }

    fn pixel_data(&self) -> Vec<u8> {
        todo!()
    }

    fn draw_char(&self, ch: char, draw_loc: Point, draw_color: Color, font_size: Size) {
        // Scale font to the requested size
        let scale_x: f64 = (font_size.width as f64) / (CHAR_WIDTH as f64);
        let scale_y: f64 = (font_size.height as f64) / (CHAR_HEIGHT as f64);

        let bitmap = FONT8X8[ch as usize];

        for draw_y in 0..font_size.height {
            // Go from scaled pixel back to 8x8 font
            let font_y = (draw_y as f64 / scale_y) as usize;
            let row = bitmap[font_y];
            for draw_x in 0..font_size.width {
                let font_x = (draw_x as f64 / scale_x) as usize;
                if row >> font_x & 0b1 != 0 {
                    self.putpixel(draw_loc + Point::new(draw_x, draw_y), draw_color);
                }
            }
        }
    }

    fn get_pixel_row(&self, _y: usize) -> Vec<u8> {
        todo!()
    }

    fn get_pixel_row_slice(&self, _y: usize) -> (*const u8, usize) {
        todo!()
    }

    fn get_buf_ptr_and_row_size(&self) -> (*const u8, usize, usize) {
        todo!()
    }

    fn track_damage(&self, _r: Rect) {
        todo!()
    }

    fn drain_damages(&self) -> Vec<Rect> {
        vec![]
        //todo!()
    }
}


pub struct PixelLayer {
    size: Size,
    pub pixel_buffer: Rc<RefCell<PixelBuffer>>,
}

impl PixelLayer {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            pixel_buffer: Rc::new(RefCell::new(PixelBuffer::new(size))),
        }
    }
}

impl Display for PixelLayer {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "<PixelLayer>")
    }
}

impl LikeLayerSlice for PixelLayer {
    fn frame(&self) -> Rect {
        Rect::with_size(self.size)
    }

    fn fill_rect(&self, raw_rect: Rect, color: Color, thickness: StrokeThickness) {
        self.get_slice(Rect::with_size(self.size))
            .fill_rect(raw_rect, color, thickness)
    }

    fn fill(&self, color: Color) {
        self.get_slice(Rect::with_size(self.size)).fill(color)
    }

    fn putpixel(&self, _loc: Point, _color: Color) {
        todo!()
    }

    fn getpixel(&self, _loc: Point) -> Color {
        todo!()
    }

    fn get_slice(&self, rect: Rect) -> Box<dyn LikeLayerSlice> {
        let constrained = Rect::from_parts(Point::zero(), self.size).constrain(rect);
        Box::new(PixelLayerSlice::new(
            &self.pixel_buffer,
            self.size,
            constrained,
            rect.origin,
        ))
    }

    fn blit(
        &self,
        _source_layer: &Box<dyn LikeLayerSlice>,
        _source_frame: Rect,
        _dest_origin: Point,
    ) {
        todo!()
    }

    fn blit2(&self, _source_layer: &Box<dyn LikeLayerSlice>) {
        todo!()
    }

    fn pixel_data(&self) -> Vec<u8> {
        todo!()
    }

    fn draw_char(&self, _ch: char, _draw_loc: Point, _draw_color: Color, _font_size: Size) {
        todo!()
    }

    fn get_pixel_row(&self, _y: usize) -> Vec<u8> {
        todo!()
    }

    fn get_pixel_row_slice(&self, _y: usize) -> (*const u8, usize) {
        todo!()
    }

    fn get_buf_ptr_and_row_size(&self) -> (*const u8, usize, usize) {
        todo!()
    }

    fn track_damage(&self, _r: Rect) {
        todo!()
    }

    fn drain_damages(&self) -> Vec<Rect> {
        todo!()
    }
}

pub struct Screen<'a> {
    layer: PixelLayer,
    size: Size,
    graphics_protocol: RefCell<ScopedProtocol<'a, GraphicsOutput>>,
    font_regular: Font,
    font_italic: Font,
    irc_client: IrcClient<'a>,
}

impl<'a> Screen<'a> {
    pub fn new(
        size: Size,
        graphics_protocol: ScopedProtocol<'a, GraphicsOutput>,
        font_regular: Font,
        font_italic: Font,
        irc_client: IrcClient<'a>,
    ) -> Rc<Self> {
        Rc::new(
            Self {
                layer: PixelLayer::new(size),
                size,
                graphics_protocol: RefCell::new(graphics_protocol),
                font_regular,
                font_italic,
                irc_client,
            }
        )
    }

    fn render_string(
        msg: &str,
        font: &Font,
        font_size: Size,
        font_color: Color,
        onto: &mut Box<dyn LikeLayerSlice>,
    ) {
        let cursor_origin = Point::new(2, 2);
        let mut cursor = cursor_origin;
        let scale_x = font_size.width as f64 / (font.units_per_em as f64);
        let scale_y = font_size.height as f64 / (font.units_per_em as f64);
        let scaled_em_size = Size::new(
            (font.bounding_box.size.width as f64 * scale_x) as isize,
            (font.bounding_box.size.height as f64 * scale_y) as isize,
        );
        for (_, ch) in msg.chars().enumerate() {
            let glyph = match font.glyph_for_codepoint(Codepoint::from(ch)) {
                None => continue,
                Some(glyph) => glyph,
            };
            let (_, metrics) = render_glyph_onto(
                glyph,
                font,
                onto,
                cursor,
                font_color,
                font_size,
            );
            cursor = Point::new(cursor.x + (metrics.advance_width as isize), cursor.y);
            // Spill onto the next line
            if cursor.x >= onto.frame().size.width - font_size.width {
                cursor.y += scaled_em_size.height;
                cursor.x = cursor_origin.x;
            }
        }
    }

    fn compute_string_layout_size(
        msg: &str,
        font: &Font,
        font_size: Size,
    ) -> Size {
        let cursor_origin = Point::new(2, 2);
        let mut cursor = cursor_origin;
        let scale_x = font_size.width as f64 / (font.units_per_em as f64);
        let scale_y = font_size.height as f64 / (font.units_per_em as f64);
        let scaled_em_size = Size::new(
            (font.bounding_box.size.width as f64 * scale_x) as isize,
            (font.bounding_box.size.height as f64 * scale_y) as isize,
        );
        for (_, ch) in msg.chars().enumerate() {
            let glyph = match font.glyph_for_codepoint(Codepoint::from(ch)) {
                None => continue,
                Some(glyph) => glyph,
            };
            let scaled_glyph_metrics = glyph.metrics().scale(scale_x, scale_y);
            cursor = Point::new(cursor.x + (scaled_glyph_metrics.advance_width as isize), cursor.y);
            // PT: NOTE: This functionality gives potentially inaccurate results depending on where the
            // string is rendered to, because we may be forced to spill onto new lines.
            /*
            // Spill onto the next line
            if cursor.x >= onto_frame_size.width - font_size.width {
                cursor.y += scaled_em_size.height;
                cursor.x = cursor_origin.x;
            }
             */
        }
        return Size::new(cursor.x + font_size.width, cursor.y + font_size.height)
    }

    pub fn step(self: &Rc<Self>) {
        // Draw the background
        self.layer.get_slice(self.layer.frame()).fill(Color::white());

        // Draw the 'menu bar' at the bottom
        // Project name on left
        let project_name_slice_size = Size::new(self.size.width, self.size.height / 20);
        let project_name_font_size = Size::new(24, 24);
        let mut project_name_slice = self.layer.get_slice(
            Rect::from_parts(
                Point::new(
                    0,
                    (self.size.height - project_name_slice_size.height) as _,
                ),
                project_name_slice_size,
            )
        );
        Self::render_string(
            "No operating system... no limits...",
            &self.font_regular,
            project_name_font_size,
            Color::new(160, 40, 90),
            &mut project_name_slice,
        );
        //self.layer.get_slice(self.layer.frame()).fill(Color::new(r as u8, g as u8, b as u8));
        // Slogan on right
        let slogan_font_size = Size::new(32, 32);
        let slogan = "No operating system... No limits...";
        //let slogan_len_x = slogan_font_size.width * (14 as isize);
        let slogan_layout_size = Self::compute_string_layout_size(
            slogan,
            &self.font_italic,
            slogan_font_size,
        );
        println!("Slogan layout size {slogan_layout_size:?}");
        let mut slogan_slice = self.layer.get_slice(
            Rect::from_parts(
                Point::new(
                    self.size.width - slogan_layout_size.width - (slogan_font_size.width * 4),
                    project_name_slice.frame().min_y(),
                ),
                slogan_layout_size,
            )
        );
        println!("Slogan slice frame {:?}", slogan_slice.frame());
        /*
        Self::render_string(
            slogan,
            &self.font_italic,
            slogan_font_size,
            Color::black(),
            &mut slogan_slice,
        );

         */

        self.render_to_display();
    }

    pub fn render_to_display(&self) {
        let pixel_buffer = self.layer.pixel_buffer.borrow_mut();

        let buf_as_u32 = {
            let buf_as_u8 = &pixel_buffer.pixels;
            let len = buf_as_u8.len() / 4;
            let capacity = buf_as_u8.capacity() / 4;

            let raw_parts = buf_as_u8.as_ptr() as *mut u32;
            let buf_as_u32 = unsafe { Vec::from_raw_parts(raw_parts, len, capacity) };
            buf_as_u32
        };

        let mut pixels: Vec<BltPixel> = vec![];
        for px in buf_as_u32.iter() {
            let bytes = px.to_le_bytes();
            pixels.push(BltPixel::new(
                bytes[2],
                bytes[1],
                bytes[0],
            ));
        }

        let mut graphics_protocol = self.graphics_protocol.borrow_mut();
        graphics_protocol.blt(
            BltOp::BufferToVideo {
                buffer: &pixels,
                src: BltRegion::Full,
                dest: (0, 0),
                dims: (self.size.width as _, self.size.height as _),
            }
        ).expect("Failed to blit screen");

        // Don't free the memory once done as it's owned by the pixel buffer
        core::mem::forget(buf_as_u32);
    }
}
