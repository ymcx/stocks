#include "src/model/stock.h"
#include "src/model/price.h"
#include <stdlib.h>

Stock *model_stock_new(Price **prices, int prices_length, char *name,
                       char *symbol) {
  Stock *stock = malloc(sizeof(Stock));
  stock->prices = prices;
  stock->prices_length = prices_length;
  stock->name = name;
  stock->symbol = symbol;

  return stock;
}
