use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::fmt::{Display};
use agx_definitions::{Color, LikeLayerSlice, Point, Rect, RectInsets, Size};
use libgui::bordered::Bordered;
use libgui::view::View;
use ttf_renderer::{Font};
use agx_definitions::{Drawable, NestedLayerSlice};
use libgui::ui_elements::UIElement;
use libgui::KeyCode;
use alloc::rc::Weak;
use libgui_derive::{Drawable, NestedLayerSlice, UIElement, Bordered};
use crate::gui::content_view::ContentView;
use crate::gui::input_box_view::InputBoxView;
use crate::gui::title_view::TitleView;

#[derive(Drawable, NestedLayerSlice, Bordered, UIElement)]
pub struct MainView {
    pub view: Rc<View>,
    font_regular: Font,
    content_view: Rc<ContentView>,
    input_box_view: Rc<InputBoxView>,
}

impl MainView {
    pub fn new<F: 'static + Fn(&View, Size) -> Rect>(
        font_regular: Font,
        font_arial: Font,
        sizer: F,
    ) -> Rc<Self> {
        let view = View::new(Color::yellow(), sizer);

        let title_sizer = |v: &View, superview_size: Size| {
            Rect::with_size(
                Size::new(
                    superview_size.width,
                    (superview_size.height as f64 * 0.084) as _,
                )
            )
        };

        let title_sizer_clone = title_sizer.clone();
        let content_sizer = move |v: &View, superview_size: Size| {
            let title_frame = title_sizer_clone(v, superview_size);
            Rect::from_parts(
                Point::new(0, title_frame.max_y()),
                Size::new(
                    superview_size.width,
                    (superview_size.height as f64 * 0.82) as _,
                )
            )
        };

        let content_sizer_clone = content_sizer.clone();
        let input_box_sizer = move |v: &View, superview_size: Size| {
            let content_frame = content_sizer_clone(v, superview_size);
            Rect::from_parts(
                Point::new(
                    0,
                    content_frame.max_y(),
                ),
                Size::new(
                    superview_size.width,
                    (superview_size.height as f64 * 0.1) as _,
                )
            )
        };

        let content = ContentView::new(
            font_regular.clone(),
            //Size::new(16, 16),
            Size::new(20, 20),
            content_sizer,
        );

        let input_box = InputBoxView::new(
            font_regular.clone(),
            Size::new(24, 24),
            move |v, s| input_box_sizer(v, s),
        );

        let title = TitleView::new(
            font_regular.clone(),
            Size::new(32, 32),
            move |v, s| title_sizer(v, s),
        );

        let _self = Rc::new(
            Self {
                view: Rc::new(view),
                font_regular: font_regular.clone(),
                content_view: content.clone(),
                input_box_view: input_box.clone(),
            }
        );

        Rc::clone(&_self).add_component(Rc::clone(&title) as Rc<dyn UIElement>);
        Rc::clone(&_self).add_component(Rc::clone(&content) as Rc<dyn UIElement>);
        Rc::clone(&_self).add_component(Rc::clone(&input_box) as Rc<dyn UIElement>);

        _self
    }

    pub fn add_component(self: Rc<Self>, elem: Rc<dyn UIElement>) {
        Rc::clone(&self.view).add_component(elem)
    }

    pub fn handle_recv_data(&self, recv_data: &[u8]) {
        let recv_as_str = core::str::from_utf8(recv_data).unwrap();
        for ch in recv_as_str.chars() {
            self.content_view.view.draw_char_and_update_cursor(ch, Color::black());
        }
        let cursor_pos = self.content_view.view.cursor_pos.borrow().1;
        let viewport_height = self.content_view.frame().height();
        *self.content_view.view.view.layer.scroll_offset.borrow_mut() = Point::new(
            cursor_pos.x,
            cursor_pos.y - viewport_height + 32,
        );
    }
}
