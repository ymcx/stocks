#ifndef SERVICES_YAHOO_H
#define SERVICES_YAHOO_H

#include <stddef.h>

typedef struct {
  char *data;
  size_t size;
} Response;

#include "src/api/yahoo.h"
#include "src/model/price.h"
#include "src/model/stock.h"
#include <cjson/cJSON.h>
#include <curl/curl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

size_t write_callback(void *contents, size_t size, size_t nmemb, void *userp);
Price *utils_parse_prices(char *data, int *prices_length);
char *utils_parse_metadata(char *data, char *key);
char *utils_parse_name(char *data);
char *utils_parse_symbol(char *data);
Stock utils_parse_stock(char *data);
Stock api_yahoo_get_stock(const char *symbol) ;

#endif
