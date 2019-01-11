mod protocol;
mod widget;
pub mod widgets;

pub use crate::protocol::{Header, I3Protocol, Block};
pub use crate::widget::{Widget, WidgetUpdate, WidgetCollection};
