use reqwest::Client;

#[allow(dead_code)]
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
    YTD,
    MAX,
}

#[allow(dead_code)]
pub enum Interval {
    OneMinute,
    TwoMinutes,
    FiveMinutes,
    FifteenMinutes,
    ThirtyMinutes,
    SixtyMinutes,
    NinetyMinutes,
    OneHour,
    OneDay,
    FiveDays,
    OneWeek,
    OneMonth,
    ThreeMonths,
}

impl Range {
    fn as_str(&self) -> &'static str {
        match self {
            Range::OneDay => "1d",
            Range::FiveDays => "5d",
            Range::OneMonth => "1mo",
            Range::ThreeMonths => "3mo",
            Range::SixMonths => "6mo",
            Range::OneYear => "1y",
            Range::TwoYears => "2y",
            Range::FiveYears => "5y",
            Range::TenYears => "10y",
            Range::YTD => "ytd",
            Range::MAX => "max",
        }
    }
}
impl Interval {
    fn as_str(&self) -> &'static str {
        match self {
            Interval::OneMinute => "1m",
            Interval::TwoMinutes => "2m",
            Interval::FiveMinutes => "5m",
            Interval::FifteenMinutes => "15m",
            Interval::ThirtyMinutes => "30m",
            Interval::SixtyMinutes => "60m",
            Interval::NinetyMinutes => "90m",
            Interval::OneHour => "1h",
            Interval::OneDay => "1d",
            Interval::FiveDays => "5d",
            Interval::OneWeek => "1wk",
            Interval::OneMonth => "1mo",
            Interval::ThreeMonths => "3mo",
        }
    }
}

fn parse(text: &str) -> Option<(String, String, String, String, f64, Vec<f64>)> {
    let json: serde_json::Value = serde_json::from_str(&text).ok()?;
    let result = json.get("chart")?.get("result")?.get(0)?;
    let meta = result.get("meta")?;
    let indicators = result.get("indicators")?;
    let symbol = meta.get("symbol")?.as_str()?.to_string();
    let long_name = meta.get("longName")?.as_str()?.to_string();
    let short_name = meta.get("shortName")?.as_str()?.to_string();
    let currency = meta.get("currency")?.as_str()?.to_string();
    let price = meta.get("regularMarketPrice")?.as_f64()?;
    let open = indicators
        .get("quote")?
        .get(0)?
        .get("open")?
        .as_array()?
        .iter()
        .map(|i| i.as_f64().unwrap())
        .collect();

    Some((symbol, long_name, short_name, currency, price, open))
}

pub async fn fetch_stock(
    symbol: &str,
    interval: Interval,
    range: Range,
) -> Option<(String, String, String, String, f64, Vec<f64>)> {
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval={}&range={}",
        symbol,
        interval.as_str(),
        range.as_str()
    );
    let ua = "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36";
    let client = Client::builder().user_agent(ua).build().ok()?;
    let text = client.get(url).send().await.ok()?.text().await.ok()?;
    let paska = parse(&text);

    paska
}
