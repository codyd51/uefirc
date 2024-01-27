use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::fmt::{Display};
use agx_definitions::{Color, LikeLayerSlice, Point, Rect, RectInsets, Size};
use libgui::bordered::Bordered;
use libgui::font::load_font;
use libgui::scroll_view::ScrollView;
use libgui::text_view::{CursorPos, DrawnCharacter, TextView};
use libgui::view::View;
use ttf_renderer::{Font, render_glyph_onto, Codepoint};
use agx_definitions::{Drawable, NestedLayerSlice};
use libgui::ui_elements::UIElement;
use libgui::KeyCode;
use alloc::rc::Weak;
use libgui_derive::{Drawable, NestedLayerSlice, UIElement};

#[derive(Drawable, NestedLayerSlice, UIElement)]
pub struct MainView {
    pub view: Rc<View>,
}

impl MainView {
    pub fn new<F: 'static + Fn(&View, Size) -> Rect>(
        sizer: F,
    ) -> Rc<Self> {
        let view = View::new(Color::yellow(), sizer);

        Rc::new(
            Self {
                view: Rc::new(view),
            }
        )
    }
}

impl Bordered for MainView {
    fn border_insets(&self) -> RectInsets {
        self.view.border_insets()
    }

    fn draw_inner_content(&self, outer_frame: Rect, onto: &mut Box<dyn LikeLayerSlice>) {
        self.view.draw_inner_content(outer_frame, onto);
    }
}
