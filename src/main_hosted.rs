use agx_definitions::{
    Color, LikeLayerSlice, NestedLayerSlice, Point, PointF64, Polygon, Rect, Size,
};
use ttf_renderer::{parse, render_glyph_onto, Codepoint, Font};

use libgui::text_input_view::TextInputView;
use libgui::ui_elements::UIElement;
use libgui::AwmWindow;
use log::info;
use std::cell::RefCell;
use std::rc::Rc;
use std::{error, fs};

pub fn main() -> Result<(), Box<dyn error::Error>> {
    let window_size = Size::new(1360, 768);
    let window = Rc::new(AwmWindow::new("Hosted UEFIRC", window_size));

    let font_regular = ttf_renderer::parse(&std::fs::read("/Users/philliptennen/CLionProjects/uefirc/esp/EFI/Boot/BigCaslon.ttf").expect("Failed to read font file"));
    let font_arial = ttf_renderer::parse(&std::fs::read("/Users/philliptennen/CLionProjects/uefirc/esp/EFI/Boot/Arial.ttf").expect("Failed to read font file"));
    /*
    let main_view = MainView::new(
        font_regular,
        font_arial,
        move |_v, superview_size| {
            Rect::with_size(superview_size)
        }
    );
    Rc::clone(&window).add_component(Rc::clone(&main_view) as Rc<dyn UIElement>);

     */

    window.enter_event_loop();
    Ok(())
}
