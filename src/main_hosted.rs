use agx_definitions::{
    Color, LikeLayerSlice, NestedLayerSlice, Point, PointF64, Polygon, Rect, Size,
};
use ttf_renderer::{parse, render_glyph_onto, Codepoint, Font};

use libgui::text_input_view::TextInputView;
use libgui::ui_elements::UIElement;
use libgui::AwmWindow;
use std::cell::RefCell;
use std::rc::Rc;
use std::{error, fs};
use crate::gui::MainView;

pub fn main() -> Result<(), Box<dyn error::Error>> {
    let window_size = Size::new(1240, 1000);
    let window = Rc::new(AwmWindow::new("Hosted UEFIRC", window_size));

    let main_view_sizer = |superview_size: Size| Rect::from_parts(Point::zero(), superview_size);
    /*
    let font_path = "/System/Library/Fonts/NewYorkItalic.ttf";
    let font_size = Size::new(32, 32);
    let main_view = TextInputView::new(
        Some(font_path),
        font_size,
        move |_v, superview_size| {
            main_view_sizer(superview_size)
        },
    );

    Rc::clone(&window).add_component(Rc::clone(&main_view) as Rc<dyn UIElement>);

    let mut main_view_slice = main_view.get_slice();
     */

    let main_view = MainView::new(
        move |_v, superview_size| {
            Rect::with_size(superview_size)
        }
    );
    Rc::clone(&window).add_component(Rc::clone(&main_view) as Rc<dyn UIElement>);

    window.enter_event_loop();
    Ok(())
}
