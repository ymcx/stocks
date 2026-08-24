#ifndef SERVICES_YAHOO_H
#define SERVICES_YAHOO_H

#include "src/model/price.h"
#include "src/model/stock.h"
#include <cjson/cJSON.h>
#include <curl/curl.h>
#include <stddef.h>

Price **api_yahoo_get_prices(const char *data, size_t *prices_length);
char *api_yahoo_get_metadata(const char *data, const char *key);
char *api_yahoo_get_name(const char *data);
char *api_yahoo_get_symbol(const char *data);
char *api_yahoo_get_url(const char *symbol);
Stock *api_yahoo_get_stock(const char *symbol_input);

#endif
