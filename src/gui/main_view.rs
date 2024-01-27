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
use crate::gui::content_view::ContentView;
use crate::gui::title_view::TitleView;

#[derive(Drawable, NestedLayerSlice, Bordered, UIElement)]
pub struct MainView {
    pub view: Rc<View>,
    font_regular: Font,
}

impl MainView {
    pub fn new<F: 'static + Fn(&View, Size) -> Rect>(
        font_regular: Font,
        font_arial: Font,
        sizer: F,
    ) -> Rc<Self> {
        let view = View::new(Color::yellow(), sizer);

        let _self = Rc::new(
            Self {
                view: Rc::new(view),
                font_regular: font_regular.clone(),
            }
        );

        let content_sizer = |v: &View, superview_size: Size| {
            Rect::with_size(
                Size::new(
                    superview_size.width,
                    (superview_size.height as f64 * 0.92) as _,
                )
            )
        };

        let content_sizer_clone = content_sizer.clone();
        let title_sizer = move |v: &View, superview_size| {
            let content_frame = content_sizer_clone(v, superview_size);
            Rect::from_parts(
                Point::new(
                    0,
                    content_frame.max_y(),
                ),
                Size::new(
                    superview_size.width,
                    superview_size.height - content_frame.height(),
                )
            )
        };

        let title = TitleView::new(
            font_regular.clone(),
            Size::new(32, 32),
            move |v, s| title_sizer(v, s),
        );
        Rc::clone(&_self).add_component(Rc::clone(&title) as Rc<dyn UIElement>);

        let content = ContentView::new(
            font_arial.clone(),
            Size::new(18, 18),
            content_sizer,
        );
        Rc::clone(&_self).add_component(Rc::clone(&content) as Rc<dyn UIElement>);

        _self
    }

    pub fn add_component(self: Rc<Self>, elem: Rc<dyn UIElement>) {
        Rc::clone(&self.view).add_component(elem)
    }
}
