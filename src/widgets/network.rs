use crate::protocol::Block;
use crate::widget::{Widget, WidgetUpdate};

use std::path::PathBuf;

use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind, Result};
use std::time::SystemTime;

const NETWORK_PATH_PREFIX: &'static str = "/sys/class/net";
const NETWORK_STAT_SUFFIX: &'static str = "statistics/dummy";

struct TransferStat {
    rx: u64,
    tx: u64,
    ts: SystemTime,
}

impl TransferStat {
    fn read_stat(interface: &str) -> Result<Self> {
        let mut path = PathBuf::new();
        path.push(format!(
            "{}/{}/{}",
            NETWORK_PATH_PREFIX, interface, NETWORK_STAT_SUFFIX
        ));

        let mut read_stat_file = move |what: &str| {
            path.set_file_name(what);

            let reader = BufReader::new(File::open(path.as_path())?);

            if let Some(line) = reader.lines().next() {
                return line;
            }

            return Err(Error::new(ErrorKind::Other, "Empty file"));
        };

        let rx = u64::from_str_radix(&(read_stat_file("rx_bytes")?), 10).unwrap();
        let tx = u64::from_str_radix(&(read_stat_file("tx_bytes")?), 10).unwrap();
        let ts = SystemTime::now();

        return Ok(Self { rx, tx, ts });
    }

    fn duration(&self, earlier: &Self) -> f64 {
        let duration = self.ts.duration_since(earlier.ts).unwrap();
        let secs = duration.as_secs() as f64 + duration.subsec_nanos() as f64 / 1_000_000_000.0;
        return secs;
    }

    fn rx_rate(&self, earlier: &Self) -> f64 {
        let duration = self.duration(earlier);
        if duration < 1e-5 {
            return std::f64::NAN;
        }

        return (self.rx - earlier.rx) as f64 / duration;
    }

    fn tx_rate(&self, earlier: &Self) -> f64 {
        let duration = self.duration(earlier);
        if duration < 1e-5 {
            return std::f64::NAN;
        }

        return (self.tx - earlier.tx) as f64 / duration;
    }
}

/// A widget that shows the network speed realtimely
pub struct NetworkSpeedWidget {
    interface: String,
    last_stat: TransferStat,
}

impl NetworkSpeedWidget {
    /// Create the widget, for given interface.
    ///
    /// **interface** The interface to monitor
    pub fn new(interface: &str) -> Self {
        let last_stat = TransferStat::read_stat(interface).unwrap();
        let interface = interface.to_string();
        Self {
            last_stat,
            interface,
        }
    }

    fn format_rate(rate: f64) -> String {
        if rate.is_nan() {
            return "N/A".to_string();
        }

        const UNIT_NAME: [&'static str; 6] = [" B/s", "KB/s", "MB/s", "GB/s", "TB/s", "PB/s"];

        let mut best_unit = UNIT_NAME[0];
        let mut best_multiplier = 1.0;

        for unit in UNIT_NAME[1..].iter() {
            if best_multiplier > rate / 1024.0 {
                break;
            }
            best_unit = unit;
            best_multiplier *= 1024.0
        }

        let ret = format!("{:6.1}{}", rate / best_multiplier, best_unit);

        return ret;
    }

    fn get_human_readable_stat(&mut self) -> Result<(String, String)> {
        let cur_stat = TransferStat::read_stat(&self.interface)?;

        let rx_rate = cur_stat.rx_rate(&self.last_stat);
        let tx_rate = cur_stat.tx_rate(&self.last_stat);

        self.last_stat = cur_stat;

        return Ok((Self::format_rate(rx_rate), Self::format_rate(tx_rate)));
    }
}

impl Widget for NetworkSpeedWidget {
    fn update(&mut self) -> Option<WidgetUpdate> {
        if let Ok((rx, tx)) = self.get_human_readable_stat() {
            let mut data = Block::new();
            data.use_pango();
            data.append_full_text(&format!("Rx:<tt>{}</tt> Tx:<tt>{}</tt>", rx, tx));
            return Some(WidgetUpdate {
                refresh_interval: std::time::Duration::new(1, 0),
                data: Some(data),
            });
        }
        None
    }
}
