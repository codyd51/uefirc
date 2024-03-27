use alloc::boxed::Box;
use alloc::rc::Rc;
use agx_definitions::{Color, LikeLayerSlice, Rect, RectInsets, Size, Point, PixelByteLayout};
use libgui::bordered::Bordered;
use libgui::text_view::TextView;
use agx_definitions::{Drawable, NestedLayerSlice};
use libgui::KeyCode;
use libgui::ui_elements::UIElement;
use alloc::rc::Weak;
use libgui::view::View;
use libgui_derive::{Bordered, Drawable, NestedLayerSlice, UIElement};
use alloc::vec::Vec;
use libgui::label::Label;
use ttf_renderer::Font;

#[derive(Drawable, NestedLayerSlice, UIElement, Bordered)]
pub struct ContentView {
    pub view: Rc<TextView>,
}

impl ContentView {
    pub fn new<F: Fn(&View, Size) -> Rect + 'static>(
        font: Font,
        font_size: Size,
        sizer: F,
    ) -> Rc<Self> {
        let view = TextView::new_with_font(
            Color::white(),
            font.clone(),
            font_size,
            RectInsets::new(2, 2, 2, 2),
            sizer,
            // PT: My emulated UEFI environment uses BGRA
            PixelByteLayout::BGRA,
        );

        Rc::new(
            Self {
                view: Rc::clone(&view),
            }
        )
    }

    pub fn add_component(self: Rc<Self>, elem: Rc<dyn UIElement>) {
        Rc::clone(&self.view).add_component(elem)
    }
}
