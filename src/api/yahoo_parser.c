#include "src/api/yahoo_parser.h"
#include "glib.h"
#include "src/model/price.h"
#include "src/model/stock.h"
#include <cjson/cJSON.h>
#include <stdio.h>

Price *utils_parse_prices(char *data, int *prices_length) {
  cJSON *root = cJSON_Parse(data);
  cJSON *json;

  if (!root) {
    return NULL;
  }

  json = cJSON_GetObjectItemCaseSensitive(root, "chart");
  json = cJSON_GetObjectItemCaseSensitive(json, "result");
  json = cJSON_GetArrayItem(json, 0);
  json = cJSON_GetObjectItemCaseSensitive(json, "indicators");
  json = cJSON_GetObjectItemCaseSensitive(json, "quote");
  json = cJSON_GetArrayItem(json, 0);

  cJSON *json_close = cJSON_GetObjectItemCaseSensitive(json, "close");
  cJSON *json_high = cJSON_GetObjectItemCaseSensitive(json, "high");
  cJSON *json_low = cJSON_GetObjectItemCaseSensitive(json, "low");
  cJSON *json_open = cJSON_GetObjectItemCaseSensitive(json, "open");
  cJSON *json_volume = cJSON_GetObjectItemCaseSensitive(json, "volume");

  *prices_length = cJSON_GetArraySize(json_close);
  Price *prices = malloc(sizeof(Price) * *prices_length);

  for (int i = 0; i < *prices_length; ++i) {
    double close = cJSON_GetNumberValue(cJSON_GetArrayItem(json_close, i));
    double high = cJSON_GetNumberValue(cJSON_GetArrayItem(json_high, i));
    double low = cJSON_GetNumberValue(cJSON_GetArrayItem(json_low, i));
    double open = cJSON_GetNumberValue(cJSON_GetArrayItem(json_open, i));
    double volume = cJSON_GetNumberValue(cJSON_GetArrayItem(json_volume, i));

    prices[i] = model_price_new(close, high, low, open, volume);
  }

  cJSON_Delete(root);

  return prices;
}

gchar *utils_parse_metadata(char *data, char *key) {
  cJSON *root = cJSON_Parse(data);
  cJSON *json;

  if (!root) {
    return NULL;
  }

  json = cJSON_GetObjectItemCaseSensitive(root, "chart");
  json = cJSON_GetObjectItemCaseSensitive(json, "result");
  json = cJSON_GetArrayItem(json, 0);
  json = cJSON_GetObjectItemCaseSensitive(json, "meta");
  json = cJSON_GetObjectItemCaseSensitive(json, key);

  gchar *value = cJSON_GetStringValue(json);
  value = value ? g_strdup(value) : NULL;

  cJSON_Delete(root);

  return value;
}

gchar *utils_parse_name(char *data) {
  return utils_parse_metadata(data, "longName");
}

gchar *utils_parse_symbol(char *data) {
  return utils_parse_metadata(data, "symbol");
}

Stock *utils_parse_stock(char *data) {
  int prices_length;
  Price *prices = utils_parse_prices(data, &prices_length);
  gchar *name = utils_parse_name(data);
  gchar *symbol = utils_parse_symbol(data);

  return model_stock_new(prices, prices_length, name, symbol);
}
