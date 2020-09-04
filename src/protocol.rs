//! The abstraction for [i3 bar protcol](https://i3wm.org/docs/i3bar-protocol.html)

use serde::{Serialize, Serializer};
use std::io::{BufWriter, Write};

/// The I3 protocol header
#[derive(Serialize, Default)]
pub struct Header {
    /// Version number, currently must be 1
    version: u32,
    /// Sepecify the signal for stopping the program
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_signal: Option<u32>,
    /// Specify the signal for resume the program
    #[serde(skip_serializing_if = "Option::is_none")]
    cont_signal: Option<u32>,
    /// If the mouse event is enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    click_events: Option<bool>,
}

impl Header {
    /// Create a new instance of protocol header
    ///
    /// **version** The version number of protocol, which should be 1
    pub fn new(version: u32) -> Self {
        let mut ret = Self::default();
        ret.version = version;
        return ret;
    }

    pub fn click_events(mut self, enable: bool) -> Self {
        self.click_events = Some(enable);
        self
    }
}

/// An RGB color
#[derive(Clone)]
pub struct ColorRGB(pub u8, pub u8, pub u8);

impl Serialize for ColorRGB {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let color_string = format!("#{:.2x}{:02x}{:02x}", self.0, self.1, self.2);
        s.serialize_str(&color_string)
    }
}

impl ColorRGB {
    pub fn red() -> Self {
        ColorRGB(255, 0, 0)
    }

    pub fn green() -> Self {
        ColorRGB(0, 255, 0)
    }

    pub fn blue() -> Self {
        ColorRGB(0, 0, 255)
    }

    pub fn yellow() -> Self {
        ColorRGB(255, 255, 0)
    }
}

/// The option indicate what markup language should the i3bar use to parse the output
#[derive(Debug, Clone)]
pub enum MarkupLang {
    /// Use plain-text
    Text,
    /// Use the Pango markup language
    Pango,
}

impl Serialize for MarkupLang {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            MarkupLang::Text => s.serialize_str("none"),
            MarkupLang::Pango => s.serialize_str("pango"),
        }
    }
}

/// A block shown on the I3 status bar
#[derive(Serialize, Clone)]
pub struct Block {
    name : String,
    instance : String,
    /// The full text shown on the bar
    full_text: String,
    /// A short alternative for the message
    #[serde(skip_serializing_if = "String::is_empty")]
    short_text: String,
    /// The text color
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<ColorRGB>,
    /// The markup language options
    markup: MarkupLang,
}

impl Block {
    /// Create a new block
    pub fn new() -> Self {
        Self {
            name : "".to_string(),
            instance : "".to_string(),
            full_text: "".to_string(),
            short_text: "".to_string(),
            color: None,
            markup: MarkupLang::Text,
        }
    }

    pub fn instance(&mut self, text: &str) -> &mut Self {
        self.instance = text.to_string();
        self
    }

    pub fn name(&mut self, text: &str) -> &mut Self {
        self.name = text.to_string();
        self
    }

    /// Set the full text of the block
    #[allow(dead_code)]
    pub fn full_text(&mut self, text: &str) -> &mut Self {
        self.full_text = text.to_string();
        self
    }

    /// Append text to the full text
    pub fn append_full_text(&mut self, text: &str) -> &mut Self {
        self.full_text.push_str(text);
        self
    }

    /// Set the short text
    pub fn short_text(&mut self, text: &str) -> &mut Self {
        self.short_text = text.to_string();
        self
    }

    /// Set the foreground color
    pub fn color(&mut self, color: ColorRGB) -> &mut Self {
        self.color = Some(color);
        self
    }

    /// Clear the color option and use default color
    pub fn clear_color(&mut self) -> &mut Self {
        self.color = None;
        self
    }

    /// Make the block uses the pango markup language
    pub fn use_pango(&mut self) -> &mut Self {
        self.markup = MarkupLang::Pango;
        self
    }

    /// Make the block uses plain text
    pub fn use_plain_text(&mut self) -> &mut Self {
        self.markup = MarkupLang::Text;
        self
    }
}

/// The abstraction for a i3 protocol instance
pub struct I3Protocol<T: Write>(BufWriter<T>);

impl<T: Write> I3Protocol<T> {
    fn write<S: AsRef<str>>(&mut self, data: S) {
        self.0
            .write_all(AsRef::<str>::as_ref(&data).as_bytes())
            .expect("Cannot write");
        self.0.write_all(b"\n").expect("Cannot write");
        self.0.flush().ok();
    }
    fn write_json<S: Serialize>(&mut self, data: &S) {
        if let Ok(serialized) = serde_json::to_string(data) {
            self.write(serialized);
        }
    }

    /// Create a i3 protocol instance
    ///
    /// **wr** Where the protocol message should be dumped
    pub fn new(header: Header, wr: T) -> Self {
        let mut ret = I3Protocol(BufWriter::new(wr));
        ret.write_json(&header);
        ret.write("[ []");
        ret
    }

    /// Refresh the bar
    ///
    /// **status** The new list of blocks `i3bar` should redraw
    pub fn refresh(&mut self, status: &Vec<Block>) {
        self.write(",");
        self.write_json(status)
    }
}

impl<T: Write> Drop for I3Protocol<T> {
    fn drop(&mut self) {
        self.write("]");
    }
}
