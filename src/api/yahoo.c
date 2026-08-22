#include "src/api/yahoo.h"
#include <cjson/cJSON.h>
#include <curl/curl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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

char *services_yahoo_fetch_data(char *symbol) {
  CURL *curl;

  Response response = {.data = NULL, .size = 0};

  curl = curl_easy_init();
  if (!curl) {
    return "";
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
