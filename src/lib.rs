// Copyright (C) 2019, Hao Hou

//! # i3monkit - The i3 Status Bar Monitor Toolkit
//!
//! This is a toolkit for building customized i3 status bar for the [i3 tiling window manager](https://i3wm.org).
//! i3 has its default status bar program called `i3status`, but it's somehow limited and hard to
//! extend and customize. This crate gives you the ability to reimplement an status bar program for
//! i3 quickly.
//!
//! It comes with a set of builtin widget as well, such as, CPU usage bar, Network speed meter,
//! etc.
//!
//! You can crate your own status bar with just a few lines of code in Rust.
//!
//! First, you need to crate a crate and import this crate
//!
//! ```toml
//! [dependencies]
//! i3mokit = "*"
//!
//! ```
//!
//! Then, config your customized i3 status bar
//!
//! ```rust
//! use i3monkit::*;
//! use i3monkit::widget::*;
//! fn main() {
//!     let bar = WidgetCollection::new();
//!
//!     //Add realtime stock prices, for example: Microsoft, AMD and Facebook
//!     let socket_client = StockClient::new("your-alphavantage-API-key");
//!     bar.push(StockClient::create_widget("MSFT"));
//!     bar.push(StockClient::create_widget("AMD"));
//!     bar.push(StockClient::create_widget("FB"));
//!
//!     //Realtime upload/download rate for a interface
//!     bar.push(NetworkSpeedWidget::new("wlp58s0"));
//!
//!     //Display all the cpu usage for each core
//!     for i in 0..4 {
//!         bar.push(CpuWidget::new(i));
//!     }
//!
//!     //Volume widget
//!     bar.push(VolumeWidget::new("default", "Master"));
//!
//!     //Battery status
//!     bar.push(BatteryWidget::new(0));
//!
//!     //Time
//!     bar.push(DateTimeWidget::new());
//!
//!     // Then start updating the satus bar
//!     bar.update_loop(I3Protocol::new(Header::new(1)));
//! }
//! ```
//!

mod protocol;
mod widget;
pub mod widgets;

pub use crate::protocol::{Header, I3Protocol, Block};
pub use crate::widget::{Widget, WidgetUpdate, WidgetCollection, Decoratable};
