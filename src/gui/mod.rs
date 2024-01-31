//mod window;
mod main_view;
mod title_view;
mod content_view;
mod input_box_view;

use agx_definitions::{Rect, Size};
use libgui::view::View;
//pub use window::{Screen};
pub use main_view::{MainView};
pub use title_view::TitleView;
pub use content_view::ContentView;
pub use input_box_view::InputBoxView;
