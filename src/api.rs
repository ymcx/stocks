use reqwest::Client;
use serde_json::Value;
use std::sync::OnceLock;

#[derive(Clone, Copy)]
pub enum Range {
    OneDay,
    FiveDays,
    OneMonth,
    ThreeMonths,
    SixMonths,
    OneYear,
    TwoYears,
    FiveYears,
    TenYears,
    Ytd,
    Max,
}

impl Range {
    pub const VALUES: [Self; 11] = [
        Self::OneDay,
        Self::FiveDays,
        Self::OneMonth,
        Self::ThreeMonths,
        Self::SixMonths,
        Self::OneYear,
        Self::TwoYears,
        Self::FiveYears,
        Self::TenYears,
        Self::Ytd,
        Self::Max,
    ];

    pub fn as_str(&self) -> &str {
        match self {
            Self::OneDay => "1d",
            Self::FiveDays => "5d",
            Self::OneMonth => "1mo",
            Self::ThreeMonths => "3mo",
            Self::SixMonths => "6mo",
            Self::OneYear => "1y",
            Self::TwoYears => "2y",
            Self::FiveYears => "5y",
            Self::TenYears => "10y",
            Self::Ytd => "ytd",
            Self::Max => "max",
        }
    }

    pub fn parse(string: &str) -> Option<Self> {
        match string {
            "1d" => Some(Self::OneDay),
            "5d" => Some(Self::FiveDays),
            "1mo" => Some(Self::OneMonth),
            "3mo" => Some(Self::ThreeMonths),
            "6mo" => Some(Self::SixMonths),
            "1y" => Some(Self::OneYear),
            "2y" => Some(Self::TwoYears),
            "5y" => Some(Self::FiveYears),
            "10y" => Some(Self::TenYears),
            "ytd" => Some(Self::Ytd),
            "max" => Some(Self::Max),
            _ => None,
        }
    }

    pub fn get_interval(&self) -> Interval {
        match self {
            Self::OneDay => Interval::OneMinute,
            Self::FiveDays => Interval::FiveMinutes,
            Self::OneMonth | Self::ThreeMonths | Self::SixMonths | Self::OneYear | Self::Ytd => {
                Interval::OneDay
            }
            Self::TwoYears | Self::FiveYears => Interval::OneWeek,
            Self::TenYears | Self::Max => Interval::OneMonth,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Interval {
    OneMinute,
    FiveMinutes,
    OneDay,
    OneWeek,
    OneMonth,
    ThreeMonths,
}

impl Interval {
    #[allow(dead_code)]
    pub const VALUES: [Self; 6] = [
        Self::OneMinute,
        Self::FiveMinutes,
        Self::OneDay,
        Self::OneWeek,
        Self::OneMonth,
        Self::ThreeMonths,
    ];

    pub fn as_str(&self) -> &str {
        match self {
            Self::OneMinute => "1m",
            Self::FiveMinutes => "5m",
            Self::OneDay => "1d",
            Self::OneWeek => "1wk",
            Self::OneMonth => "1mo",
            Self::ThreeMonths => "3mo",
        }
    }

    pub fn parse(string: &str) -> Option<Self> {
        match string {
            "1m" => Some(Self::OneMinute),
            "5m" => Some(Self::FiveMinutes),
            "1d" => Some(Self::OneDay),
            "1wk" => Some(Self::OneWeek),
            "1mo" => Some(Self::OneMonth),
            "3mo" => Some(Self::ThreeMonths),
            _ => None,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct TradingPeriod {
    pub timezone: String,
    pub start: i64,
    pub end: i64,
    pub gmtoffset: i64,
}

impl TradingPeriod {
    pub fn new(timezone: &str, start: i64, end: i64, gmtoffset: i64) -> Self {
        let timezone = timezone.to_string();
        Self {
            timezone,
            start,
            end,
            gmtoffset,
        }
    }

    pub fn parse(json: &Value) -> Option<Self> {
        let timezone = json.get("timezone")?.as_str()?;
        let start = json.get("start")?.as_i64()?;
        let end = json.get("end")?.as_i64()?;
        let gmtoffset = json.get("gmtoffset")?.as_i64()?;
        let trading_period = Self::new(timezone, start, end, gmtoffset);

        Some(trading_period)
    }
}

#[allow(dead_code)]
#[derive(Clone)]
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
    pub fn new(
        currency: &str,
        symbol: &str,
        exchange_name: &str,
        full_exchange_name: &str,
        instrument_type: &str,
        first_trade_date: i64,
        regular_market_time: i64,
        has_pre_post_market_data: bool,
        gmtoffset: i64,
        timezone: &str,
        exchange_timezone_name: &str,
        regular_market_price: f64,
        regular_market_change_percent: f64,
        fullday_price: f64,
        fullday_change: f64,
        fullday_change_percent: f64,
        fifty_two_week_high: f64,
        fifty_two_week_low: f64,
        regular_market_day_high: f64,
        regular_market_day_low: f64,
        regular_market_volume: i64,
        long_name: &str,
        short_name: &str,
        chart_previous_close: f64,
        price_hint: i64,
        current_trading_period_pre: TradingPeriod,
        current_trading_period_regular: TradingPeriod,
        current_trading_period_post: TradingPeriod,
        data_granularity: Interval,
        range: Range,
        valid_ranges: Vec<Range>,
        timestamp: Vec<i64>,
        quote_low: Vec<f64>,
        quote_volume: Vec<i64>,
        quote_close: Vec<f64>,
        quote_open: Vec<f64>,
        quote_high: Vec<f64>,
    ) -> Self {
        let currency = currency.to_string();
        let symbol = symbol.to_string();
        let exchange_name = exchange_name.to_string();
        let full_exchange_name = full_exchange_name.to_string();
        let instrument_type = instrument_type.to_string();
        let timezone = timezone.to_string();
        let exchange_timezone_name = exchange_timezone_name.to_string();
        let long_name = long_name.to_string();
        let short_name = short_name.to_string();

        Self {
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
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        let json: Value = serde_json::from_str(text).ok()?;
        let result = json.get("chart")?.get("result")?.get(0)?;
        let meta = result.get("meta")?;
        let timestamp = result
            .get("timestamp")?
            .as_array()?
            .into_iter()
            .filter_map(Value::as_i64)
            .collect();
        let quote = result.get("indicators")?.get("quote")?.get(0)?;

        let currency = meta.get("currency")?.as_str()?;
        let symbol = meta.get("symbol")?.as_str()?;
        let exchange_name = meta.get("exchangeName")?.as_str()?;
        let full_exchange_name = meta.get("fullExchangeName")?.as_str()?;
        let instrument_type = meta.get("instrumentType")?.as_str()?;
        let first_trade_date = meta.get("firstTradeDate")?.as_i64()?;
        let regular_market_time = meta.get("regularMarketTime")?.as_i64()?;
        let has_pre_post_market_data = meta.get("hasPrePostMarketData")?.as_bool()?;
        let gmtoffset = meta.get("gmtoffset")?.as_i64()?;
        let timezone = meta.get("timezone")?.as_str()?;
        let exchange_timezone_name = meta.get("exchangeTimezoneName")?.as_str()?;
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
        let long_name = meta.get("longName")?.as_str()?;
        let short_name = meta.get("shortName")?.as_str()?;
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
            .into_iter()
            .filter_map(|i| Range::parse(i.as_str().unwrap_or_default()))
            .collect();

        let quote_low = quote
            .get("low")?
            .as_array()?
            .into_iter()
            .filter_map(Value::as_f64)
            .collect();
        let quote_volume = quote
            .get("volume")?
            .as_array()?
            .into_iter()
            .filter_map(Value::as_i64)
            .collect();
        let quote_close = quote
            .get("close")?
            .as_array()?
            .into_iter()
            .filter_map(Value::as_f64)
            .collect();
        let quote_open = quote
            .get("open")?
            .as_array()?
            .into_iter()
            .filter_map(Value::as_f64)
            .collect();
        let quote_high = quote
            .get("high")?
            .as_array()?
            .into_iter()
            .filter_map(Value::as_f64)
            .collect();

        let stock = Self::new(
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
        );

        Some(stock)
    }

    pub async fn fetch(symbol: &str, range: Range) -> Option<Self> {
        let interval = range.get_interval();
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?range={}&interval={}",
            symbol,
            range.as_str(),
            interval.as_str(),
        );
        let client = client();
        let response = client.get(url).send().await.ok()?;
        let text = response.text().await.ok()?;

        Self::parse(&text)
    }

    pub async fn is_valid(symbol: &str) -> bool {
        Self::fetch(symbol, Range::OneDay).await.is_some()
    }
}

fn client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    let user_agent = "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36";
    let client = CLIENT.get_or_init(|| Client::builder().user_agent(user_agent).build().unwrap());

    client
}
