use reqwest::Client;
use serde_json::Value;
use std::{
    error,
    fmt::{self, Display, Formatter},
    sync::OnceLock,
};

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub const ALL: [Self; 11] = [
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

    pub const fn as_str(self) -> &'static str {
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

    const fn interval(self) -> &'static str {
        match self {
            Self::OneDay => "1m",
            Self::FiveDays => "15m",
            Self::OneMonth | Self::ThreeMonths | Self::SixMonths | Self::OneYear | Self::Ytd => {
                "1d"
            }
            Self::TwoYears | Self::FiveYears => "1wk",
            Self::TenYears | Self::Max => "1mo",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Stock {
    pub symbol: String,
    pub long_name: String,
    pub short_name: String,
    pub currency: String,
    pub price: f64,
    pub open: Vec<f64>,
}

impl Stock {
    pub fn name(&self) -> &str {
        if !self.long_name.is_empty() {
            &self.long_name
        } else if !self.short_name.is_empty() {
            &self.short_name
        } else {
            &self.symbol
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Request(reqwest::Error),
    Parse,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Request(error) => write!(f, "request failed: {error}"),
            Self::Parse => write!(f, "failed to parse the response"),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Request(error) => Some(error),
            Self::Parse => None,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::Request(error)
    }
}

fn client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("building the HTTP client should succeed")
    })
}

fn parse(text: &str) -> Option<Stock> {
    let json: Value = serde_json::from_str(text).ok()?;
    let result = json.get("chart")?.get("result")?.get(0)?;
    let meta = result.get("meta")?;

    let symbol = meta.get("symbol")?.as_str()?.to_owned();
    let long_name = meta
        .get("longName")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_owned();
    let short_name = meta
        .get("shortName")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_owned();
    let currency = meta
        .get("currency")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_owned();
    let price = meta
        .get("regularMarketPrice")
        .and_then(|value| value.as_f64())
        .unwrap_or_default();
    let open = result
        .get("indicators")?
        .get("quote")?
        .get(0)?
        .get("open")?
        .as_array()?
        .iter()
        .filter_map(Value::as_f64)
        .collect();

    Some(Stock {
        symbol,
        long_name,
        short_name,
        currency,
        price,
        open,
    })
}

pub async fn fetch_stock(symbol: &str, range: Range) -> Result<Stock, Error> {
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?range={}&interval={}",
        symbol,
        range.as_str(),
        range.interval()
    );
    let text = client().get(url).send().await?.text().await?;

    parse(&text).ok_or(Error::Parse)
}
