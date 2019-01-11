mod datetime;
mod volume;
mod battery;
mod cpu;
mod network;
mod stock;

pub use self::datetime::DateTimeWidget;
pub use self::volume::VolumeWidget;
pub use self::battery::BatteryWidget;
pub use self::cpu::CpuWidget;
pub use self::network::NetworkSpeedWidget;
pub use self::stock::{StockClient, StockWidget};
