use i3monkit::*;

struct Greeter(&'static str);

impl Widget for Greeter {
    fn update(&mut self) -> Option<WidgetUpdate> {
        Some(WidgetUpdate {
            refresh_interval: std::time::Duration::new(1,0),
            data: Some(Block::new().append_full_text(self.0).clone())
        })
    }
}

fn main() {

    let mut bar = WidgetCollection::new();

    bar.push(Greeter("Hello World"));

    bar.update_loop(I3Protocol::new(Header::new(1), std::io::stdout()));
}
