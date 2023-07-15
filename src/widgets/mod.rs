mod battery;
mod cpu;
mod datetime;
mod network;
mod stock;
mod volume;

pub use self::battery::BatteryWidget;
pub use self::cpu::CpuWidget;
pub use self::datetime::DateTimeWidget;
pub use self::network::NetworkSpeedWidget;
pub use self::stock::{StockClient, StockWidget};
pub use self::volume::VolumeWidget;
