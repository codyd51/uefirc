
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
use alloc::vec::Vec;
use libgui::label::Label;
use ttf_renderer::Font;

#[derive(Drawable, NestedLayerSlice, UIElement, Bordered)]
pub struct TitleView {
    pub view: Rc<View>,
}

impl TitleView {
    pub fn new<F: Fn(&View, Size) -> Rect + 'static>(
        font: Font,
        font_size: Size,
        sizer: F,
    ) -> Rc<Self> {
        let view = Rc::new(
            View::new(
                Color::white(),
                sizer,
            )
        );

        let _self = Rc::new(
            Self {
                view: Rc::clone(&view),
            }
        );

        let title = Label::new_with_font(
            "UEFIRC",
            Color::black(),
            font.clone(),
            Size::new(32, 32),
            move |_v, superview_size| {
                Rect::from_parts(
                    Point::new(
                        (font_size.width as f64 * 0.5) as _,
                        ((superview_size.height as f64 / 2.0) - (font_size.height as f64 / 1.5)) as _,
                    ),
                    Size::new(superview_size.width / 2, superview_size.height),
                )
            }
        );
        Rc::clone(&_self).add_component(Rc::clone(&title) as Rc<dyn UIElement>);

        let slogan = Label::new_with_font(
            "No operating system... No limits...",
            Color::black(),
            font.clone(),
            Size::new(24, 24),
            move |_v, superview_size| {
                Rect::from_parts(
                    Point::new(
                        (superview_size.width as f64 * 0.74) as _,
                        ((superview_size.height as f64 / 2.0) - (font_size.height as f64 / 1.8)) as _,
                    ),
                    Size::new(superview_size.width / 2, superview_size.height),
                )
            }
        );
        Rc::clone(&_self).add_component(Rc::clone(&slogan) as Rc<dyn UIElement>);

        _self
    }

    pub fn add_component(self: Rc<Self>, elem: Rc<dyn UIElement>) {
        Rc::clone(&self.view).add_component(elem)
    }
}
