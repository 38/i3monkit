use crate::widget::{Widget, WidgetUpdate};
use crate::protocol::{Block};

use std::fs::File;
use std::io::{BufRead, BufReader, Result};

/// The CPU usage widget
///
/// This widget draws a CPU usage pertentage bar on your i3 status bar. 
pub struct CpuWidget {
    id  : u32,
    user: u64,
    nice: u64,
    system: u64,
    idel: u64,
    width: u8,
    user_color: String,
    nice_color: String,
    system_color: String,
}

impl CpuWidget {
    fn read_status(id:u32) -> Result<(u64,u64,u64,u64)> {
        let name = format!("cpu{}", id);
        let file = File::open("/proc/stat")?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            let tokens:Vec<_> = line.trim().split(|c| c == ' ' || c == '\t').collect();
            if tokens[0] == name {
                let parsed = tokens[1..5].iter().map(|x| u64::from_str_radix(x,10).unwrap()).collect::<Vec<_>>();
                return Ok((parsed[0], parsed[1], parsed[2], parsed[3]));
            }
        }

        Err(std::io::Error::new(std::io::ErrorKind::Other, "No such CPU core"))
    }

    /// Create a new CPU usage monitor widget for specified core
    ///
    /// **id** The core id
    pub fn new(id:u32) -> Self {
        let (user, nice, system, idel) = Self::read_status(id).unwrap();
        let ret = Self {
            id, user, nice, system, idel,
            width: 20,
            user_color: "#00ff00".to_string(),
            nice_color: "#0000ff".to_string(),
            system_color: "#ff0000".to_string(),
        };

        return ret;
    }

    fn draw_bar(&mut self) -> Option<String> {
        let mut ret = Vec::new();
        for _ in 0..self.width {
            ret.push("<span foreground=\"grey\">|</span>".to_string());
        }

        let (user, nice, system, idel) = Self::read_status(self.id).ok()?;

        let total_diff = (user + nice + system + idel) - (self.user + self.nice + self.system + self.idel);

        if total_diff > 0 {
            let diffs = [system - self.system, nice - self.nice, user - self.user];
            let color = [&self.system_color, &self.nice_color, &self.user_color];
            
            let mut idx = 0;
            for (d,c) in diffs.iter().zip(color.iter()) {
                for _ in 0..(d * (self.width as u64) / total_diff) {
                    ret[idx] = format!("<span foreground=\"{}\">|</span>", c);
                    idx += 1;
                }
            }
        }

        let mut result = String::new();
        for s in ret {
            result.push_str(&s);
        }

        self.nice = nice;
        self.user = user;
        self.idel = idel;
        self.system = system;

        return Some(result);
    }
}

impl Widget for CpuWidget {
    fn update(&mut self) -> Option<WidgetUpdate> {

        if let Some(bar) = self.draw_bar() {

            let mut data = Block::new();

            data.use_pango();
            data.append_full_text(&format!("{}[{}]", self.id + 1, bar));

            return Some(WidgetUpdate {
               refresh_interval: std::time::Duration::new(1, 0),
               data: Some(data)
            });
        }

        None
    }
}
