
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
use libgui::text_input_view::TextInputView;
use ttf_renderer::Font;

#[derive(Drawable, NestedLayerSlice, UIElement, Bordered)]
pub struct InputBoxView {
    pub view: Rc<TextInputView>,
}

impl InputBoxView {
    pub fn new<F: Fn(&View, Size) -> Rect + 'static>(
        font: Font,
        font_size: Size,
        sizer: F,
    ) -> Rc<Self> {
        let view = TextInputView::new_with_font(
            font.clone(),
            font_size,
            RectInsets::new(80, 2, 2, 2),
            sizer,
            // PT: My UEFI environment uses BGRA
            PixelByteLayout::BGRA,
        );

        let _self = Rc::new(
            Self {
                view: Rc::clone(&view),
            }
        );

        /*
        let prompt = Rc::new(
            Label::new_with_font(
                "Type: ",
                Color::new(50, 50, 50),
                font.clone(),
                Size::new(32, 32),
                move |_v, superview_size| {
                    Rect::from_parts(
                        Point::new(
                            (font_size.width as f64 * 1.0) as _,
                            ((superview_size.height as f64 / 2.0) - (font_size.height as f64 / 1.5)) as _,
                        ),
                        Size::new(superview_size.width / 2, superview_size.height),
                    )
                }
            )
        );
        Rc::clone(&_self).add_component(Rc::clone(&prompt) as Rc<dyn UIElement>);
        */

        _self
    }

    pub fn add_component(self: Rc<Self>, elem: Rc<dyn UIElement>) {
        Rc::clone(&self.view).add_component(elem)
    }
}
