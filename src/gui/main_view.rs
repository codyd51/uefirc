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
use libgui_derive::{Drawable, NestedLayerSlice, UIElement, Bordered};
use crate::gui::title_view::TitleView;

#[derive(Drawable, NestedLayerSlice, Bordered, UIElement)]
pub struct MainView {
    pub view: Rc<View>,
    font_regular: Font,
}

impl MainView {
    pub fn new<F: 'static + Fn(&View, Size) -> Rect>(
        font_regular: Font,
        sizer: F,
    ) -> Rc<Self> {
        let view = View::new(Color::yellow(), sizer);

        let _self = Rc::new(
            Self {
                view: Rc::new(view),
                font_regular: font_regular.clone(),
            }
        );

        let title = TitleView::new(
            font_regular.clone(),
            Size::new(32, 32),
            move |_, superview_size| {
                let size = Size::new(
                    superview_size.width,
                    superview_size.height / 12,
                );
                Rect::from_parts(
                    Point::new(
                        0,
                        superview_size.height - size.height
                    ),
                    size
                )
                /*
                let size = Size::new(
                    superview_size.width,
                    superview_size.height / 12,
                );
                Rect::from_parts(
                    Point::new(
                        0,
                        superview_size.height - size.height
                    ),
                    size
                )
                */
            }
        );
        Rc::clone(&_self).add_component(Rc::clone(&title) as Rc<dyn UIElement>);

        /*
        let v = Rc::new(View::new(
            Color::white(),
            move |_, superview_size| {
                let size = Size::new(
                    superview_size.width,
                    superview_size.height / 12,
                );
                Rect::from_parts(
                    Point::new(
                        0,
                        superview_size.height - size.height
                    ),
                    size
                )
            }
        ));
        Rc::clone(&_self).add_component(Rc::clone(&v) as Rc<dyn UIElement>);
        */

        _self
    }

    pub fn add_component(self: Rc<Self>, elem: Rc<dyn UIElement>) {
        Rc::clone(&self.view).add_component(elem)
    }
}
