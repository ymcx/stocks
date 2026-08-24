#include "src/model/stock.h"
#include "src/model/price.h"
#include <stdlib.h>

Stock *model_stock_new(Price **prices, size_t prices_length, char *name,
                       char *symbol) {
  Stock *stock = malloc(sizeof(Stock));
  if (!stock) {
    return NULL;
  }

  stock->prices = prices;
  stock->prices_length = prices_length;
  stock->name = name;
  stock->symbol = symbol;

  return stock;
}

void model_stock_free(Stock *stock) {
  if (!stock) {
    return;
  }

  for (size_t i = 0; i < stock->prices_length; ++i) {
    model_price_free(stock->prices[i]);
  }

  free(stock->prices);
  free(stock->name);
  free(stock->symbol);
  free(stock);
}
