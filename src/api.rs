use reqwest::Client;
use serde_json::Value;
use std::sync::OnceLock;
use tokio::sync::mpsc::Sender;

#[allow(dead_code)]
#[derive(Clone, Copy, Default)]
pub enum Range {
    OneDay,
    FiveDays,
    #[default]
    OneMonth,
    ThreeMonths,
    SixMonths,
    Ytd,
    OneYear,
    TwoYears,
    FiveYears,
    TenYears,
    Max,
}

impl Range {
    pub fn as_api_str(&self) -> &str {
        match self {
            Range::OneDay => "1d",
            Range::FiveDays => "5d",
            Range::OneMonth => "1mo",
            Range::ThreeMonths => "3mo",
            Range::SixMonths => "6mo",
            Range::Ytd => "ytd",
            Range::OneYear => "1y",
            Range::TwoYears => "2y",
            Range::FiveYears => "5y",
            Range::TenYears => "10y",
            Range::Max => "max",
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Range::OneDay => "1D",
            Range::FiveDays => "5D",
            Range::OneMonth => "1M",
            Range::ThreeMonths => "3M",
            Range::SixMonths => "6M",
            Range::Ytd => "YTD",
            Range::OneYear => "1Y",
            Range::TwoYears => "2Y",
            Range::FiveYears => "5Y",
            Range::TenYears => "10Y",
            Range::Max => "All",
        }
    }

    pub fn parse(string: &str) -> Option<Range> {
        match string {
            "1D" | "1d" => Some(Range::OneDay),
            "5D" | "5d" => Some(Range::FiveDays),
            "1M" | "1mo" => Some(Range::OneMonth),
            "3M" | "3mo" => Some(Range::ThreeMonths),
            "6M" | "6mo" => Some(Range::SixMonths),
            "YTD" | "ytd" => Some(Range::Ytd),
            "1Y" | "1y" => Some(Range::OneYear),
            "2Y" | "2y" => Some(Range::TwoYears),
            "5Y" | "5y" => Some(Range::FiveYears),
            "10Y" | "10y" => Some(Range::TenYears),
            "All" | "max" => Some(Range::Max),
            _ => None,
        }
    }

    // The absolute minimum Yahoo accepts for each range is given in the comment.
    // Using the most granular interval is not preferred, since it increases
    // latency for us, is costly for Yahoo, lags our machine trying to render a
    // gazillion points and is of little benefit.
    pub fn get_interval(&self) -> Option<Interval> {
        match self {
            Range::OneDay => Some(Interval::FiveMinutes), // Interval::OneMinute
            Range::FiveDays => Some(Interval::ThirtyMinutes), // Interval::OneMinute
            Range::OneMonth => Some(Interval::FourHours), // Interval::TwoMinutes
            Range::ThreeMonths => Some(Interval::OneDay), // Interval::OneHour
            Range::SixMonths => Some(Interval::OneDay),   // Interval::OneHour
            Range::Ytd => Some(Interval::OneDay),         // Interval::OneHour
            Range::OneYear => Some(Interval::OneDay),     // Interval::OneHour
            Range::TwoYears => Some(Interval::FiveDays),  // Interval::OneHour
            Range::FiveYears => Some(Interval::OneWeek),  // Interval::OneDay
            Range::TenYears => Some(Interval::OneMonth),  // Interval::OneDay
            Range::Max => None,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Default)]
pub enum Interval {
    OneMinute,
    TwoMinutes,
    FiveMinutes,
    FifteenMinutes,
    #[default]
    ThirtyMinutes,
    SixtyMinutes,
    NinetyMinutes,
    OneHour,
    FourHours,
    OneDay,
    FiveDays,
    OneWeek,
    OneMonth,
    ThreeMonths,
}

impl Interval {
    pub fn as_api_str(&self) -> &str {
        match self {
            Interval::OneMinute => "1m",
            Interval::TwoMinutes => "2m",
            Interval::FiveMinutes => "5m",
            Interval::FifteenMinutes => "15m",
            Interval::ThirtyMinutes => "30m",
            Interval::SixtyMinutes => "60m",
            Interval::NinetyMinutes => "90m",
            Interval::OneHour => "1h",
            Interval::FourHours => "4h",
            Interval::OneDay => "1d",
            Interval::FiveDays => "5d",
            Interval::OneWeek => "1wk",
            Interval::OneMonth => "1mo",
            Interval::ThreeMonths => "3mo",
        }
    }

    pub fn parse(string: &str) -> Option<Interval> {
        match string {
            "1m" => Some(Interval::OneMinute),
            "2m" => Some(Interval::TwoMinutes),
            "5m" => Some(Interval::FiveMinutes),
            "15m" => Some(Interval::FifteenMinutes),
            "30m" => Some(Interval::ThirtyMinutes),
            "60m" => Some(Interval::SixtyMinutes),
            "90m" => Some(Interval::NinetyMinutes),
            "1h" => Some(Interval::OneHour),
            "4h" => Some(Interval::FourHours),
            "1d" => Some(Interval::OneDay),
            "5d" => Some(Interval::FiveDays),
            "1wk" => Some(Interval::OneWeek),
            "1mo" => Some(Interval::OneMonth),
            "3mo" => Some(Interval::ThreeMonths),
            _ => None,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Default)]
pub struct TradingPeriod {
    pub timezone: String,
    pub start: i64,
    pub end: i64,
    pub gmtoffset: i64,
}

impl TradingPeriod {
    pub fn new(timezone: &str, start: i64, end: i64, gmtoffset: i64) -> TradingPeriod {
        let timezone = timezone.to_string();
        TradingPeriod {
            timezone,
            start,
            end,
            gmtoffset,
        }
    }

    pub fn parse(json: &Value) -> Option<TradingPeriod> {
        let timezone = json.get("timezone")?.as_str()?;
        let start = json.get("start")?.as_i64()?;
        let end = json.get("end")?.as_i64()?;
        let gmtoffset = json.get("gmtoffset")?.as_i64()?;
        let trading_period = TradingPeriod::new(timezone, start, end, gmtoffset);

        Some(trading_period)
    }
}

#[allow(dead_code)]
#[derive(Clone, Default)]
pub struct Stock {
    pub currency: String,
    pub symbol: String,
    pub exchange_name: String,
    pub full_exchange_name: String,
    pub instrument_type: String,
    pub first_trade_date: i64,
    pub regular_market_time: i64,
    pub has_pre_post_market_data: bool,
    pub gmtoffset: i64,
    pub timezone: String,
    pub exchange_timezone_name: String,
    pub regular_market_price: f64,
    pub regular_market_change_percent: f64,
    pub fullday_price: f64,
    pub fullday_change: f64,
    pub fullday_change_percent: f64,
    pub fifty_two_week_high: f64,
    pub fifty_two_week_low: f64,
    pub regular_market_day_high: f64,
    pub regular_market_day_low: f64,
    pub regular_market_volume: i64,
    pub long_name: String,
    pub short_name: String,
    pub chart_previous_close: f64,
    pub price_hint: i64,
    pub current_trading_period_pre: TradingPeriod,
    pub current_trading_period_regular: TradingPeriod,
    pub current_trading_period_post: TradingPeriod,
    pub data_granularity: Interval,
    pub range: Range,
    pub valid_ranges: Vec<Range>,
    pub timestamp: Vec<i64>,
    pub quote_low: Vec<f64>,
    pub quote_volume: Vec<i64>,
    pub quote_close: Vec<f64>,
    pub quote_open: Vec<f64>,
    pub quote_high: Vec<f64>,
}

impl Stock {
    pub fn parse(text: &str) -> Option<Stock> {
        let json: Value = serde_json::from_str(text).ok()?;
        let result = json.get("chart")?.get("result")?.get(0)?;
        let meta = result.get("meta")?;
        let timestamp = result
            .get("timestamp")?
            .as_array()?
            .iter()
            .filter_map(Value::as_i64)
            .collect();
        let quote = result.get("indicators")?.get("quote")?.get(0)?;

        let currency = meta.get("currency")?.as_str()?.to_string();
        let symbol = meta.get("symbol")?.as_str()?.to_string();
        let exchange_name = meta.get("exchangeName")?.as_str()?.to_string();
        let full_exchange_name = meta.get("fullExchangeName")?.as_str()?.to_string();
        let instrument_type = meta.get("instrumentType")?.as_str()?.to_string();
        let first_trade_date = meta.get("firstTradeDate")?.as_i64()?;
        let regular_market_time = meta.get("regularMarketTime")?.as_i64()?;
        let has_pre_post_market_data = meta.get("hasPrePostMarketData")?.as_bool()?;
        let gmtoffset = meta.get("gmtoffset")?.as_i64()?;
        let timezone = meta.get("timezone")?.as_str()?.to_string();
        let exchange_timezone_name = meta.get("exchangeTimezoneName")?.as_str()?.to_string();
        let regular_market_price = meta.get("regularMarketPrice")?.as_f64()?;
        let regular_market_change_percent = meta.get("regularMarketChangePercent")?.as_f64()?;
        let fullday_price = meta.get("fulldayPrice")?.as_f64()?;
        let fullday_change = meta.get("fulldayChange")?.as_f64()?;
        let fullday_change_percent = meta.get("fulldayChangePercent")?.as_f64()?;
        let fifty_two_week_high = meta.get("fiftyTwoWeekHigh")?.as_f64()?;
        let fifty_two_week_low = meta.get("fiftyTwoWeekLow")?.as_f64()?;
        let regular_market_day_high = meta.get("regularMarketDayHigh")?.as_f64()?;
        let regular_market_day_low = meta.get("regularMarketDayLow")?.as_f64()?;
        let regular_market_volume = meta.get("regularMarketVolume")?.as_i64()?;
        let long_name = meta.get("longName")?.as_str()?.to_string();
        let short_name = meta.get("shortName")?.as_str()?.to_string();
        let chart_previous_close = meta.get("chartPreviousClose")?.as_f64()?;
        let price_hint = meta.get("priceHint")?.as_i64()?;
        let current_trading_period = meta.get("currentTradingPeriod")?;
        let current_trading_period_pre = TradingPeriod::parse(current_trading_period.get("pre")?)?;
        let current_trading_period_regular =
            TradingPeriod::parse(current_trading_period.get("regular")?)?;
        let current_trading_period_post =
            TradingPeriod::parse(current_trading_period.get("post")?)?;
        let data_granularity = Interval::parse(meta.get("dataGranularity")?.as_str()?)?;
        let range = Range::parse(meta.get("range")?.as_str()?)?;
        let valid_ranges = meta
            .get("validRanges")?
            .as_array()?
            .iter()
            .filter_map(|i| Range::parse(i.as_str().unwrap_or_default()))
            .collect();

        let quote_low = quote
            .get("low")?
            .as_array()?
            .iter()
            .filter_map(Value::as_f64)
            .collect();
        let quote_volume = quote
            .get("volume")?
            .as_array()?
            .iter()
            .filter_map(Value::as_i64)
            .collect();
        let quote_close = quote
            .get("close")?
            .as_array()?
            .iter()
            .filter_map(Value::as_f64)
            .collect();
        let quote_open = quote
            .get("open")?
            .as_array()?
            .iter()
            .filter_map(Value::as_f64)
            .collect();
        let quote_high = quote
            .get("high")?
            .as_array()?
            .iter()
            .filter_map(Value::as_f64)
            .collect();

        let stock = Stock {
            currency,
            symbol,
            exchange_name,
            full_exchange_name,
            instrument_type,
            first_trade_date,
            regular_market_time,
            has_pre_post_market_data,
            gmtoffset,
            timezone,
            exchange_timezone_name,
            regular_market_price,
            regular_market_change_percent,
            fullday_price,
            fullday_change,
            fullday_change_percent,
            fifty_two_week_high,
            fifty_two_week_low,
            regular_market_day_high,
            regular_market_day_low,
            regular_market_volume,
            long_name,
            short_name,
            chart_previous_close,
            price_hint,
            current_trading_period_pre,
            current_trading_period_regular,
            current_trading_period_post,
            data_granularity,
            range,
            valid_ranges,
            timestamp,
            quote_low,
            quote_volume,
            quote_close,
            quote_open,
            quote_high,
        };

        Some(stock)
    }

    pub async fn fetch(symbol: &str, range: Range) -> Option<Stock> {
        let base = "https://query1.finance.yahoo.com/v8/finance/chart";
        let url = if let Some(interval) = range.get_interval() {
            let range = range.as_api_str();
            let interval = interval.as_api_str();
            format!("{}/{}?range={}&interval={}", base, symbol, range, interval)
        } else {
            let range = range.as_api_str();
            format!("{}/{}?range={}", base, symbol, range)
        };

        let client = client();
        let response = client.get(url).send().await.ok()?;
        let text = response.text().await.ok()?;

        Stock::parse(&text)
    }

    pub async fn fetch_and_send(symbol: String, range: Range, sender: Sender<Stock>) {
        if let Some(stock) = Stock::fetch(&symbol, range).await {
            let _ = sender.send(stock).await;
        }
    }
}

fn client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    let user_agent = "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36";

    CLIENT.get_or_init(|| Client::builder().user_agent(user_agent).build().unwrap())
}
