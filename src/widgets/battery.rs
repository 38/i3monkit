use crate::widget::{Widget, WidgetUpdate};
use crate::protocol::{Block, ColorRGB};

use chrono::Duration;

use std::path::PathBuf;

use std::fs::File;
use std::io::{BufRead, BufReader};

const BATTERY_STATUS_PREFIX:&'static str = "/sys/class/power_supply/BAT";

#[derive(Debug)]
enum BatteryStatus {
    Unknown,
    Charging,
    Discharging,
    Full,
}

impl BatteryStatus {
    fn from_str(status:&str) -> BatteryStatus {
        match status {
            "Charging" => BatteryStatus::Charging,
            "Discharging" => BatteryStatus::Discharging,
            "Full" => BatteryStatus::Full,
            _ => BatteryStatus::Unknown
        }
    }

    fn get_status_text(&self) -> &'static str {
        match self {
            BatteryStatus::Unknown => "<span foreground=\"grey\">U</span> ",
            BatteryStatus::Charging => "<span foreground=\"green\">C</span> ",
            BatteryStatus::Discharging => "<span foreground=\"red\">D</span> ",
            BatteryStatus::Full => "<span foreground=\"green\">F</span> "
        }
    }
}

#[derive(Debug)]
struct BatteryState {
    full: u32,
    now : u32,
    design: u32,
    voltage: u32,
    rate: u32,
    stat: BatteryStatus,
}

impl BatteryState {
    fn get(idx:u32) -> Option<Self> {
        let mut root_path = PathBuf::new();
        root_path.push(format!("{}{}", BATTERY_STATUS_PREFIX, idx));
        root_path.push("dummy");
        
        let mut read_battery_status = move |status:&str| {
            root_path.set_file_name(status);
            if let Ok(file) = File::open(root_path.as_path()) {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        return Some(line.trim().to_string());
                    }
                }
            } 
            None
        };

        let stat = BatteryStatus::from_str(&read_battery_status("status")?);
        let full = u32::from_str_radix(&read_battery_status("charge_full")?, 10).ok()?;
        let now  = u32::from_str_radix(&read_battery_status("charge_now")?, 10).ok()?;
        let rate = u32::from_str_radix(&read_battery_status("current_now")?, 10).ok()?;
        let design = u32::from_str_radix(&read_battery_status("charge_full_design")?, 10).ok()?;
        let voltage = u32::from_str_radix(&read_battery_status("voltage_now")?, 10).ok()?;

        Some(Self {full, now, rate, stat, design, voltage})
    }

    fn percentage(&self) -> u8 {
        ((self.now * 100) as f32 / self.design as f32).round() as u8
    }

    fn time_remaining(&self) -> Option<(Duration, f32)> {
        let target = match self.stat {
            BatteryStatus::Charging => self.full,
            BatteryStatus::Discharging => 0,
            _ => {return None; }
        };

        let remaning = if target < self.now { 
            self.now - target 
        } else {
            target - self.now 
        };

        let time = Duration::seconds((3600.0 * (remaning as f32) / (self.rate as f32)).round() as i64);
        let power = (self.voltage as f32/ 1_000_000.0) * (self.rate as f32 / 1_000_000.0);

        return Some((time,power));
    }
}

/// The battery status widget.
///
/// This widget shows the battery status of a laptop, such as, the percentage battery level,
/// current status (charing, discharing, full, etc), current discharging/charing rate, estimated
/// reminaing time, etc...
pub struct BatteryWidget(u32);

impl BatteryWidget {
    /// Create a new widget for specified battery
    ///
    /// **idx** The index for the battery, for most of the system with only 1 battery, it should be
    /// 0
    pub fn new(idx:u32) -> Self { 
        Self(idx)
    }

    fn render_batter_status(&self) -> (String, i32) {
        if let Some(info) = BatteryState::get(self.0) {
            let mut ret = format!("{} {}%", info.stat.get_status_text(), info.percentage());
            if let Some((time,power)) = info.time_remaining() {
                ret.push_str(&format!(" [{:02}:{:02}|{:3.1}W]", time.num_hours(), time.num_minutes() % 60, power)); 
            } 

            let sev = match info.percentage() {
                x if x > 50 => 3,
                x if x > 30 => 2,
                x if x > 10 => 1,
                _           => 0
            };

            return (ret, sev);
        }

        return ("Unknown".to_string(), -1);
    }
}

impl Widget for BatteryWidget {
    fn update(&mut self) -> Option<WidgetUpdate> {
        let (msg, sev) = self.render_batter_status();

        let mut data = Block::new();

        data.use_pango();
        data.append_full_text(&msg);

        match sev {
            0 => {data.color(ColorRGB::red());} ,
            1 => {data.color(ColorRGB::yellow());},
            _ => {}
        }

        return Some(WidgetUpdate {
            refresh_interval: std::time::Duration::new(5,0),
            data: Some(data)
        });
    }
}
