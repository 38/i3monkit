use crate::protocol::Block;
use crate::widget::{Widget, WidgetUpdate};

use curl::easy::Easy;
use serde::Deserialize;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime};

/// An simple Alpha Vantage client for realtime stock price
///
/// For details about the Alpha Vantage, see [their website](https://www.alphavantage.co/)
///
/// To use this client, you need an API key from Alpha Vantage.
/// For your own API key, Follow the instructions at
/// [https://www.alphavantage.co/support/#api-key](https://www.alphavantage.co/support/#api-key)
///
/// With the key, you can create an instance of the client. And you can use the client to create
/// stock widgets for specified stocks.
///
/// ```rust
///     let client = StockClient::new("your-api-key");
///
///     let msft_price_widget = StockClient::create_widget("MSFT");
///
/// ```
///
pub struct StockClient<'a> {
    symbols: Vec<&'a str>,
    api_key: &'a str,
    cache: HashMap<String, StockPrice>,
    refresh_thread: Option<JoinHandle<()>>,
    refresh_channel: Option<Receiver<HashMap<String, StockPrice>>>,
}
#[derive(Deserialize, Debug)]
struct RawStockPrice {
    #[serde(rename = "1. open")]
    open: String,
    #[serde(rename = "2. high")]
    high: String,
    #[serde(rename = "3. low")]
    low: String,
    #[serde(rename = "4. close")]
    close: String,
    #[serde(rename = "5. volume")]
    volume: String,
}

#[derive(Deserialize)]
struct Response {
    #[serde(rename = "Time Series (Daily)")]
    time_series: HashMap<String, RawStockPrice>,
}

#[derive(Debug)]
struct StockPrice {
    previous_close: f32,
    open: f32,
    high: f32,
    low: f32,
    close: f32,
    volume: f32,
}

/// The widget for realtime stock price
pub struct StockWidget<'a> {
    symbol: &'a str,
    client: Rc<RefCell<StockClient<'a>>>,
}

impl<'a> Widget for StockWidget<'a> {
    fn update(&mut self) -> Option<WidgetUpdate> {
        self.client.borrow_mut().refresh();
        let mut block = Block::new();
        block.use_pango();
        block.append_full_text(&format!(
            "<span foreground=\"#eaeaea\">{} </span>",
            self.symbol
        ));
        if let Some(latest) = self.client.borrow().cache.get(&self.symbol.to_string()) {
            let color = if latest.previous_close > latest.close {
                "#ff0000"
            } else if latest.previous_close < latest.close {
                "#00ff00"
            } else {
                "#ffffff"
            };

            block.append_full_text(&format!(
                "<span foreground=\"{color}\">{value:.2}({percent:.1}%)</span>",
                color = color,
                value = latest.close,
                percent =
                    100.0 * (latest.close - latest.previous_close).abs() / latest.previous_close
            ));
        } else {
            block.append_full_text("<span foreground=\"#777777\">waiting</span>");
        }

        return Some(WidgetUpdate {
            refresh_interval: std::time::Duration::new(1, 0),
            data: Some(block),
        });
    }
}

impl<'a> StockClient<'a> {
    /// Creates a new Alpha Vantage client
    pub fn new(api_key: &'a str) -> Rc<RefCell<Self>> {
        let client = Self {
            symbols: Vec::new(),
            api_key,
            cache: HashMap::new(),
            refresh_thread: None,
            refresh_channel: None,
        };
        return Rc::new(RefCell::new(client));
    }

    /// Get a widget that shows the stock price for given symbol
    ///
    /// **this** The stock client
    /// **symbol** The stock symbol to show
    ///
    pub fn create_widget(this: &Rc<RefCell<Self>>, symbol: &'a str) -> StockWidget<'a> {
        this.borrow_mut().push(symbol);
        return StockWidget {
            symbol,
            client: Rc::clone(this),
        };
    }

    fn ensure_refresh_started(&mut self) {
        if self.refresh_thread.is_none() {
            let mut symbols: Vec<_> = self
                .symbols
                .iter()
                .map(|x| (x.to_string(), SystemTime::now()))
                .collect();
            let api_key = self.api_key.to_string();

            let (sx, rx) = std::sync::mpsc::channel();

            let mut min_sleep_duration = Duration::new(1, 0);

            let thread_handle = std::thread::spawn(move || loop {
                let mut data = HashMap::<String, StockPrice>::new();

                for (symbol, next_update) in symbols.iter_mut() {
                    if *next_update > SystemTime::now() {
                        continue;
                    }
                    if let Some(price) = Self::query_latest(symbol, &api_key) {
                        data.insert(symbol.to_string(), price);
                        *next_update = SystemTime::now() + Duration::new(300, 0);
                        min_sleep_duration = Duration::new(1, 0);
                    }
                }

                symbols.sort_by_key(|(_, ts)| *ts);

                sx.send(data).ok();

                let next_wakeup = symbols.iter().min_by_key(|(_, ts)| ts).unwrap();

                if let Ok(period) = next_wakeup.1.duration_since(SystemTime::now()) {
                    std::thread::sleep(period);
                } else {
                    std::thread::sleep(min_sleep_duration);
                    if min_sleep_duration < Duration::new(60, 0) {
                        min_sleep_duration *= 2;
                    }
                }
            });

            self.refresh_thread = Some(thread_handle);
            self.refresh_channel = Some(rx);
        }
    }

    fn refresh(&mut self) {
        self.ensure_refresh_started();
        if let Some(ref mut rx) = self.refresh_channel {
            if let Ok(new_data) = rx.try_recv() {
                for (k, v) in new_data {
                    self.cache.insert(k, v);
                }
            }
        }
    }

    fn push(&mut self, symbol: &'a str) {
        self.symbols.push(symbol);
    }

    fn query_latest(symbol: &str, key: &str) -> Option<StockPrice> {
        let url = format!("https://{server}/query?function=TIME_SERIES_DAILY&symbol={symbol}&interval=5min&outputsize=compact&apikey={key}",
                          server = "www.alphavantage.co", symbol = symbol, key = key);

        let mut body = Vec::new();

        let mut request = move || -> Result<String, curl::Error> {
            let mut handle = Easy::new();
            {
                handle.url(&url)?;

                let mut transfer = handle.transfer();
                transfer.write_function(|data| {
                    body.extend_from_slice(data);
                    Ok(data.len())
                })?;
                transfer.perform()?;
            }
            return Ok(String::from_utf8_lossy(&body[..]).to_string());
        };

        if let Ok(body) = request() {
            if let Ok(response) = serde_json::from_str::<Response>(&body) {
                if let Some(latest_date) = response.time_series.keys().max() {
                    let latest = &response.time_series[latest_date];
                    let yesterday = response
                        .time_series
                        .keys()
                        .filter(|d| d != &latest_date)
                        .max();

                    let open = f32::from_str(&latest.open).unwrap();

                    let pc = if let Some(yesterday) = yesterday {
                        let yesterday = &response.time_series[yesterday];
                        f32::from_str(&yesterday.close).unwrap_or(0.0)
                    } else {
                        open
                    };

                    return Some(StockPrice {
                        previous_close: pc,
                        open,
                        high: f32::from_str(&latest.high).unwrap_or(0.0),
                        low: f32::from_str(&latest.low).unwrap_or(0.0),
                        close: f32::from_str(&latest.close).unwrap_or(0.0),
                        volume: f32::from_str(&latest.volume).unwrap_or(0.0),
                    });
                }
            }
        }

        return None;
    }
}
