#include "src/api/yahoo.h"
#include "src/model/price.h"
#include "src/model/stock.h"
#include <cjson/cJSON.h>
#include <curl/curl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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

char *utils_parse_metadata(char *data, char *key) {
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

  char *value = cJSON_GetStringValue(json);
  value = value ? g_strdup(value) : NULL;

  cJSON_Delete(root);

  return value;
}

char *utils_parse_name(char *data) {
  return utils_parse_metadata(data, "longName");
}

char *utils_parse_symbol(char *data) {
  return utils_parse_metadata(data, "symbol");
}

Stock utils_parse_stock(char *data) {
  int prices_length;
  Price *prices = utils_parse_prices(data, &prices_length);
  char *name = utils_parse_name(data);
  char *symbol = utils_parse_symbol(data);

  return model_stock_new(prices, prices_length, name, symbol);
}

size_t write_callback(void *contents, size_t size, size_t nmemb, void *userp) {
  size_t total = size * nmemb;
  Response *response = userp;

  char *ptr = realloc(response->data, response->size + total + 1);

  if (ptr == NULL)
    return 0;

  response->data = ptr;

  memcpy(response->data + response->size, contents, total);

  response->size += total;
  response->data[response->size] = '\0';

  return total;
}

Stock *api_yahoo_get_stock(const char *symbol) {
  CURL *curl;

  Response response = {.data = NULL, .size = 0};

  curl = curl_easy_init();
  if (!curl) {
    return NULL;
  }

  char *url = malloc(sizeof(char) * 128);
  sprintf(url,
          "https://query1.finance.yahoo.com/v8/finance/chart/"
          "%s?interval=1d&range=1y",
          symbol);
  char *ua =
      "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, "
      "like Gecko) Chrome/58.0.3029.110 Safari/537.36";
  curl_easy_setopt(curl, CURLOPT_USERAGENT, ua);
  curl_easy_setopt(curl, CURLOPT_URL, url);
  curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_callback);
  curl_easy_setopt(curl, CURLOPT_WRITEDATA, &response);

  curl_easy_perform(curl);

  return response.data;
}
