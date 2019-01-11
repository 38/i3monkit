use crate::widget::{Widget, WidgetUpdate};
use crate::protocol::Block;

/// The widget that shows local time
pub struct DateTimeWidget(bool);

impl DateTimeWidget {
    /// Create a new time widget
    pub fn new() -> Self {
        DateTimeWidget(true)
    }
}

impl Widget for DateTimeWidget {
    fn update(&mut self) -> Option<WidgetUpdate> {
       let time_string = if self.0 {
           format!("{}", chrono::Local::now().format("%H:%M"))
       } else {
           format!("{}", chrono::Local::now().format("%H %M"))
       };

       self.0 = !self.0;

       Some(WidgetUpdate {
           refresh_interval: std::time::Duration::new(1, 0),
           data: Some(Block::new().append_full_text(&time_string).clone())
       })
    }
}
