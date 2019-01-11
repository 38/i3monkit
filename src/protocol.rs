use std::io::{BufWriter, Write};
use serde::{Serialize, Serializer};
use serde_derive::Serialize;

#[derive(Serialize, Default)]
pub struct Header {
    version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_signal: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cont_signal: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    click_events: Option<bool>,
}

impl Header {
    pub fn new(version:u32) -> Self {
        let mut ret = Self::default();
        ret.version = version;
        return ret;
    }
}

#[derive(Clone)]
pub struct ColorRGB(pub u8,pub u8,pub u8);

impl Serialize for ColorRGB {
    fn serialize<S:Serializer>(&self, s:S) -> Result<S::Ok, S::Error> {
        let color_string = format!("#{:.2x}{:02x}{:02x}", self.0, self.1, self.2);
        s.serialize_str(&color_string)
    }
}

impl ColorRGB {
    pub fn red() -> Self { 
        ColorRGB(255, 0, 0)
    }

    #[allow(dead_code)]
    pub fn green() -> Self {
        ColorRGB(0, 255, 0)
    }

    #[allow(dead_code)]
    pub fn blue() -> Self {
        ColorRGB(0, 0, 255)
    }

    pub fn yellow() -> Self {
        ColorRGB(255, 255, 0)
    }
}

#[derive(Debug, Clone)]
pub enum MarkupLang {
    Text,
    Pango,
}

impl Serialize for  MarkupLang {
    fn serialize<S:Serializer>(&self, s:S) -> Result<S::Ok, S::Error> {
        match self {
            MarkupLang::Text => s.serialize_str("none"),
            MarkupLang::Pango => s.serialize_str("pango"),
        }
    }
}

#[derive(Serialize, Clone)] 
pub struct Block {
    full_text : String,
    #[serde(skip_serializing_if = "String::is_empty")]
    short_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    color     : Option<ColorRGB>,
    markup    : MarkupLang,
}

impl Block {
    pub fn new() -> Self {
        Self {
            full_text: "".to_string(),
            short_text: "".to_string(),
            color: None,
            markup: MarkupLang::Text
        }
    }

    #[allow(dead_code)]
    pub fn full_text(&mut self, text:&str) -> &mut Self {
        self.full_text = text.to_string();
        self
    }

    pub fn append_full_text(&mut self, text:&str) -> &mut Self {
       self.full_text.push_str(text);
        self
    }

    #[allow(dead_code)]
    pub fn short_text(&mut self, text:&str) -> &mut Self {
        self.short_text = text.to_string();
        self
    }

    pub fn color(&mut self, color:ColorRGB) -> &mut Self {
        self.color = Some(color);
        self
    }

    #[allow(dead_code)]
    pub fn clear_color(&mut self) -> &mut Self {
        self.color = None;
        self
    }

    pub fn use_pango(&mut self) -> &mut Self {
        self.markup = MarkupLang::Pango;
        self
    }

    #[allow(dead_code)]
    pub fn use_plain_text(&mut self) -> &mut Self {
        self.markup = MarkupLang::Text;
        self
    }
}

pub struct I3Protocol<T:Write>(BufWriter<T>);

impl <T:Write> I3Protocol<T> {
    fn write<S:AsRef<str>>(&mut self, data:S) {
        self.0.write_all(AsRef::<str>::as_ref(&data).as_bytes()).expect("Cannot write");
        self.0.write_all(b"\n").expect("Cannot write");
        self.0.flush().ok();
    }
    fn write_json<S:Serialize>(&mut self, data:&S) {
        if let Ok(serialized) = serde_json::to_string(data) {
            self.write(serialized);
        }
    }

    pub fn new(header:Header, wr:T) -> Self {
        let mut ret = I3Protocol(BufWriter::new(wr));
        ret.write_json(&header);
        ret.write("[ []");
        ret
    }

    pub fn refresh(&mut self, status:&Vec<Block>) {
        self.write(",");
        self.write_json(status)
    }
}

impl <T:Write> Drop for I3Protocol<T> {
    fn drop(&mut self) {
        self.write("]");
    }
}
