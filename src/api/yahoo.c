#include "src/api/yahoo.h"
#include "src/api/curl.h"
#include "src/model/price.h"
#include "src/model/stock.h"
#include <cjson/cJSON.h>
#include <curl/curl.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

Price **api_yahoo_get_prices(const char *data, size_t *prices_length) {
  if (!data || !prices_length) {
    return NULL;
  }

  cJSON *json_root = cJSON_Parse(data);
  if (!json_root) {
    return NULL;
  }

  cJSON *json = json_root;
  json = cJSON_GetObjectItemCaseSensitive(json, "chart");
  json = cJSON_GetObjectItemCaseSensitive(json, "result");
  json = cJSON_GetArrayItem(json, 0);
  json = cJSON_GetObjectItemCaseSensitive(json, "indicators");
  json = cJSON_GetObjectItemCaseSensitive(json, "quote");
  json = cJSON_GetArrayItem(json, 0);
  if (!json) {
    cJSON_Delete(json_root);
    return NULL;
  }

  const cJSON *json_close = cJSON_GetObjectItemCaseSensitive(json, "close");
  const cJSON *json_high = cJSON_GetObjectItemCaseSensitive(json, "high");
  const cJSON *json_low = cJSON_GetObjectItemCaseSensitive(json, "low");
  const cJSON *json_open = cJSON_GetObjectItemCaseSensitive(json, "open");
  const cJSON *json_volume = cJSON_GetObjectItemCaseSensitive(json, "volume");

  *prices_length = cJSON_GetArraySize(json_close);
  Price **prices = malloc(sizeof(Price *) * *prices_length);
  if (!prices) {
    cJSON_Delete(json_root);
    return NULL;
  }

  for (size_t i = 0; i < *prices_length; ++i) {
    const double close =
        cJSON_GetNumberValue(cJSON_GetArrayItem(json_close, i));
    const double high = cJSON_GetNumberValue(cJSON_GetArrayItem(json_high, i));
    const double low = cJSON_GetNumberValue(cJSON_GetArrayItem(json_low, i));
    const double open = cJSON_GetNumberValue(cJSON_GetArrayItem(json_open, i));
    const double volume =
        cJSON_GetNumberValue(cJSON_GetArrayItem(json_volume, i));

    prices[i] = model_price_new(close, high, low, open, volume);
    if (!prices[i]) {
      for (size_t j = 0; j < i; ++j) {
        model_price_free(prices[j]);
      }

      free(prices);
      cJSON_Delete(json_root);
      return NULL;
    }
  }

  cJSON_Delete(json_root);
  return prices;
}

char *api_yahoo_get_metadata(const char *data, const char *key) {
  if (!data || !key) {
    return NULL;
  }

  cJSON *json_root = cJSON_Parse(data);
  if (!json_root) {
    return NULL;
  }

  cJSON *json = json_root;
  json = cJSON_GetObjectItemCaseSensitive(json, "chart");
  json = cJSON_GetObjectItemCaseSensitive(json, "result");
  json = cJSON_GetArrayItem(json, 0);
  json = cJSON_GetObjectItemCaseSensitive(json, "meta");
  json = cJSON_GetObjectItemCaseSensitive(json, key);

  char *value = cJSON_GetStringValue(json);

  cJSON_Delete(json_root);

  if (!value) {
    return NULL;
  }

  value = strdup(value);
  if (!value) {
    return NULL;
  }

  return value;
}

char *api_yahoo_get_name(const char *data) {
  return api_yahoo_get_metadata(data, "longName");
}

char *api_yahoo_get_symbol(const char *data) {
  return api_yahoo_get_metadata(data, "symbol");
}

char *api_yahoo_get_url(const char *symbol) {
  if (!symbol) {
    return NULL;
  }

  const char *url_prefix = "https://query1.finance.yahoo.com/v8/finance/chart/";
  const char *url_postfix = "?interval=1d&range=1y";

  const size_t url_length =
      strlen(url_prefix) + strlen(symbol) + strlen(url_postfix) + 1;

  char *url = malloc(url_length);
  if (!url) {
    return NULL;
  }

  snprintf(url, url_length, "%s%s%s", url_prefix, symbol, url_postfix);

  return url;
}

Stock *api_yahoo_get_stock(const char *symbol_input) {
  if (!symbol_input) {
    return NULL;
  }

  char *url = api_yahoo_get_url(symbol_input);
  if (!url) {
    return NULL;
  }

  CURL *curl = api_curl_new();
  if (!curl) {
    free(url);
    return NULL;
  }

  char *data = api_curl_run(curl, url);
  if (!data) {
    api_curl_free(curl);
    free(url);
    return NULL;
  }

  size_t prices_length = 0;
  Price **prices = api_yahoo_get_prices(data, &prices_length);
  char *name = api_yahoo_get_name(data);
  char *symbol = api_yahoo_get_symbol(data);

  free(data);
  api_curl_free(curl);
  free(url);

  Stock *stock = model_stock_new(prices, prices_length, name, symbol);
  if (!stock) {
    for (size_t i = 0; i < prices_length; ++i) {
      model_price_free(prices[i]);
    }

    free(prices);
    free(name);
    free(symbol);
    return NULL;
  }

  return stock;
}
