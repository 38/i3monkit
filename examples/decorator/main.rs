use i3monkit::widgets::*;
use i3monkit::*;

fn main() {
    let mut bar = WidgetCollection::new();

    bar.push(DateTimeWidget::new().decorate_with(|b| {
        b.color(ColorRGB::red());
    }));

    bar.update_loop(I3Protocol::new(Header::new(1), std::io::stdout()));
}
