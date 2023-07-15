//! The widget infrastructure

use crate::protocol::{Block, I3Protocol};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::io::Write;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

/// An update of a widget.
///
/// This is used to return an widget update from the widget implementation
/// to the widget framework
///
/// For some use cases, it's possible that we do not deliver any update, but
/// requires the widget framework to call the widget again after some time.
/// This can be done by passing the widget update with an empty data payload.
pub struct WidgetUpdate {
    /// Amount of time until the widget gets refresh
    pub refresh_interval: Duration,
    /// Data payload to update, None indicates
    pub data: Option<Block>,
}

/// Widget decorator, which modifies the block returned by the widget
///
/// *i3monkit* allows uses to override any widget output by their own function with the dectorator.
/// For example, changing the color of time display
///
/// ```rust
///     use i3monkit::Decoratable;
///     let widget = i3monkit::widget::DateTimeWidget::new().decorate_with(|b| {
///         b.color(i3monkit::protcol::ColorRGB::red());
///     })
/// ```
///
pub struct WidgetDecorator<T: Widget, F: FnMut(&mut Block)> {
    inner: T,
    proc: F,
}

/// The trait for an widget.
///
/// A widget maintains a dynamic block on the i3bar
pub trait Widget {
    /// The function used to update the widget.
    ///
    /// Note: even with no update, the widget should return an non-empty update with empty data
    /// payload.
    /// If None is returned, the framework will disable this widget and do not call the update
    /// function anymore.
    fn update(&mut self) -> Option<WidgetUpdate>;
}

/// The trait for a decoratable object
pub trait Decoratable: Widget + Sized {
    /// Decorate the widget's block with a user-specified function
    ///
    /// **proc** The function we are going to use for block decoration
    fn decorate_with<F: FnMut(&mut Block)>(self, proc: F) -> WidgetDecorator<Self, F> {
        WidgetDecorator { inner: self, proc }
    }
}

impl<T: Widget + Sized> Decoratable for T {}

impl<T: Widget, F: FnMut(&mut Block)> Widget for WidgetDecorator<T, F> {
    fn update(&mut self) -> Option<WidgetUpdate> {
        if let Some(mut inner_result) = self.inner.update() {
            if let Some(ref mut data) = inner_result.data {
                (self.proc)(data);
            }

            return Some(inner_result);
        }
        None
    }
}

#[derive(PartialEq, Eq)]
struct RefreshEvent(SystemTime, usize);

impl PartialOrd for RefreshEvent {
    fn partial_cmp(&self, that: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&that.0, &self.0)
    }
}

impl Ord for RefreshEvent {
    fn cmp(&self, that: &Self) -> Ordering {
        Ord::cmp(&that.0, &self.0)
    }
}

/// The collection of widgets
///
/// in **i3monkit** a status bar is abstracted as an widget collection.
///
/// To create a i3 bar application with i3monkit, what needs to be done is:
///
/// ```rust
///     let bar = WidgetUpdate::new();
///
///     // Add whatever widget to the bar
///     bar.push(...);
///     ....
///
///     bar.update_loop();
/// ```
pub struct WidgetCollection {
    widgets: Vec<Box<dyn Widget>>,
    idx_map: Vec<usize>,
    event_queue: BinaryHeap<RefreshEvent>,
    result_buffer: Vec<Block>,
}

impl WidgetCollection {
    /// Creates a new widget collection
    pub fn new() -> WidgetCollection {
        WidgetCollection {
            widgets: Vec::new(),
            event_queue: BinaryHeap::new(),
            result_buffer: Vec::new(),
            idx_map: Vec::new(),
        }
    }

    /// Push a new widget to the collection
    pub fn push<W: Widget + 'static>(&mut self, widget: W) -> &mut Self {
        self.widgets.push(Box::new(widget));
        self
    }

    /// Start the main update loop and drawing the wigets on the i3bar
    pub fn update_loop<T: Write>(&mut self, mut proto_inst: I3Protocol<T>) {
        self.event_queue.clear();

        let size = self.widgets.len();

        for (idx, widget) in self.widgets.iter_mut().enumerate() {
            if let Some(result) = widget.update() {
                self.event_queue.push(RefreshEvent(
                    SystemTime::now() + result.refresh_interval,
                    idx,
                ));
                if let Some(data) = result.data {
                    self.result_buffer.push(data);
                }
                self.idx_map.push(self.result_buffer.len() - 1);
            } else {
                self.idx_map.push(size);
            }
        }

        while !self.event_queue.is_empty() {
            let next_event = self.event_queue.pop().unwrap();

            let sleep_duration = next_event
                .0
                .duration_since(SystemTime::now())
                .unwrap_or_else(|_| Duration::new(0, 0));

            sleep(sleep_duration);

            if let Some(mut update) = self.widgets[next_event.1].update() {
                if update.data.is_some() {
                    std::mem::swap(
                        &mut self.result_buffer[self.idx_map[next_event.1]],
                        update.data.as_mut().unwrap(),
                    );
                }

                let new_event =
                    RefreshEvent(SystemTime::now() + update.refresh_interval, next_event.1);

                self.event_queue.push(new_event);

                proto_inst.refresh(&self.result_buffer)
            }
        }
    }
}
