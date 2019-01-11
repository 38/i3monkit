use std::time::{Duration, SystemTime};
use std::thread::sleep;
use std::collections::BinaryHeap;
use std::io::Write;
use std::cmp::Ordering;
use crate::protocol::{Block, I3Protocol};

pub struct WidgetUpdate {
    pub refresh_interval : Duration,
    pub data             : Option<Block>,
}

#[allow(dead_code)]
pub struct WidgetDecorator<T:Widget, F: FnMut(&mut Block)> {
    inner : T,
    proc  : F
}

pub trait Widget {
    fn update(&mut self) -> Option<WidgetUpdate>;
}

pub trait Decoratable : Widget + Sized {
    fn decorate_with<F:FnMut(&mut Block)>(self, proc:F) -> WidgetDecorator<Self, F> {
        WidgetDecorator {
            inner:self ,
            proc
        }
    }
}

impl <T:Widget + Sized> Decoratable for T {}

impl <T:Widget, F: FnMut(&mut Block)> Widget for WidgetDecorator<T, F> {
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
    fn partial_cmp(&self, that:&Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&that.0, &self.0)
    }
}

impl Ord for RefreshEvent {
    fn cmp(&self, that:&Self) -> Ordering {
        Ord::cmp(&that.0, &self.0)
    }
}

pub struct WidgetCollection {
    widgets: Vec<Box<dyn Widget>>,
    idx_map: Vec<usize>,
    event_queue: BinaryHeap<RefreshEvent>,
    result_buffer: Vec<Block>
}

impl WidgetCollection {
    pub fn new() -> WidgetCollection {
        WidgetCollection {
            widgets: Vec::new(),
            event_queue: BinaryHeap::new(),
            result_buffer: Vec::new(),
            idx_map: Vec::new()
        }
    }

    pub fn push<W:Widget + 'static>(&mut self, widget:W) -> &mut Self {
        self.widgets.push(Box::new(widget));
        self
    }

    pub fn update_loop<T:Write>(&mut self, mut proto_inst : I3Protocol<T>) {
        self.event_queue.clear();

        let size = self.widgets.len() ;

        for (idx, widget) in self.widgets.iter_mut().enumerate() {
            if let Some(result) = widget.update() {
                self.event_queue.push(RefreshEvent(SystemTime::now() + result.refresh_interval, idx));
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

            let sleep_duration = next_event.0.duration_since(SystemTime::now()).unwrap_or_else(|_| Duration::new(0,0));

            sleep(sleep_duration);

            if let Some(mut update) = self.widgets[next_event.1].update() {

                if update.data.is_some() {
                    std::mem::swap(&mut self.result_buffer[self.idx_map[next_event.1]], update.data.as_mut().unwrap());
                }

                let new_event = RefreshEvent(SystemTime::now() + update.refresh_interval, next_event.1);

                self.event_queue.push(new_event);

                proto_inst.refresh(&self.result_buffer)
            }
        }
    }
}
