#ifndef UTILS_PARSE_H
#define UTILS_PARSE_H

#include "src/model/price.h"
#include "src/model/stock.h"

Price *utils_parse_prices(char *data, int *prices_length);
char *utils_parse_metadata(char *data, char *key);
char *utils_parse_name(char *data);
char *utils_parse_symbol(char *data);
Stock*utils_parse_stock(char *data) ;

#endif
