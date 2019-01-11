use i3monkit::*;
use i3monkit::widgets::*;

fn main() {
    let mut bar = WidgetCollection::new();

    //Add realtime stock prices, for example: Microsoft, AMD and Facebook
    let stock_client = StockClient::new("your-alphavantage-API-key");
    bar.push(StockClient::create_widget(&stock_client, "MSFT"));
    bar.push(StockClient::create_widget(&stock_client, "AMD"));
    bar.push(StockClient::create_widget(&stock_client, "FB"));

    //Realtime upload/download rate for a interface
    bar.push(NetworkSpeedWidget::new("wlp58s0"));

    //Display all the cpu usage for each core
    for i in 0..4 {
        bar.push(CpuWidget::new(i));
    }

    //Volume widget
    bar.push(VolumeWidget::new("default", "Master", 0));

    //Battery status
    bar.push(BatteryWidget::new(0));

    //Time
    bar.push(DateTimeWidget::new());

    // Then start updating the satus bar
    bar.update_loop(I3Protocol::new(Header::new(1), std::io::stdout()));
}
                                                                             
