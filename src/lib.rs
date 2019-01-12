// Copyright (C) 2019, Hao Hou

//! # i3monkit - The i3 Status Bar Monitor Toolkit
//!
//! This is a toolkit for building customized i3 status bar for the [i3 tiling window manager](https://i3wm.org).
//! i3 has its default status bar program called `i3status`, but it's somehow limited and hard to
//! extend and customize. This crate gives you the ability to reimplement an status bar program for
//! i3 quickly.
//!
//! ![Screenshort](https://raw.githubusercontent.com/38/i3monkit/master/screenshot.png)
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
//! use i3monkit::widgets::*;                                                     
//! 
//! fn main() {
//!     let mut bar = WidgetCollection::new();
//! 
//!     //Add realtime stock prices, for example: Microsoft, AMD and Facebook
//!     let stock_client = StockClient::new("your-alphavantage-API-key");         
//!     bar.push(StockClient::create_widget(&stock_client, "MSFT"));              
//!     bar.push(StockClient::create_widget(&stock_client, "AMD"));               
//!     bar.push(StockClient::create_widget(&stock_client, "FB"));
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
//!     bar.push(VolumeWidget::new("default", "Master", 0));
//!                                                                               
//!     //Battery status                                                          
//!     bar.push(BatteryWidget::new(0));
//!                                                                               
//!     //Time                                                                    
//!     bar.push(DateTimeWidget::new());
//!                                                                               
//!     // Then start updating the satus bar                                      
//!     bar.update_loop(I3Protocol::new(Header::new(1), std::io::stdout()));
//! }
//! ```
//!                                                                              
//! Finally, you can change `~/.config/i3/config` to make i3wm uses your status bar program
//!
//! ``` config
//!  # Start i3bar to display a workspace bar (plus the system information i3status
//!  # finds out, if available)
//!  bar {
//!  	status_command path/to/your/customized/status/program
//!  	tray_output primary
//!  	colors {
//!  	   background #222222
//!  	   statusline #00ee22
//!  	   separator #666666
//!  	   #                  border  backgr. text
//!  	   focused_workspace  #4c7899 #285577 #ffffff
//!  	   inactive_workspace #333333 #222222 #888888
//!  	   urgent_workspace   #2f343a #900000 #ffffff
//!  	}                                                           
//!   }
//! ```
//! ## Write your own widget
//!
//! You can also add your customized widget to the framework by implementing the `Widget` trait.
//!
//! ```
//! use i3monkit::{Block, Widget, WidgetUpdate};
//! struct Greeter(&'static str);
//! impl Widget for Greeter {
//!     fn update(&mut self) -> Option<WidgetUpdate> {
//!         Some(WidgetUpdate{
//!             refresh_interval: std::time::Duration::new(3600,0),
//!             data: Some(Block::new().append_full_text(self.0).clone())
//!         })
//!     }
//! }
//!
//! fn main() {
//!     let bar = WidgetCollection::new();
//!     bar.push(Greeter("hello world"));
//!     .....
//! }
//! ```
//!
mod protocol;
mod widget;
pub mod widgets;

pub use crate::protocol::{Header, I3Protocol, Block, ColorRGB};
pub use crate::widget::{Widget, WidgetUpdate, WidgetCollection, Decoratable};
