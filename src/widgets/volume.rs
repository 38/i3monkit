use crate::protocol::{Block, ColorRGB};
use crate::widget::{Widget, WidgetUpdate};

use alsa::mixer::{Mixer, Selem, SelemChannelId, SelemId};
use alsa::Result;

use std::ffi::CString;

/// The system volume widget
pub struct VolumeWidget {
    device: CString,
    #[allow(dead_code)]
    mixer: CString,
    selem_id: SelemId,
}

impl VolumeWidget {
    fn get_volume(&self) -> Result<Option<(bool, u32)>> {
        let mut handle = Mixer::open(false)?;
        handle.attach(self.device.as_c_str())?;
        Selem::register(&mut handle)?;
        handle.load()?;

        if let Some(selem) = handle.find_selem(&self.selem_id) {
            let (min, max) = selem.get_playback_volume_range();
            let vol = selem.get_playback_volume(SelemChannelId::FrontLeft)?;
            let mute = selem.get_playback_switch(SelemChannelId::FrontLeft)?;
            let vol = ((100 * (vol - min)) as f32 / (max - min) as f32).round() as u32;
            return Ok(Some((mute == 0, vol)));
        }

        return Ok(None);
    }

    /// Creates new widget for the given mixer and channel id
    pub fn new(device: &str, mixer: &str, idx: u32) -> Self {
        let device = CString::new(device).unwrap();
        let mixer = CString::new(mixer).unwrap();

        let mut selem_id = SelemId::empty();
        selem_id.set_name(mixer.as_c_str());
        selem_id.set_index(idx);

        Self {
            device,
            mixer,
            selem_id,
        }
    }
}

impl Widget for VolumeWidget {
    fn update(&mut self) -> Option<WidgetUpdate> {
        if let Ok(Some((mute, vol))) = self.get_volume() {
            let icon = if !mute { "🔊" } else { "🔇" };
            let status = format!("{}%{}", vol, icon);
            let mut data = Block::new().append_full_text(&status).clone();
            if mute {
                data.color(ColorRGB::yellow());
            }

            return Some(WidgetUpdate {
                refresh_interval: std::time::Duration::new(1, 0),
                data: Some(data),
            });
        }

        None
    }
}
