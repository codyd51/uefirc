
use alloc::boxed::Box;
use alloc::rc::Rc;
use agx_definitions::{Color, LikeLayerSlice, Rect, RectInsets, Size, Point};
use libgui::bordered::Bordered;
use libgui::text_view::TextView;
use agx_definitions::{Drawable, NestedLayerSlice};
use libgui::KeyCode;
use libgui::ui_elements::UIElement;
use alloc::rc::Weak;
use libgui::view::View;
use libgui_derive::{Bordered, Drawable, NestedLayerSlice, UIElement};
use crate::gui::Sizer;
use alloc::vec::Vec;
use ttf_renderer::Font;

#[derive(Drawable, NestedLayerSlice, UIElement, Bordered)]
pub struct TitleView {
    pub view: Rc<TextView>,
}

impl TitleView {
    pub fn new<F: Fn(&View, Size) -> Rect + 'static>(
        font: Font,
        font_size: Size,
        sizer: F,
    ) -> Rc<Self> {
        let view = TextView::new_with_font(
            Color::white(),
            font,
            font_size,
            RectInsets::new(2, 2, 2, 2),
            sizer,
        );

        let _self = Rc::new(
            Self {
                view,
            }
        );

        _self
    }
}
