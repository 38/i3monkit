use i3monkit::widgets::*;
use i3monkit::{Header, I3Protocol, WidgetCollection};

fn main() {
    let p = I3Protocol::new(Header::new(1), std::io::stdout());

    let mut bar = WidgetCollection::new();

    let client = StockClient::new("alpha-vantage-api-key");
    bar.push(StockClient::create_widget(&client, "MSFT"));
    bar.push(StockClient::create_widget(&client, "AMD"));
    bar.push(StockClient::create_widget(&client, "FB"));
    bar.push(StockClient::create_widget(&client, "AMZN"));
    bar.push(StockClient::create_widget(&client, "GOOG"));
    bar.push(StockClient::create_widget(&client, "AAPL"));
    bar.push(StockClient::create_widget(&client, "NVDA"));

    bar.push(NetworkSpeedWidget::new("wlp58s0"));

    for i in 0..4 {
        bar.push(CpuWidget::new(i));
    }

    bar.push(VolumeWidget::new("default", "Master", 0));
    bar.push(BatteryWidget::new(0));
    bar.push(DateTimeWidget::new());

    bar.update_loop(p);
}
